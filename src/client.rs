//! [`Client`], [`ClientBuilder`], [`Environment`] and the HTTP layer: sending, retries, pacing and error parsing.

use crate::error::{codes, Error, ErrorKind};
use crate::request::{Endpoint, FieldError, Method, OperationSafety, PagedRequest, Paging, Request, Validator};
use futures_util::stream::{self, BoxStream, StreamExt};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

/// The SDK version, for example `"2.0.0"`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The default `User-Agent` header.
pub const DEFAULT_USER_AGENT: &str =
    concat!("DomainNameAPI-Rust/", env!("CARGO_PKG_VERSION"), " (+https://github.com/domainreseller/rust-dna)");

/// The default `X-App` header: the application that uses the SDK. Override it with [`ClientBuilder::app`].
pub const DEFAULT_APP: &str = "rust";

/// The API deployment a client talks to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Environment {
    /// The live environment: real domains, real charges. `https://api.domainresellerapi.com`
    #[default]
    Production,
    /// The test environment (Operational Test & Evaluation), with its own data and its own API key. Nothing you do
    /// there touches live domains or your live balance. `https://ote.domainresellerapi.com`
    Ote,
}

impl Environment {
    /// Returns the environment's base address.
    pub fn base_url(self) -> &'static str {
        match self {
            Environment::Production => "https://api.domainresellerapi.com",
            Environment::Ote => "https://ote.domainresellerapi.com",
        }
    }

    /// Reads an environment name: `production`, `prod`, `live` or `ote`, `test`, `testing`, `sandbox`. Test-mode flags
    /// as stored by panels and the other SDKs work too: `true`, `1`, `on`, `yes` mean OTE and `false`, `0`, `off`,
    /// `no` (or nothing) mean production.
    pub fn parse(text: &str) -> Option<Environment> {
        match text.trim().to_ascii_lowercase().as_str() {
            "" | "production" | "prod" | "live" | "false" | "0" | "off" | "no" => Some(Environment::Production),
            "ote" | "test" | "testing" | "sandbox" | "true" | "1" | "on" | "yes" => Some(Environment::Ote),
            _ => None,
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Environment::Production => "production",
            Environment::Ote => "ote",
        })
    }
}

/// The soft-quota state the gateway reports after a request.
///
/// The quota never rejects requests: once `used` exceeds `limit`, the gateway delays each answer (by `delay_seconds`,
/// at most 5) until points decay by 20% per minute. Lookups cost 1 point, everything else 10.
#[derive(Debug, Clone, PartialEq, Default)]
#[non_exhaustive]
pub struct Quota {
    /// The area: `domain`, `contact`, `ssl`, `products`, `order`, `deposit` or `hosting`.
    pub scope: String,
    /// The points this request cost.
    pub cost: f64,
    /// The points in the bucket after this request.
    pub used: f64,
    /// Where slowing down starts (200).
    pub limit: f64,
    /// The seconds this answer was held back.
    pub delay_seconds: f64,
    /// The seconds until the bucket is back under the limit.
    pub reset_seconds: f64,
}

/// A decoded answer together with the HTTP details, returned by [`Client::send_with_response`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Response<T> {
    /// The decoded answer.
    pub value: T,
    /// The HTTP status of the final attempt.
    pub status: u16,
    /// The gateway's correlation ID (or the one the client sent). Quote it when contacting support.
    pub correlation_id: String,
    /// The soft-quota state, when the gateway reported it.
    pub quota: Option<Quota>,
    /// The number of HTTP attempts, including retries.
    pub attempts: u32,
    /// The total time, including retries and pacing.
    pub duration: Duration,
}

/// The client of the Domain Name API reseller REST API.
///
/// Each area of the API has its own client, grouped like the `dna` CLI's commands: [`account`](Client::account),
/// [`contacts`](Client::contacts), [`domains`](Client::domains), [`dns`](Client::dns), [`dnssec`](Client::dnssec),
/// [`ssl`](Client::ssl), [`hosting`](Client::hosting), [`orders`](Client::orders) and
/// [`products`](Client::products). Any request, including your own [`Request`] types, can also be sent with
/// [`send`](Client::send).
///
/// ```no_run
/// use domainnameapi::{Client, Environment};
///
/// # async fn run() -> Result<(), domainnameapi::Error> {
/// let client = Client::builder()
///     .credentials("your-reseller-id", "your-api-key")
///     .environment(Environment::Ote)
///     .build()?;
///
/// let account = client.account().get().await?;
/// println!("{}: {} USD", account.reseller_name, account.usd_balance);
/// # Ok(()) }
/// ```
///
/// A `Client` is cheap to clone (it holds an [`Arc`]) and safe to share between tasks. Create one per set of
/// credentials and reuse it: it paces requests to stay under the gateway's rate limits. Failed requests return an
/// [`Error`]. Rate limits are respected automatically, 429 answers are retried, and network errors are retried only
/// for reads and idempotent writes, so nothing is charged twice. The SDK sends nothing anywhere except your requests
/// to the API.
#[derive(Clone)]
pub struct Client {
    inner: Arc<Inner>,
}

struct Inner {
    http: reqwest::Client,
    reseller_id: String,
    api_key: String,
    environment: Environment,
    base_url: String,
    timeout: Duration,
    max_retries: u32,
    max_retry_delay: Duration,
    read_only: bool,
    user_agent: String,
    app: String,
    app_version: String,
    pacer: Option<Pacer>,
}

impl fmt::Debug for Client {
    /// Shows the environment, the base address and read-only mode. Never the credentials.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("environment", &self.inner.environment)
            .field("base_url", &self.inner.base_url)
            .field("read_only", &self.inner.read_only)
            .finish_non_exhaustive()
    }
}

