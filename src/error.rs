//! The [`Error`] type, its [`ErrorKind`] and the error [`codes`].

use crate::request::{FieldError, OperationSafety};
use std::fmt;
use std::time::Duration;

/// A coarse, stable classification of failures, the same as the `dna` CLI's. Branch on it instead of parsing
/// messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Missing or wrong credentials, or an unknown, inactive or suspended reseller in this environment (HTTP 401 or a
    /// redirect).
    Auth,
    /// Authenticated but not allowed (HTTP 403).
    Forbidden,
    /// The domain, contact, certificate, order or other resource does not exist, or is not yours (HTTP 404).
    NotFound,
    /// The API refused the input or a business rule failed: bad field, domain locked, not renewable, insufficient
    /// balance (HTTP 400, 422).
    Validation,
    /// The resource is in a state that does not allow the operation (HTTP 409).
    Conflict,
    /// Still rate limited after the automatic retries (HTTP 429).
    RateLimited,
    /// A server-side failure (HTTP 5xx). Often temporary.
    Server,
    /// The request could not be completed on the network: DNS, TLS, proxy, connection reset.
    Network,
    /// No answer within the client's timeout.
    Timeout,
    /// The request or the client settings are invalid; nothing was sent.
    InvalidInput,
    /// Blocked by the client's read-only mode; nothing was sent.
    ReadOnly,
    /// Anything else, including an answer the SDK could not read.
    Unknown,
}