impl Client {
    /// Creates a production client with default settings.
    ///
    /// `reseller_id` (a UUID) and `api_key` come from the reseller panel's API settings; panel login details do not
    /// work. Use [`Client::builder`] for the OTE test environment and other settings.
    pub fn new(reseller_id: impl Into<String>, api_key: impl Into<String>) -> Result<Client, Error> {
        Client::builder().credentials(reseller_id, api_key).build()
    }

    /// Starts building a client. Only the credentials are required.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Creates a client from the environment variables the `dna` CLI reads: `DNA_RESELLER_ID`, `DNA_API_KEY`,
    /// `DNA_ENV` (`production` or `ote`) or `DNA_TEST_MODE` (`true` for OTE), `DNA_BASE_URL`, `DNA_READ_ONLY`,
    /// `DNA_APP` and `DNA_APP_VERSION`.
    /// Use [`ClientBuilder::from_env`] to change other settings as well.
    pub fn from_env() -> Result<Client, Error> {
        ClientBuilder::from_env()?.build()
    }

    /// Returns the environment this client talks to.
    pub fn environment(&self) -> Environment {
        self.inner.environment
    }

    /// Returns the base address requests are sent to, without a trailing `/`.
    pub fn base_url(&self) -> &str {
        &self.inner.base_url
    }

    /// Returns whether the client blocks every request that is not a read.
    pub fn is_read_only(&self) -> bool {
        self.inner.read_only
    }

    /// Returns the `User-Agent` header the client sends.
    pub fn user_agent(&self) -> &str {
        &self.inner.user_agent
    }

    /// Returns the `X-App` header the client sends: the application that uses the SDK, [`DEFAULT_APP`] unless set with
    /// [`ClientBuilder::app`].
    pub fn app(&self) -> &str {
        &self.inner.app
    }

    /// Returns the `X-Version` header the client sends: the version of the application that uses the SDK, the SDK's
    /// own [`VERSION`] unless set with [`ClientBuilder::app`].
    pub fn app_version(&self) -> &str {
        &self.inner.app_version
    }