impl ErrorKind {
    /// Returns the CLI's name of the kind: `"auth"`, `"forbidden"`, `"not_found"`, `"validation"`, `"conflict"`,
    /// `"rate_limited"`, `"server"`, `"network"`, `"timeout"`, `"invalid_input"`, `"read_only"` or `"unknown"`.
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorKind::Auth => "auth",
            ErrorKind::Forbidden => "forbidden",
            ErrorKind::NotFound => "not_found",
            ErrorKind::Validation => "validation",
            ErrorKind::Conflict => "conflict",
            ErrorKind::RateLimited => "rate_limited",
            ErrorKind::Server => "server",
            ErrorKind::Network => "network",
            ErrorKind::Timeout => "timeout",
            ErrorKind::InvalidInput => "invalid_input",
            ErrorKind::ReadOnly => "read_only",
            ErrorKind::Unknown => "unknown",
        }
    }

    /// Returns the kind for an HTTP status.
    pub fn from_status(status: u16) -> ErrorKind {
        match status {
            301 | 302 | 401 => ErrorKind::Auth,
            403 => ErrorKind::Forbidden,
            404 => ErrorKind::NotFound,
            400 | 422 => ErrorKind::Validation,
            409 => ErrorKind::Conflict,
            429 => ErrorKind::RateLimited,
            s if s >= 500 => ErrorKind::Server,
            _ => ErrorKind::Unknown,
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error codes. Compare them with [`Error::code`].
///
/// The SDK sets the upper-case codes itself when the API sends none; `HTTP_<status>` is used for any other answer
/// without a code. The API's own codes look like `Dna.DomainService:Domain:10007`; the full catalogue is at
/// <https://www.domainnameapi.com/docs/reference/errors.md>.
pub mod codes {
    /// The gateway redirected to its suspend page: the reseller ID is unknown, inactive or suspended in this
    /// environment.
    pub const INVALID_RESELLER: &str = "INVALID_RESELLER";
    /// The API key does not match the reseller. Production and OTE keys differ.
    pub const INVALID_API_KEY: &str = "INVALID_API_KEY";
    /// The `X-API-KEY` header was empty.
    pub const MISSING_API_KEY: &str = "MISSING_API_KEY";
    /// The `__reseller` header was empty.
    pub const MISSING_RESELLER_ID: &str = "MISSING_RESELLER_ID";
    /// Sub-reseller accounts cannot use the API.
    pub const SUBRESELLER_NOT_ALLOWED: &str = "SUBRESELLER_NOT_ALLOWED";
    /// Any other 401.
    pub const UNAUTHORIZED: &str = "UNAUTHORIZED";
    /// No reseller ID or API key was configured.
    pub const NO_CREDENTIALS: &str = "NO_CREDENTIALS";
    /// 429 after the automatic retries.
    pub const RATE_LIMITED: &str = "RATE_LIMITED";
    /// The request failed on the network.
    pub const NETWORK: &str = "NETWORK";
    /// No answer within the timeout.
    pub const TIMEOUT: &str = "TIMEOUT";
    /// Blocked by read-only mode.
    pub const READ_ONLY: &str = "READ_ONLY";
    /// A request failed its checks before sending.
    pub const INVALID_INPUT: &str = "INVALID_INPUT";
    /// A client setting (environment, base URL, ...) is invalid.
    pub const INVALID_CONFIGURATION: &str = "INVALID_CONFIGURATION";
    /// A successful answer had no body where one was expected.
    pub const EMPTY_RESPONSE: &str = "EMPTY_RESPONSE";
    /// A successful answer could not be read.
    pub const INVALID_RESPONSE: &str = "INVALID_RESPONSE";

    /// Not found, or not yours.
    pub const DOMAIN_NOT_FOUND: &str = "Dna.DomainService:Domain:10007";
    /// The TLD has no WHOIS privacy.
    pub const PRIVACY_NOT_SUPPORTED: &str = "Dna.DomainService:Domain:10040";
    /// The domain has no URL forwarding.
    pub const FORWARDING_NOT_FOUND: &str = "Dna.DomainService:Domain:10061";
    /// Registered at the registry, not recorded yet.
    pub const REGISTERED_NOT_RECORDED: &str = "Dna.DomainService:Domain:10058";
    /// Renewed at the registry, not recorded yet.
    pub const RENEWED_NOT_RECORDED: &str = "Dna.DomainService:Domain:10059";
    /// Transfer started at the registry, not recorded yet.
    pub const TRANSFER_NOT_RECORDED: &str = "Dna.DomainService:Domain:10060";
    /// Sent to the certificate authority.
    pub const SSL_SENT_TO_CA: &str = "Dna.SslService:Ssl:10053";
    /// The certificate authority's answer is unknown.
    pub const SSL_OUTCOME_UNVERIFIED: &str = "Dna.SslService:Ssl:10054";
    /// There is no verification pending for the contact.
    pub const VERIFICATION_NOT_PENDING: &str = "Dna.DomainService:Contact:Verification:10005";
    /// The contact is already verified.
    pub const VERIFICATION_ALREADY_DONE: &str = "Dna.DomainService:Contact:Verification:10003";
    /// The balance is too low (domain operations).
    pub const INSUFFICIENT_BALANCE: &str = "Dna.DomainService:Deposit:10001";
    /// The balance is too low (deposit service).
    pub const DEPOSIT_INSUFFICIENT_BALANCE: &str = "Dna.DepositService:Deposit:10002";

    /// Codes meaning "the registry or certificate authority may already have done it".
    pub(crate) const OUTCOME_UNKNOWN: &[&str] =
        &[REGISTERED_NOT_RECORDED, RENEWED_NOT_RECORDED, TRANSFER_NOT_RECORDED, SSL_SENT_TO_CA, SSL_OUTCOME_UNVERIFIED];
}

/// The error of every failed request: API errors (non-2xx answers), network failures, timeouts, read-only blocks,
/// and requests that failed their checks before sending.
///
/// ```no_run
/// use domainnameapi::{Client, ErrorKind};
///
/// # async fn run(client: Client) -> Result<(), domainnameapi::Error> {
/// match client.domains().renew("example.com", 1).await {
///     Ok(result) => println!("{}: {}", result.success, result.message),
///     Err(e) if e.kind() == ErrorKind::Validation => println!("{}: {}", e.code(), e.message()),
///     Err(e) if e.is_outcome_unknown() => {
///         // The renewal may have run: check the domain before sending it again.
///         let info = client.domains().info("example.com").await?;
///         println!("expires {:?}", info.expiration_date);
///     }
///     Err(e) => return Err(e),
/// }
/// # Ok(()) }
/// ```
pub struct Error {
    inner: Box<Inner>,
}

struct Inner {
    kind: ErrorKind,
    code: String,
    message: String,
    status: Option<u16>,
    details: Option<String>,
    validation_errors: Vec<FieldError>,
    correlation_id: Option<String>,
    retry_after: Option<Duration>,
    response_body: Option<String>,
    operation: Option<String>,
    safety: Option<OperationSafety>,
    hint: Option<String>,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl Error {
    /// Creates an error with a kind, a code and a message, for example to return from a mock in your tests.
    pub fn new(kind: ErrorKind, code: impl Into<String>, message: impl Into<String>) -> Self {
        Error {
            inner: Box::new(Inner {
                kind,
                code: code.into(),
                message: message.into(),
                status: None,
                details: None,
                validation_errors: Vec::new(),
                correlation_id: None,
                retry_after: None,
                response_body: None,
                operation: None,
                safety: None,
                hint: None,
                source: None,
            }),
        }
    }

    /// An [`ErrorKind::InvalidInput`] error about one field.
    pub(crate) fn invalid_input(field: &str, message: impl Into<String>) -> Self {
        let message = message.into();
        Error::new(ErrorKind::InvalidInput, codes::INVALID_INPUT, message.clone())
            .with_validation_errors(vec![FieldError { field: field.to_string(), message }])
    }

    /// An [`ErrorKind::InvalidInput`] error about a client setting.
    pub(crate) fn configuration(message: impl Into<String>) -> Self {
        Error::new(ErrorKind::InvalidInput, codes::INVALID_CONFIGURATION, message)
    }

    /// Sets the HTTP status.
    pub fn with_status(mut self, status: u16) -> Self {
        self.inner.status = Some(status);
        self
    }

    /// Sets the field-level problems.
    pub fn with_validation_errors(mut self, errors: Vec<FieldError>) -> Self {
        self.inner.validation_errors = errors;
        self
    }

    /// Sets the correlation ID.
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        let id = id.into();
        self.inner.correlation_id = (!id.is_empty()).then_some(id);
        self
    }

    /// Sets the request that failed (`POST /api/v1/domains/renew`) and its safety class.
    pub fn with_operation(mut self, operation: impl Into<String>, safety: OperationSafety) -> Self {
        self.inner.operation = Some(operation.into());
        self.inner.safety = Some(safety);
        self
    }

    /// Sets the suggestion on what to do next.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.inner.hint = Some(hint.into());
        self
    }

    pub(crate) fn with_details(mut self, details: Option<String>) -> Self {
        self.inner.details = details;
        self
    }

    pub(crate) fn with_retry_after(mut self, retry_after: Option<Duration>) -> Self {
        self.inner.retry_after = retry_after;
        self
    }

    pub(crate) fn with_response_body(mut self, body: &str) -> Self {
        self.inner.response_body = (!body.is_empty()).then(|| body.to_string());
        self
    }

    pub(crate) fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.inner.source = Some(Box::new(source));
        self
    }

    pub(crate) fn fill_hint(mut self) -> Self {
        if self.inner.hint.is_none() {
            self.inner.hint = default_hint(&self);
        }
        self
    }

    /// Returns the classification of the failure.
    pub fn kind(&self) -> ErrorKind {
        self.inner.kind
    }

    /// Returns the API's error code when it sends one (for example `Dna.DomainService:Domain:10007`), otherwise one
    /// of the SDK's [`codes`] or `HTTP_<status>`. Branch on the code rather than on the message: messages are
    /// localised.
    pub fn code(&self) -> &str {
        &self.inner.code
    }

    /// Returns what happened. Boilerplate messages are replaced with the API's details.
    pub fn message(&self) -> &str {
        &self.inner.message
    }

    /// Returns the HTTP status, or `None` when no answer arrived.
    pub fn status(&self) -> Option<u16> {
        self.inner.status
    }

    /// Returns additional detail the API sent with the error.
    pub fn details(&self) -> Option<&str> {
        self.inner.details.as_deref()
    }

    /// Returns the field-level problems: the API's validation errors, or the checks that failed before sending.
    pub fn validation_errors(&self) -> &[FieldError] {
        &self.inner.validation_errors
    }

    /// Returns the gateway's correlation ID. Quote it when contacting support.
    pub fn correlation_id(&self) -> Option<&str> {
        self.inner.correlation_id.as_deref()
    }

    /// Returns how long the gateway asked to wait before retrying (rate limits).
    pub fn retry_after(&self) -> Option<Duration> {
        self.inner.retry_after
    }

    /// Returns the raw response body.
    pub fn response_body(&self) -> Option<&str> {
        self.inner.response_body.as_deref()
    }

    /// Returns the request that failed, for example `POST /api/v1/domains/renew`.
    pub fn operation(&self) -> Option<&str> {
        self.inner.operation.as_deref()
    }

    /// Returns what the failed request could do to your account.
    pub fn safety(&self) -> Option<OperationSafety> {
        self.inner.safety
    }

    /// Returns a short suggestion on what to do next.
    pub fn hint(&self) -> Option<&str> {
        self.inner.hint.as_deref()
    }

    /// Returns whether repeating the same request later can succeed: rate limits, network failures, timeouts and
    /// server errors other than 501, unless the API said the operation may already have happened.
    pub fn is_retryable(&self) -> bool {
        if codes::OUTCOME_UNKNOWN.contains(&self.code()) {
            return false;
        }
        match self.kind() {
            ErrorKind::RateLimited | ErrorKind::Network | ErrorKind::Timeout => true,
            ErrorKind::Server => self.status() != Some(501),
            _ => false,
        }
    }

    /// Returns whether the failed request may still have been carried out: a request that is not a read failed on
    /// the network or timed out, or the API said the registry or certificate authority may already have done it.
    /// Check the state (for example with `domains().info(...)` or the account transactions) before sending it
    /// again; repeating a registration or renewal blindly can charge twice.
    pub fn is_outcome_unknown(&self) -> bool {
        if codes::OUTCOME_UNKNOWN.contains(&self.code()) {
            return true;
        }
        matches!(self.kind(), ErrorKind::Network | ErrorKind::Timeout)
            && self.safety().is_some_and(|s| s != OperationSafety::Read)
    }

    /// Returns whether the resource does not exist (or is not yours).
    pub fn is_not_found(&self) -> bool {
        self.kind() == ErrorKind::NotFound
    }

    /// Returns whether the balance is too low for the operation.
    pub fn is_insufficient_balance(&self) -> bool {
        self.status() == Some(402)
            || self.code() == codes::INSUFFICIENT_BALANCE
            || self.code() == codes::DEPOSIT_INSUFFICIENT_BALANCE
    }
}

impl fmt::Display for Error {
    /// `[CODE] message (HTTP status)`, followed by the field-level problems.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code(), self.message())?;
        if let Some(status) = self.status() {
            write!(f, " (HTTP {status})")?;
        }
        for error in self.validation_errors() {
            write!(f, "; {error}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("Error");
        d.field("kind", &self.kind()).field("code", &self.code()).field("message", &self.message());
        if let Some(status) = self.status() {
            d.field("status", &status);
        }
        if !self.validation_errors().is_empty() {
            d.field("validation_errors", &self.validation_errors());
        }
        if let Some(id) = self.correlation_id() {
            d.field("correlation_id", &id);
        }
        if let Some(operation) = self.operation() {
            d.field("operation", &operation);
        }
        if let Some(hint) = self.hint() {
            d.field("hint", &hint);
        }
        d.finish()
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source.as_ref().map(|s| s.as_ref() as &(dyn std::error::Error + 'static))
    }
}

fn default_hint(e: &Error) -> Option<String> {
    let hint = match e.code() {
        codes::INVALID_RESELLER => {
            "The reseller ID is unknown, inactive or suspended in this environment. Check the \
            reseller ID (a UUID from the panel's API settings) and the environment (production or OTE)."
        }
        codes::INVALID_API_KEY => {
            "Production and OTE use different API keys. Make sure the key belongs to the client's environment."
        }
        codes::MISSING_API_KEY | codes::MISSING_RESELLER_ID | codes::NO_CREDENTIALS => {
            "Pass the reseller ID and API key to Client::new or ClientBuilder::credentials, or set DNA_RESELLER_ID \
             and DNA_API_KEY."
        }
        codes::SUBRESELLER_NOT_ALLOWED => {
            "Sub-reseller accounts cannot use the API. Use the main reseller's credentials."
        }
        codes::RATE_LIMITED => {
            "The gateway allows 5 requests/second per key (2/second for domain searches). Slow down, or keep \
             pacing on (the default)."
        }
        codes::READ_ONLY => {
            "The client is read-only (ClientBuilder::read_only, or DNA_READ_ONLY is set). Turn it off to send \
             changes."
        }
        code if codes::OUTCOME_UNKNOWN.contains(&code) => {
            "The operation may already have been carried out. Check the object's state before sending the request \
             again."
        }
        _ => match e.kind() {
            ErrorKind::NotFound => {
                if e.operation().is_some_and(|op| op.contains("/api/v1/ssls")) {
                    "Check the certificate ID and the environment: production and OTE hold different certificates. \
                     ssl().list(...) shows the certificates this account holds."
                } else {
                    "Check the spelling and the environment: production and OTE have separate data."
                }
            }
            ErrorKind::Validation | ErrorKind::InvalidInput if !e.validation_errors().is_empty() => {
                "Fix the listed fields and send the request again."
            }
            ErrorKind::Server => {
                return e.correlation_id().map(|id| {
                    format!("Server-side failure. If it persists, contact support and quote correlation ID {id}.")
                })
            }
            ErrorKind::Network | ErrorKind::Timeout => {
                if e.safety().is_some_and(|s| s != OperationSafety::Read) {
                    "The request may have reached the API. Check the current state before sending it again."
                } else {
                    "Check the network, proxy and firewall. Reads are retried automatically."
                }
            }
            _ => return None,
        },
    };
    Some(hint.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_follow_the_status() {
        assert_eq!(ErrorKind::from_status(401), ErrorKind::Auth);
        assert_eq!(ErrorKind::from_status(302), ErrorKind::Auth);
        assert_eq!(ErrorKind::from_status(403), ErrorKind::Forbidden);
        assert_eq!(ErrorKind::from_status(404), ErrorKind::NotFound);
        assert_eq!(ErrorKind::from_status(422), ErrorKind::Validation);
        assert_eq!(ErrorKind::from_status(409), ErrorKind::Conflict);
        assert_eq!(ErrorKind::from_status(429), ErrorKind::RateLimited);
        assert_eq!(ErrorKind::from_status(503), ErrorKind::Server);
        assert_eq!(ErrorKind::from_status(418), ErrorKind::Unknown);
        assert_eq!(ErrorKind::RateLimited.to_string(), "rate_limited");
    }

    #[test]
    fn retryable_and_outcome_unknown() {
        let server = Error::new(ErrorKind::Server, "HTTP_503", "x").with_status(503);
        assert!(server.is_retryable());
        assert!(!Error::new(ErrorKind::Server, "HTTP_501", "x").with_status(501).is_retryable());

        let network = Error::new(ErrorKind::Network, codes::NETWORK, "x")
            .with_operation("POST /api/v1/domains/renew", OperationSafety::Billable);
        assert!(network.is_retryable());
        assert!(network.is_outcome_unknown());

        let read = Error::new(ErrorKind::Timeout, codes::TIMEOUT, "x")
            .with_operation("GET /api/v1/domains/info", OperationSafety::Read);
        assert!(!read.is_outcome_unknown());

        let recorded = Error::new(ErrorKind::Server, codes::RENEWED_NOT_RECORDED, "x").with_status(500);
        assert!(recorded.is_outcome_unknown());
        assert!(!recorded.is_retryable());
        assert!(recorded.fill_hint().hint().unwrap().contains("already"));
    }

    #[test]
    fn display_shows_code_status_and_fields() {
        let e = Error::new(ErrorKind::Validation, "Dna.X:1", "Invalid input")
            .with_status(400)
            .with_validation_errors(vec![FieldError { field: "domainName".into(), message: "is required".into() }]);
        assert_eq!(e.to_string(), "[Dna.X:1] Invalid input (HTTP 400); domainName: is required");
        assert!(format!("{e:?}").contains("Validation"));
    }

    #[test]
    fn insufficient_balance() {
        assert!(Error::new(ErrorKind::Unknown, "HTTP_402", "x").with_status(402).is_insufficient_balance());
        assert!(Error::new(ErrorKind::Validation, codes::INSUFFICIENT_BALANCE, "x").is_insufficient_balance());
        assert!(!Error::new(ErrorKind::Validation, "other", "x").is_insufficient_balance());
    }
}