    /// Your account: balances, profile and the transaction ledger. CLI: `dna account`.
    pub fn account(&self) -> crate::account::AccountClient<'_> {
        crate::account::AccountClient::new(self)
    }

    /// Reusable contact handles and their verification. CLI: `dna contact`.
    pub fn contacts(&self) -> crate::contacts::ContactsClient<'_> {
        crate::contacts::ContactsClient::new(self)
    }

    /// Search, register, renew, transfer and manage domains, with URL forwarding, child nameservers and documents.
    /// CLI: `dna domain`.
    pub fn domains(&self) -> crate::domains::DomainsClient<'_> {
        crate::domains::DomainsClient::new(self)
    }

    /// DNS zone records of domains on the platform nameservers. CLI: `dna dns`.
    pub fn dns(&self) -> crate::dns::DnsClient<'_> {
        crate::dns::DnsClient::new(self)
    }

    /// DNSSEC (DS records) of domains. CLI: `dna dnssec`.
    pub fn dnssec(&self) -> crate::dnssec::DnssecClient<'_> {
        crate::dnssec::DnssecClient::new(self)
    }

    /// SSL certificates: buy, configure, issue, list, renew, reissue and revoke. CLI: `dna ssl`.
    pub fn ssl(&self) -> crate::ssl::SslClient<'_> {
        crate::ssl::SslClient::new(self)
    }

    /// Web hosting packages and services. CLI: `dna hosting`.
    pub fn hosting(&self) -> crate::hosting::HostingClient<'_> {
        crate::hosting::HostingClient::new(self)
    }

    /// Orders. CLI: `dna order`.
    pub fn orders(&self) -> crate::orders::OrdersClient<'_> {
        crate::orders::OrdersClient::new(self)
    }

    /// The product catalogue and prices: TLDs, SSL certificates and other products. CLI: `dna product`.
    pub fn products(&self) -> crate::products::ProductsClient<'_> {
        crate::products::ProductsClient::new(self)
    }

    /// Validates a request, sends it and decodes the answer.
    ///
    /// Errors: [`ErrorKind::InvalidInput`] when the request failed its checks and [`ErrorKind::ReadOnly`] when the
    /// client is read-only (nothing was sent in both cases); otherwise the API's error or a network failure.
    pub async fn send<R: Request + ?Sized>(&self, request: &R) -> Result<R::Response, Error> {
        self.send_with_response(request).await.map(|response| response.value)
    }

    /// Like [`send`](Client::send), and also returns the HTTP details: correlation ID, soft-quota state, number of
    /// attempts and duration.
    pub async fn send_with_response<R: Request + ?Sized>(&self, request: &R) -> Result<Response<R::Response>, Error> {
        let prepared = self.prepare(request)?;
        let exchange = self.execute(&prepared).await?;
        let value = decode::<R::Response>(&prepared, &exchange)?;
        Ok(Response {
            value,
            status: exchange.status,
            correlation_id: exchange.correlation_id,
            quota: exchange.quota,
            attempts: exchange.attempts,
            duration: exchange.duration,
        })
    }

    /// Pages through a list request and yields every item. A page is fetched only when the stream needs it. The
    /// request's own skip is the starting point and its limit the page size ([`Paging::DEFAULT_PAGE_SIZE`] when not
    /// set). The stream ends after the first error, which it yields.
    ///
    /// ```no_run
    /// use domainnameapi::domains::DomainListRequest;
    /// use futures_util::TryStreamExt;
    ///
    /// # async fn run(client: domainnameapi::Client) -> Result<(), domainnameapi::Error> {
    /// let mut domains = client.list_all(DomainListRequest::new().domain_name("example"));
    /// while let Some(domain) = domains.try_next().await? {
    ///     println!("{} expires in {} days", domain.domain_name, domain.remaining_days);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn list_all<T, R>(&self, request: R) -> BoxStream<'static, Result<T, Error>>
    where
        T: Send + 'static,
        R: PagedRequest<T> + Send + Sync + 'static,
    {
        let page_size = request.paging().limit.filter(|n| *n > 0).unwrap_or(Paging::DEFAULT_PAGE_SIZE);
        let state = PageState {
            client: self.clone(),
            skip: request.paging().skip.unwrap_or(0),
            request,
            page_size,
            buffer: VecDeque::new(),
            done: false,
        };
        stream::try_unfold(state, |mut state| async move {
            loop {
                if let Some(item) = state.buffer.pop_front() {
                    return Ok(Some((item, state)));
                }
                if state.done {
                    return Ok(None);
                }
                let mut request = state.request.clone();
                {
                    let paging = request.paging_mut();
                    paging.skip = Some(state.skip);
                    paging.limit = Some(state.page_size);
                }
                let page = state.client.send(&request).await?;
                let count = page.items.len() as u32;
                state.skip += count;
                state.done = count == 0
                    || (page.total_count > 0 && i64::from(state.skip) >= page.total_count)
                    || (page.total_count <= 0 && count < state.page_size);
                state.buffer.extend(page.items);
            }
        })
        .boxed()
    }

    // ------------------------------------------------------------------------------------------------ transport

    fn prepare<R: Request + ?Sized>(&self, request: &R) -> Result<Prepared, Error> {
        let mut endpoint = request.endpoint();
        if !endpoint.path.starts_with('/') {
            endpoint.path.insert(0, '/');
        }
        let operation = endpoint.to_string();

        let mut validator = Validator::new();
        request.validate(&mut validator);
        if !validator.is_valid() {
            let errors = validator.into_errors();
            let summary: Vec<String> = errors.iter().map(FieldError::to_string).collect();
            return Err(Error::new(
                ErrorKind::InvalidInput,
                codes::INVALID_INPUT,
                format!("{operation} was not sent: {}", summary.join("; ")),
            )
            .with_validation_errors(errors)
            .with_operation(operation, endpoint.safety)
            .fill_hint());
        }

        if self.inner.read_only && endpoint.safety != OperationSafety::Read {
            return Err(Error::new(
                ErrorKind::ReadOnly,
                codes::READ_ONLY,
                format!("{operation} is a {} operation and the client is read-only; nothing was sent", endpoint.safety),
            )
            .with_operation(operation, endpoint.safety)
            .fill_hint());
        }

        let mut url = format!("{}{}", self.inner.base_url, endpoint.path);
        let query = request.query();
        if !query.is_empty() {
            let encoded: Vec<String> =
                query.pairs().iter().map(|(k, v)| format!("{}={}", encode(k), encode(v))).collect();
            url.push('?');
            url.push_str(&encoded.join("&"));
        }
        let body = request.body().map(|b| b.to_string());
        Ok(Prepared { endpoint, operation, url, body })
    }

    async fn execute(&self, prepared: &Prepared) -> Result<Exchange, Error> {
        let started = Instant::now();
        let correlation_id = new_correlation_id();
        let retry_safe = prepared.endpoint.is_retry_safe();
        let safety = prepared.endpoint.safety;

        let mut attempt: u32 = 1;
        loop {
            if let Some(pacer) = &self.inner.pacer {
                pacer.wait(&prepared.endpoint.path).await;
            }

            let result = self.build(prepared, &correlation_id).send().await;
            let response = match result {
                Ok(response) => response,
                Err(e) => {
                    if retry_safe && attempt <= self.inner.max_retries {
                        tokio::time::sleep(backoff(attempt)).await;
                        attempt += 1;
                        continue;
                    }
                    return Err(self.network_error(prepared, e, &correlation_id));
                }
            };

            let status = response.status().as_u16();
            let headers = Headers::from(response.headers());
            let redirected = redirected_to_error_page(&prepared.url, response.url());
            let text = match response.text().await {
                Ok(text) => text,
                Err(e) => {
                    if retry_safe && attempt <= self.inner.max_retries {
                        tokio::time::sleep(backoff(attempt)).await;
                        attempt += 1;
                        continue;
                    }
                    return Err(self.network_error(prepared, e, &correlation_id));
                }
            };
            let response_id =
                headers.get("x-correlation-id").filter(|s| !s.is_empty()).unwrap_or(&correlation_id).to_string();

            if (200..300).contains(&status) && !redirected {
                return Ok(Exchange {
                    status,
                    body: text,
                    correlation_id: response_id,
                    quota: quota(&headers),
                    attempts: attempt,
                    duration: started.elapsed(),
                });
            }

            let retry = status == 429 || (retry_safe && matches!(status, 502..=504));
            if retry && !redirected && attempt <= self.inner.max_retries {
                let wait = retry_after(headers.get("retry-after"))
                    .map(|d| d + Duration::from_millis(50))
                    .unwrap_or_else(|| backoff(attempt));
                if wait <= self.inner.max_retry_delay {
                    tokio::time::sleep(wait).await;
                    attempt += 1;
                    continue;
                }
            }

            return Err(api_error(status, redirected, &headers, &text)
                .with_correlation_id(response_id)
                .with_operation(prepared.operation.clone(), safety)
                .fill_hint());
        }
    }

    fn build(&self, prepared: &Prepared, correlation_id: &str) -> reqwest::RequestBuilder {
        let method = match prepared.endpoint.method {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Delete => reqwest::Method::DELETE,
            Method::Patch => reqwest::Method::PATCH,
        };
        let mut builder = self
            .inner
            .http
            .request(method, &prepared.url)
            .timeout(self.inner.timeout)
            .header("Accept", "application/json")
            .header("User-Agent", &self.inner.user_agent)
            .header("X-API-KEY", &self.inner.api_key)
            .header("__reseller", &self.inner.reseller_id)
            .header("X-Correlation-ID", correlation_id)
            .header("X-App", &self.inner.app)
            .header("X-Version", &self.inner.app_version);
        if let Some(body) = &prepared.body {
            builder = builder.header("Content-Type", "application/json; charset=utf-8").body(body.clone());
        }
        builder
    }

    fn network_error(&self, prepared: &Prepared, e: reqwest::Error, correlation_id: &str) -> Error {
        let (kind, code, message) = if e.is_timeout() {
            (
                ErrorKind::Timeout,
                codes::TIMEOUT,
                format!("No answer from the API within {} s.", self.inner.timeout.as_secs_f64()),
            )
        } else {
            (ErrorKind::Network, codes::NETWORK, format!("The request could not be completed: {}", describe(&e)))
        };
        Error::new(kind, code, message)
            .with_correlation_id(correlation_id)
            .with_operation(prepared.operation.clone(), prepared.endpoint.safety)
            .with_source(e)
            .fill_hint()
    }
}

/// Response headers with lower-case names.
struct Headers(HashMap<String, String>);

impl Headers {
    fn get(&self, name: &str) -> Option<&str> {
        self.0.get(name).map(String::as_str)
    }
}

impl From<&reqwest::header::HeaderMap> for Headers {
    fn from(map: &reqwest::header::HeaderMap) -> Headers {
        let mut headers = HashMap::new();
        for (name, value) in map {
            if let Ok(value) = value.to_str() {
                headers
                    .entry(name.as_str().to_ascii_lowercase())
                    .and_modify(|v: &mut String| {
                        v.push_str(", ");
                        v.push_str(value.trim());
                    })
                    .or_insert_with(|| value.trim().to_string());
            }
        }
        Headers(headers)
    }
}

struct Prepared {
    endpoint: Endpoint,
    operation: String,
    url: String,
    body: Option<String>,
}

struct Exchange {
    status: u16,
    body: String,
    correlation_id: String,
    quota: Option<Quota>,
    attempts: u32,
    duration: Duration,
}

struct PageState<R, T> {
    client: Client,
    request: R,
    skip: u32,
    page_size: u32,
    buffer: VecDeque<T>,
    done: bool,
}

fn decode<T: serde::de::DeserializeOwned>(prepared: &Prepared, exchange: &Exchange) -> Result<T, Error> {
    let text = exchange.body.trim();
    let error = |code: &str, message: String| {
        Error::new(ErrorKind::Unknown, code, message)
            .with_status(exchange.status)
            .with_correlation_id(exchange.correlation_id.clone())
            .with_response_body(&exchange.body)
            .with_operation(prepared.operation.clone(), prepared.endpoint.safety)
    };
    if text.is_empty() || text == "null" {
        // Operations without an answer decode `null` into `Empty`; anything else needed a body.
        return serde_json::from_str::<T>("null").map_err(|_| {
            error(codes::EMPTY_RESPONSE, format!("The API answered {} with an empty body.", prepared.operation))
        });
    }
    serde_json::from_str::<T>(text).map_err(|e| {
        error(codes::INVALID_RESPONSE, format!("The answer to {} could not be read: {e}", prepared.operation))
            .with_source(e)
    })
}

/// Builds the error for a non-2xx answer (or a redirect to the gateway's error page).
fn api_error(status: u16, redirected: bool, headers: &Headers, body: &str) -> Error {
    // The multi-reseller middleware redirects to /Error/Suspend when the __reseller value is unknown, inactive or
    // suspended in this environment.
    if matches!(status, 301 | 302) || redirected {
        return Error::new(
            ErrorKind::Auth,
            codes::INVALID_RESELLER,
            "Authentication failed: the reseller ID is unknown, inactive or suspended in this environment.",
        )
        .with_status(status)
        .with_response_body(body);
    }
    if status == 429 {
        let wait = retry_after(headers.get("retry-after"));
        let message = match wait {
            Some(d) => format!("Rate limit exceeded; retry after {} s.", d.as_secs_f64()),
            None => "Rate limit exceeded.".to_string(),
        };
        return Error::new(ErrorKind::RateLimited, codes::RATE_LIMITED, message)
            .with_status(status)
            .with_retry_after(wait)
            .with_response_body(body);
    }

    let mut code = String::new();
    let mut message = String::new();
    let mut details = String::new();
    let mut fields = Vec::new();
    let trimmed = body.trim();
    if let Some(Value::Object(root)) =
        trimmed.starts_with('{').then(|| serde_json::from_str::<Value>(trimmed).ok()).flatten()
    {
        // ABP wraps errors as {"error": {...}}; some endpoints answer with a flat object.
        let error = match property(&root, "error") {
            Some(Value::Object(inner)) => inner.clone(),
            _ => root.clone(),
        };
        code = text(&error, "code");
        message = text(&error, "message");
        details = text(&error, "details");
        if let Some(Value::Array(items)) = property(&error, "validationErrors") {
            for item in items {
                match item {
                    Value::String(s) => fields.push(FieldError { field: String::new(), message: s.clone() }),
                    Value::Object(o) => {
                        let members: Vec<String> = match property(o, "members") {
                            Some(Value::Array(m)) => m
                                .iter()
                                .filter_map(Value::as_str)
                                .map(|s| s.trim_start_matches("$.").to_string())
                                .collect(),
                            _ => Vec::new(),
                        };
                        fields.push(FieldError { field: members.join(","), message: text(o, "message") });
                    }
                    _ => {}
                }
            }
        }
    } else if !trimmed.is_empty() && !trimmed.starts_with('<') {
        // 401 bodies are plain text; HTML error pages are dropped.
        message = trimmed.to_string();
    }

    if status == 401 && code.is_empty() {
        let lower = message.to_lowercase();
        code = if lower.contains("api key is missing") {
            codes::MISSING_API_KEY
        } else if lower.contains("resellerid is missing") {
            codes::MISSING_RESELLER_ID
        } else if lower.contains("invalid api key") {
            codes::INVALID_API_KEY
        } else if lower.contains("authorisation") || lower.contains("authorization") {
            codes::SUBRESELLER_NOT_ALLOWED
        } else {
            codes::UNAUTHORIZED
        }
        .to_string();
    }

    // Replace the generic .NET exception text, or an echoed error code, with the details.
    if is_boilerplate(&message) || (!message.is_empty() && message == code) {
        let d = details.trim();
        let d = d.strip_prefix("Dna ").unwrap_or(d).trim();
        message = if !d.is_empty() && !is_boilerplate(d) && d != code {
            d.to_string()
        } else {
            format!("The operation could not be completed (error {code}).")
        };
    }
    if code.is_empty() {
        code = format!("HTTP_{status}");
    }
    if message.trim().is_empty() {
        message = format!("HTTP {status} {}.", reason_phrase(status));
    }

    Error::new(ErrorKind::from_status(status), code, message)
        .with_status(status)
        .with_details((!details.is_empty()).then_some(details))
        .with_validation_errors(fields)
        .with_response_body(body)
}

fn is_boilerplate(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.find("exception of type '").is_some_and(|i| lower[i + 19..].contains("' was thrown"))
}

fn property<'v>(object: &'v serde_json::Map<String, Value>, name: &str) -> Option<&'v Value> {
    object.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v)
}

fn text(object: &serde_json::Map<String, Value>, name: &str) -> String {
    match property(object, name) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

fn reason_phrase(status: u16) -> &'static str {
    reqwest::StatusCode::from_u16(status).ok().and_then(|s| s.canonical_reason()).unwrap_or("Unexpected Status")
}

fn quota(headers: &Headers) -> Option<Quota> {
    if headers.get("x-quota-limit").is_none() && headers.get("x-quota-used").is_none() {
        return None;
    }
    let number = |name: &str| headers.get(name).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
    Some(Quota {
        scope: headers.get("x-quota-scope").unwrap_or_default().to_string(),
        cost: number("x-quota-cost"),
        used: number("x-quota-used"),
        limit: number("x-quota-limit"),
        delay_seconds: number("x-quota-delay"),
        reset_seconds: number("x-quota-reset"),
    })
}

/// Reads `Retry-After` as seconds or as an HTTP date.
fn retry_after(value: Option<&str>) -> Option<Duration> {
    let value = value?.trim();
    if let Ok(seconds) = value.parse::<f64>() {
        return (seconds.is_finite() && seconds >= 0.0).then(|| Duration::from_secs_f64(seconds));
    }
    let date = httpdate::parse_http_date(value).ok()?;
    Some(date.duration_since(SystemTime::now()).unwrap_or(Duration::ZERO))
}

/// 500 ms, 1 s, 2 s, ... plus up to 250 ms of jitter.
fn backoff(attempt: u32) -> Duration {
    let base = 500u64 << attempt.saturating_sub(1).min(6);
    Duration::from_millis(base + random_u64() % 250)
}

/// A custom `reqwest::Client` may follow redirects. The gateway redirects unknown or suspended resellers to
/// /Error/Suspend, which then answers 200 with an HTML page; treat that as the redirect it was.
fn redirected_to_error_page(requested: &str, final_url: &reqwest::Url) -> bool {
    let requested_path = reqwest::Url::parse(requested).map(|u| u.path().to_string()).unwrap_or_default();
    final_url.path() != requested_path && final_url.path().to_ascii_lowercase().contains("/error/")
}

fn describe(e: &reqwest::Error) -> String {
    use std::error::Error as _;
    let mut text = e.to_string();
    let mut source = e.source();
    while let Some(s) = source {
        text.push_str(": ");
        text.push_str(&s.to_string());
        source = s.source();
    }
    text
}

/// Percent-encodes a query string key or value.
fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn random_u64() -> u64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u64(COUNTER.fetch_add(1, Ordering::Relaxed));
    hasher.write_u128(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_nanos());
    hasher.finish()
}

/// 32 hexadecimal digits, sent as `X-Correlation-ID`. The gateway logs it.
fn new_correlation_id() -> String {
    format!("{:016x}{:016x}", random_u64(), random_u64())
}

// ------------------------------------------------------------------------------------------------ pacing

/// Spaces requests per rate-limit class: 5 requests/second in general, 2/second for domain searches (which also count
/// against the general budget) and one DNS zone call per second. Domain info and the TLD list are not limited.
struct Pacer {
    next: Mutex<HashMap<&'static str, Instant>>,
}

impl Pacer {
    fn new() -> Pacer {
        Pacer { next: Mutex::new(HashMap::new()) }
    }

    fn class(path: &str) -> Option<(&'static str, Duration)> {
        let p = path.to_ascii_lowercase();
        if p.starts_with("/api/v1/domains/zones") {
            Some(("zones", Duration::from_secs(1)))
        } else if p == "/api/v1/domains/search" || p == "/api/v1/domains/bulk-search" {
            Some(("search", Duration::from_millis(500)))
        } else if p == "/api/v1/domains/info" || p.starts_with("/api/v1/products/tlds") {
            None
        } else {
            Some(("general", Duration::from_millis(200)))
        }
    }

    async fn wait(&self, path: &str) {
        let Some((class, gap)) = Pacer::class(path) else { return };
        let slot = {
            let mut next = self.next.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let now = Instant::now();
            let slot = next.get(class).copied().filter(|t| *t > now).unwrap_or(now);
            next.insert(class, slot + gap);
            if class == "search" {
                let general = next.get("general").copied().unwrap_or(now);
                if general < slot + Duration::from_millis(200) {
                    next.insert("general", slot + Duration::from_millis(200));
                }
            }
            slot
        };
        tokio::time::sleep_until(slot.into()).await;
    }
}

// ------------------------------------------------------------------------------------------------ builder

/// Settings of a [`Client`]. Only the credentials are required; everything else has a sensible default.
///
/// ```
/// use domainnameapi::{Client, Environment};
/// use std::time::Duration;
///
/// let client = Client::builder()
///     .credentials("reseller-id", "api-key")
///     .environment(Environment::Ote)
///     .timeout(Duration::from_secs(30))
///     .read_only(true)
///     .build()?;
/// assert_eq!(client.base_url(), "https://ote.domainresellerapi.com");
/// # Ok::<(), domainnameapi::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    reseller_id: String,
    api_key: String,
    environment: Environment,
    base_url: Option<String>,
    timeout: Duration,
    max_retries: u32,
    max_retry_delay: Duration,
    pace_requests: bool,
    read_only: bool,
    user_agent: Option<String>,
    app: Option<String>,
    app_version: Option<String>,
    http: Option<reqwest::Client>,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        ClientBuilder {
            reseller_id: String::new(),
            api_key: String::new(),
            environment: Environment::Production,
            base_url: None,
            timeout: Duration::from_secs(60),
            max_retries: 2,
            max_retry_delay: Duration::from_secs(30),
            pace_requests: true,
            read_only: false,
            user_agent: None,
            app: None,
            app_version: None,
            http: None,
        }
    }
}

impl ClientBuilder {
    /// Returns a builder filled from the environment variables the Domain Name API tools share:
    ///
    /// | Variable | Meaning |
    /// |---|---|
    /// | `DNA_RESELLER_ID`, `DNA_API_KEY` | The credentials. |
    /// | `DNA_ENV` | `production` or `ote` (also `prod`, `live`, `test`, `sandbox`). |
    /// | `DNA_TEST_MODE` | `true`, `1`, `on` or `yes` selects OTE. Used when `DNA_ENV` is not set. |
    /// | `DNA_BASE_URL` | An optional base address override. |
    /// | `DNA_READ_ONLY` | `true`, `1`, `on` or `yes` turns on read-only mode. |
    /// | `DNA_APP`, `DNA_APP_VERSION` | The application that uses the SDK and its version, sent as `X-App` and `X-Version` (see [`app`](ClientBuilder::app)). |
    ///
    /// Fails when `DNA_ENV` or `DNA_BASE_URL` cannot be read. Missing credentials are reported by
    /// [`build`](ClientBuilder::build).
    pub fn from_env() -> Result<ClientBuilder, Error> {
        ClientBuilder::from_vars(|name| std::env::var(name).ok())
    }

    pub(crate) fn from_vars(var: impl Fn(&str) -> Option<String>) -> Result<ClientBuilder, Error> {
        let get = |name: &str| var(name).map(|v| v.trim().to_string()).unwrap_or_default();
        let mut builder = ClientBuilder::default().credentials(get("DNA_RESELLER_ID"), get("DNA_API_KEY"));

        let mut env = get("DNA_ENV");
        if env.is_empty() {
            env = get("DNA_TEST_MODE");
        }
        builder.environment = Environment::parse(&env)
            .ok_or_else(|| Error::configuration(format!("DNA_ENV must be production or ote, not '{env}'.")))?;

        let base_url = get("DNA_BASE_URL");
        if !base_url.is_empty() {
            check_base_url(&base_url)?;
            builder.base_url = Some(base_url);
        }
        builder.read_only = matches!(get("DNA_READ_ONLY").to_ascii_lowercase().as_str(), "true" | "1" | "on" | "yes");
        builder.app = Some(get("DNA_APP"));
        builder.app_version = Some(get("DNA_APP_VERSION"));
        Ok(builder)
    }

    /// Sets the credentials, sent with every request as the `__reseller` and `X-API-KEY` headers. Both come from the
    /// reseller panel's API settings; production and OTE use different API keys. Surrounding spaces are removed.
    pub fn credentials(mut self, reseller_id: impl Into<String>, api_key: impl Into<String>) -> Self {
        self.reseller_id = reseller_id.into().trim().to_string();
        self.api_key = api_key.into().trim().to_string();
        self
    }

    /// Chooses the deployment. [`Environment::Production`] by default.
    pub fn environment(mut self, environment: Environment) -> Self {
        self.environment = environment;
        self
    }

    /// Overrides the environment's base address, for example for a proxy or a local test server
    /// (`http://127.0.0.1:8080`). A path prefix is kept.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Sets the time limit of one attempt. 60 seconds by default. Keep it well above 5 seconds: when the soft quota
    /// is exhausted the gateway delays answers by up to 5 seconds.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets how often a failed request is repeated. 2 by default; 0 turns retries off. 429 answers are always retried
    /// (the gateway refused them before anything ran). Network errors and 502/503/504 answers are retried only for
    /// reads and idempotent writes, so nothing is charged twice.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Sets the longest wait before a retry. A `Retry-After` longer than this is not waited for. 30 seconds by
    /// default.
    pub fn max_retry_delay(mut self, max_retry_delay: Duration) -> Self {
        self.max_retry_delay = max_retry_delay;
        self
    }

    /// Spaces requests to stay under the gateway's per-key rate limits: 5 requests/second in general, 2/second for
    /// domain searches and 1/second for DNS zone calls. On by default.
    pub fn pace_requests(mut self, pace_requests: bool) -> Self {
        self.pace_requests = pace_requests;
        self
    }

    /// Blocks every request that is not a read ([`OperationSafety::Read`]) before it is sent. Useful while exploring
    /// the API, in reports and in dashboards. Off by default.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Sets the `User-Agent` header. Defaults to [`DEFAULT_USER_AGENT`]. Put your product in it, for example
    /// `"my-panel/1.4 DomainNameAPI-Rust/2.0.0"`, to help support find your requests. The API sits behind Cloudflare,
    /// which rejects empty or generic agents.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Names the application that uses the SDK, sent with every request as the `X-App` and `X-Version` headers so the
    /// API team can tell integrations apart. Defaults to [`DEFAULT_APP`] (`rust`) and the SDK's [`VERSION`].
    ///
    /// The name is lower-cased and keeps only `a`-`z`, `0`-`9`, `.`, `_` and `-`; the version keeps only digits and
    /// dots, without leading or trailing dots. `("WiseCP5", "V2.0.20-beta")` is sent as `wisecp5` and `2.0.20`. A name
    /// that is empty after this falls back to `rust`, an empty version to the SDK's version.
    ///
    /// ```
    /// use domainnameapi::Client;
    ///
    /// let client = Client::builder().credentials("reseller-id", "api-key").app("WiseCP5", "V2.0.20-beta").build()?;
    /// assert_eq!(client.app(), "wisecp5");
    /// assert_eq!(client.app_version(), "2.0.20");
    /// # Ok::<(), domainnameapi::Error>(())
    /// ```
    pub fn app(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.app = Some(name.into());
        self.app_version = Some(version.into());
        self
    }

    /// Uses your own `reqwest::Client`, for example with a proxy or custom TLS settings. The SDK's own client follows
    /// no redirects; a client that does is still handled. The time limit of each attempt is set per request, so the
    /// client's own timeout does not matter.
    pub fn http_client(mut self, http: reqwest::Client) -> Self {
        self.http = Some(http);
        self
    }

    /// Creates the client.
    ///
    /// Fails with [`codes::NO_CREDENTIALS`] when the reseller ID or API key is empty, and with
    /// [`codes::INVALID_CONFIGURATION`] for an invalid base address or timeout.
    pub fn build(self) -> Result<Client, Error> {
        if self.reseller_id.is_empty() || self.api_key.is_empty() {
            return Err(Error::new(
                ErrorKind::Auth,
                codes::NO_CREDENTIALS,
                "The reseller ID and API key are required: find both in the reseller panel under API settings.",
            )
            .fill_hint());
        }
        if self.timeout.is_zero() {
            return Err(Error::configuration("The timeout must be positive."));
        }
        let base_url = match &self.base_url {
            Some(url) => {
                check_base_url(url)?;
                url.trim().trim_end_matches('/').to_string()
            }
            None => self.environment.base_url().to_string(),
        };
        let http = match self.http {
            Some(http) => http,
            None => reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(self.timeout.min(Duration::from_secs(30)))
                .build()
                .map_err(|e| {
                    Error::configuration(format!("The HTTP client could not be created: {e}")).with_source(e)
                })?,
        };
        let user_agent = self
            .user_agent
            .map(|ua| ua.trim().to_string())
            .filter(|ua| !ua.is_empty())
            .unwrap_or_else(|| DEFAULT_USER_AGENT.to_string());
        let app = Some(normalize_app(self.app.as_deref().unwrap_or_default()))
            .filter(|app| !app.is_empty())
            .unwrap_or_else(|| DEFAULT_APP.to_string());
        let app_version = Some(normalize_version(self.app_version.as_deref().unwrap_or_default()))
            .filter(|version| !version.is_empty())
            .unwrap_or_else(|| normalize_version(VERSION));
        Ok(Client {
            inner: Arc::new(Inner {
                http,
                reseller_id: self.reseller_id,
                api_key: self.api_key,
                environment: self.environment,
                base_url,
                timeout: self.timeout,
                max_retries: self.max_retries,
                max_retry_delay: self.max_retry_delay,
                read_only: self.read_only,
                user_agent,
                app,
                app_version,
                pacer: self.pace_requests.then(Pacer::new),
            }),
        })
    }
}

/// The `X-App` form of an application name: lower case, only `a`-`z`, `0`-`9`, `.`, `_` and `-`.
fn normalize_app(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'))
        .collect()
}

/// The `X-Version` form of a version: digits and dots only, without leading or trailing dots.
fn normalize_version(version: &str) -> String {
    let digits: String = version.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
    digits.trim_matches('.').to_string()
}

fn check_base_url(url: &str) -> Result<(), Error> {
    match reqwest::Url::parse(url.trim()) {
        Ok(u) if matches!(u.scheme(), "http" | "https") && u.host().is_some() => Ok(()),
        _ => Err(Error::configuration(format!(
            "The base address must be an absolute http(s) URL such as https://api.domainresellerapi.com, not '{url}'."
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn vars(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        move |name| map.get(name).cloned()
    }

    #[test]
    fn environments_parse() {
        assert_eq!(Environment::parse("OTE"), Some(Environment::Ote));
        assert_eq!(Environment::parse("true"), Some(Environment::Ote));
        assert_eq!(Environment::parse(""), Some(Environment::Production));
        assert_eq!(Environment::parse("live"), Some(Environment::Production));
        assert_eq!(Environment::parse("staging"), None);
    }

    #[test]
    fn builder_from_vars() {
        let client = ClientBuilder::from_vars(vars(&[
            ("DNA_RESELLER_ID", " id "),
            ("DNA_API_KEY", "key"),
            ("DNA_TEST_MODE", "1"),
            ("DNA_READ_ONLY", "yes"),
        ]))
        .unwrap()
        .build()
        .unwrap();
        assert_eq!(client.environment(), Environment::Ote);
        assert_eq!(client.base_url(), "https://ote.domainresellerapi.com");
        assert!(client.is_read_only());

        let client = ClientBuilder::from_vars(vars(&[
            ("DNA_RESELLER_ID", "id"),
            ("DNA_API_KEY", "key"),
            ("DNA_ENV", "production"),
            ("DNA_TEST_MODE", "true"),
            ("DNA_BASE_URL", "http://127.0.0.1:9/prefix/"),
        ]))
        .unwrap()
        .build()
        .unwrap();
        assert_eq!(client.environment(), Environment::Production);
        assert_eq!(client.base_url(), "http://127.0.0.1:9/prefix");
        assert!(!client.is_read_only());

        let e = ClientBuilder::from_vars(vars(&[("DNA_ENV", "staging")])).unwrap_err();
        assert_eq!(e.code(), codes::INVALID_CONFIGURATION);
        let e = ClientBuilder::from_vars(vars(&[("DNA_BASE_URL", "not a url")])).unwrap_err();
        assert_eq!(e.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn app_names_and_versions_are_normalized() {
        assert_eq!(normalize_app("WiseCP5"), "wisecp5");
        assert_eq!(normalize_app("  My Panel_v2.x-beta! "), "mypanel_v2.x-beta");
        assert_eq!(normalize_app("Çözüm"), "zm");
        assert_eq!(normalize_app(""), "");
        assert_eq!(normalize_version("V2.0.20-beta"), "2.0.20");
        assert_eq!(normalize_version("..1.2.."), "1.2");
        assert_eq!(normalize_version("v.3"), "3");
        assert_eq!(normalize_version("beta"), "");
        assert_eq!(normalize_version(VERSION), VERSION);
    }

    #[test]
    fn app_defaults_and_fallbacks() {
        let client = Client::new("id", "key").unwrap();
        assert_eq!(client.app(), DEFAULT_APP);
        assert_eq!(client.app_version(), VERSION);

        let client = Client::builder().credentials("id", "key").app("!!!", "beta").build().unwrap();
        assert_eq!(client.app(), "rust");
        assert_eq!(client.app_version(), VERSION);

        let client =
            ClientBuilder::from_vars(vars(&[("DNA_RESELLER_ID", "id"), ("DNA_API_KEY", "key"), ("DNA_APP", "WHMCS")]))
                .unwrap()
                .build()
                .unwrap();
        assert_eq!(client.app(), "whmcs");
        assert_eq!(client.app_version(), VERSION);

        let client = ClientBuilder::from_vars(vars(&[
            ("DNA_RESELLER_ID", "id"),
            ("DNA_API_KEY", "key"),
            ("DNA_APP", " WiseCP5 "),
            ("DNA_APP_VERSION", "V2.0.20-beta"),
        ]))
        .unwrap()
        .build()
        .unwrap();
        assert_eq!(client.app(), "wisecp5");
        assert_eq!(client.app_version(), "2.0.20");
    }

    #[test]
    fn missing_credentials_fail_at_build() {
        let e = ClientBuilder::from_vars(vars(&[])).unwrap().build().unwrap_err();
        assert_eq!(e.code(), codes::NO_CREDENTIALS);
        assert_eq!(e.kind(), ErrorKind::Auth);
        assert!(e.hint().unwrap().contains("DNA_RESELLER_ID"));
    }

    #[test]
    fn debug_hides_credentials() {
        let client = Client::new("secret-id", "secret-key").unwrap();
        let text = format!("{client:?}");
        assert!(!text.contains("secret"));
        assert!(text.contains("Production"));
    }

    #[test]
    fn retry_after_reads_seconds_and_dates() {
        assert_eq!(retry_after(Some("2")), Some(Duration::from_secs(2)));
        assert_eq!(retry_after(Some("0.5")), Some(Duration::from_millis(500)));
        assert_eq!(retry_after(Some("Wed, 21 Oct 2015 07:28:00 GMT")), Some(Duration::ZERO));
        assert_eq!(retry_after(Some("soon")), None);
        assert_eq!(retry_after(None), None);
    }

    #[test]
    fn backoff_grows() {
        let first = backoff(1);
        assert!(first >= Duration::from_millis(500) && first < Duration::from_millis(750));
        assert!(backoff(3) >= Duration::from_secs(2));
    }

    #[test]
    fn pacing_classes() {
        assert_eq!(Pacer::class("/api/v1/domains/zones/apply").unwrap().0, "zones");
        assert_eq!(Pacer::class("/api/v1/domains/search").unwrap().0, "search");
        assert!(Pacer::class("/api/v1/domains/info").is_none());
        assert!(Pacer::class("/api/v1/products/tlds").is_none());
        assert_eq!(Pacer::class("/api/v1/domains/renew").unwrap().0, "general");
    }

    #[test]
    fn boilerplate_is_detected() {
        assert!(is_boilerplate("Exception of type 'Volo.Abp.BusinessException' was thrown."));
        assert!(!is_boilerplate("Domain not found"));
    }

    #[test]
    fn correlation_ids_are_unique_hex() {
        let a = new_correlation_id();
        assert_eq!(a.len(), 32);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, new_correlation_id());
    }

    #[test]
    fn query_values_are_encoded() {
        assert_eq!(encode("a b&c=d/é"), "a%20b%26c%3Dd%2F%C3%A9");
        assert_eq!(encode("2026-01-01T00:00:00+00:00"), "2026-01-01T00%3A00%3A00%2B00%3A00");
    }
}
