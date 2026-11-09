//! The client: HTTP layer, error mapping, diagnostics and the domain,
//! account, transfer and contact methods (php-dna `DNARest` + facade).

use std::time::{Duration, Instant};

use serde_json::{json, Map, Value};

use crate::php::{self, get, get_path, obj};
use crate::telemetry::{CallContext, Telemetry};
use crate::transport::{HttpRequest, Transport, UreqTransport};
use crate::{
    DEFAULT_CONNECT_TIMEOUT, DEFAULT_ERRORS, DEFAULT_NAMESERVERS, DEFAULT_REASON, DEFAULT_TIMEOUT,
    RESULT_ERROR, RESULT_OK, URL_OTE, URL_PROD, USER_AGENT, ZONE_MIN_INTERVAL,
};

/// A failed gateway call: the HTTP status (or curl error number) and message.
#[derive(Debug, Clone)]
pub(crate) struct ApiError {
    pub code: i64,
    pub message: String,
}

type ApiResult = Result<Value, ApiError>;

/// Values accepted for the constructor's `test_mode` argument.
///
/// Panels store checkbox settings as strings, so besides `bool` the
/// constructor accepts strings: `"true"`, `"1"`, `"on"` and `"yes"`
/// (case-insensitive, trimmed) mean test mode, anything else means live
/// (php-dna `FILTER_VALIDATE_BOOLEAN`, BUG-10284).
pub trait TestModeFlag {
    /// Resolve to a boolean.
    fn into_test_mode(self) -> bool;
}

impl TestModeFlag for bool {
    fn into_test_mode(self) -> bool {
        self
    }
}

impl TestModeFlag for &str {
    fn into_test_mode(self) -> bool {
        matches!(
            php::trim(self).to_ascii_lowercase().as_str(),
            "1" | "true" | "on" | "yes"
        )
    }
}

impl TestModeFlag for String {
    fn into_test_mode(self) -> bool {
        self.as_str().into_test_mode()
    }
}

impl TestModeFlag for &String {
    fn into_test_mode(self) -> bool {
        self.as_str().into_test_mode()
    }
}

/// Optional arguments of [`DomainNameApi::register_with_contact_info_opts`].
/// `Default` gives php-dna's defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct RegisterOptions {
    /// Nameservers (list or map). Empty: the library defaults
    /// (`ns1/ns2.domainnameapi.com`).
    pub name_servers: Value,
    /// Registrar (theft protection) lock. Default `true`.
    pub epp_lock: bool,
    /// Privacy protection. Default `false`.
    pub privacy_lock: bool,
    /// TLD attributes (TRABIS fields for .tr). Default empty.
    pub additional_attributes: Value,
}

impl Default for RegisterOptions {
    fn default() -> Self {
        Self {
            name_servers: json!([]),
            epp_lock: true,
            privacy_lock: false,
            additional_attributes: json!({}),
        }
    }
}

/// DomainNameAPI client (php-dna `DomainNameAPI_PHPLibrary`).
///
/// Every network method returns a [`serde_json::Value`] with php-dna's
/// envelope and never panics on gateway or network failures.
pub struct DomainNameApi {
    reseller_id: String,
    api_key: String,
    service_url: String,
    ssl_verify: bool,
    transport: Box<dyn Transport>,
    pub(crate) telemetry: Telemetry,
    last_request: Value,
    last_response: Value,
    last_response_headers: Value,
    pub(crate) last_parsed_response: Value,
    last_function: String,
    start_at: f64,
    last_zone_request_at: Option<Instant>,
    zone_min_interval: Duration,
}

impl std::fmt::Debug for DomainNameApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomainNameApi")
            .field("reseller_id", &self.reseller_id)
            .field("api_key", &"***")
            .field("service_url", &self.service_url)
            .field("ssl_verify", &self.ssl_verify)
            .finish_non_exhaustive()
    }
}

/// Reason phrase for an HTTP status (fallback message for empty bodies).
fn http_status_reason(status: i64) -> &'static str {
    match status {
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        409 => "Conflict",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        s if s >= 500 => "Server Error",
        s if s >= 400 => "Client Error",
        _ => "Unexpected Response",
    }
}

/// `/Exception of type '[^']*' was thrown/i`.
fn is_abp_boilerplate(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    let needle = "exception of type '";
    let mut from = 0;
    while let Some(pos) = lower[from..].find(needle) {
        let start = from + pos + needle.len();
        if let Some(close) = lower[start..].find('\'') {
            if lower[start + close..].starts_with("' was thrown") {
                return true;
            }
        }
        from = from + pos + 1;
    }
    false
}

/// `preg_replace('/^Dna\s+/', '', $s)`.
fn strip_dna_prefix(s: &str) -> &str {
    if let Some(rest) = s.strip_prefix("Dna") {
        let trimmed = rest.trim_start_matches(|c: char| c.is_ascii_whitespace());
        if trimmed.len() < rest.len() {
            return trimmed;
        }
    }
    s
}

/// php-dna `setError()`.
pub(crate) fn set_error(code: Value, message: Value, details: Value) -> Value {
    let code_str = php::strval(&code);
    if let Some((c, m, d)) = DEFAULT_ERRORS.iter().find(|(c, _, _)| *c == code_str) {
        let message = if php::trim(&php::strval(&message)).is_empty() {
            json!(m)
        } else {
            message
        };
        let details = if php::trim(&php::strval(&details)).is_empty() {
            json!(d)
        } else {
            details
        };
        return obj([
            ("Code", json!(c)),
            ("Message", message),
            ("Details", details),
        ]);
    }
    obj([("Code", code), ("Message", message), ("Details", details)])
}

/// `setError()` with string arguments.
pub(crate) fn set_error_str(code: &str, message: &str, details: &str) -> Value {
    set_error(json!(code), json!(message), json!(details))
}

/// `['result' => 'ERROR', 'error' => setError(...)]` for local validation.
pub(crate) fn local_error(code: &str, why: &str) -> Value {
    obj([
        ("result", json!(RESULT_ERROR)),
        ("error", set_error_str(code, why, why)),
    ])
}

/// `formatErrorCode()`: HTTP / curl codes become `API_<n>`.
fn format_error_code(code: i64) -> String {
    format!("API_{code}")
}

/// Flatten the gateway's `validationErrors` into one readable string.
pub(crate) fn flatten_validation_errors(errors: &Value) -> String {
    let mut parts: Vec<String> = Vec::new();
    for ve in php::values(errors) {
        let part = match ve {
            Value::String(s) => {
                let t = php::trim(s);
                if t.is_empty() {
                    continue;
                }
                t.to_string()
            }
            Value::Array(_) | Value::Object(_) => {
                let message = get(ve, "message")
                    .map(|m| php::trim(&php::strval(m)).to_string())
                    .unwrap_or_default();
                let mut members = Vec::new();
                if let Some(list @ (Value::Array(_) | Value::Object(_))) = get(ve, "members") {
                    for member in php::values(list) {
                        let label = humanize_validation_member(&php::strval(member));
                        if !label.is_empty() {
                            members.push(label);
                        }
                    }
                }
                if !members.is_empty() && !message.is_empty() {
                    format!("{} - {}", members.join(", "), message)
                } else if !members.is_empty() {
                    members.join(", ")
                } else if !message.is_empty() {
                    message
                } else {
                    continue;
                }
            }
            _ => continue,
        };
        if !parts.contains(&part) {
            parts.push(part);
        }
    }
    parts.join(" | ")
}

/// `"$.administrativeContact.postalCode"` → `"Admin PostalCode"`.
pub(crate) fn humanize_validation_member(member: &str) -> String {
    let member = php::trim(member);
    if member.is_empty() {
        return String::new();
    }
    let member = member.trim_start_matches(['$', '.']);
    // drop array indexes: \[\d+\]
    let b = member.as_bytes();
    let mut cleaned = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'[' {
            let mut j = i + 1;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            if j > i + 1 && j < b.len() && b[j] == b']' {
                i = j + 1;
                continue;
            }
        }
        cleaned.push(b[i]);
        i += 1;
    }
    let cleaned = String::from_utf8_lossy(&cleaned).into_owned();
    let out: Vec<String> = cleaned
        .split(['.', '/'])
        .filter(|s| !s.is_empty())
        .map(|seg| match seg.to_ascii_lowercase().as_str() {
            "administrativecontact" | "administrative" => "Admin".to_string(),
            "technicalcontact" | "technical" => "Tech".to_string(),
            "registrantcontact" | "registrant" => "Registrant".to_string(),
            "billingcontact" | "billing" => "Billing".to_string(),
            _ => php::ucfirst(seg),
        })
        .collect();
    out.join(" ").trim().to_string()
}

/// Gateway DomainStatus enum → label.
fn domain_status_label(code: i64) -> Option<&'static str> {
    Some(match code {
        0 => "Active",
        1 => "WaitingForRegistration",
        2 => "WaitingForDocument",
        3 => "WaitingForIncomingTransfer",
        4 => "TransferredOut",
        7 => "PendingDelete",
        8 => "Deleted",
        9 => "ConfirmationEmailSend",
        11 => "WaitingForOutgoingTransfer",
        12 => "PendingHold",
        15 => "ModificationPending",
        18 => "InCase",
        19 => "PendingRestore",
        _ => return None,
    })
}

/// php-dna `normalizeDomainStatus()`.
pub(crate) fn normalize_domain_status(value: &Value) -> String {
    if php::is_numeric(value) {
        return domain_status_label(php::intval(value))
            .map(str::to_string)
            .unwrap_or_else(|| php::strval(value));
    }
    php::strval(value)
}

/// php-dna `formatApiDate()`.
pub(crate) fn format_api_date(value: Option<&Value>) -> String {
    match value {
        None => String::new(),
        Some(v) => {
            let s = php::strval(v);
            if s.is_empty() {
                return String::new();
            }
            php::strtotime(&s).map(php::format_date).unwrap_or_default()
        }
    }
}

/// php-dna `remainingDaysFromDate()`.
pub(crate) fn remaining_days_from_date(value: Option<&Value>) -> i64 {
    let Some(v) = value else { return 0 };
    let s = php::strval(v);
    if s.is_empty() {
        return 0;
    }
    match php::strtotime(&s) {
        None => 0,
        Some(ts) => (((ts - php::now()) as f64) / 86_400.0).ceil().max(0.0) as i64,
    }
}

/// php-dna `normalizeHostName()`.
pub(crate) fn normalize_host_name(host: &str) -> String {
    php::lower(php::trim(host).trim_end_matches('.'))
}

/// php-dna `normalizeHostNameList()`: strings only, normalised, no empties
/// or duplicates, order kept.
pub(crate) fn normalize_host_name_list(hosts: &Value) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for host in php::values(hosts) {
        if let Value::String(s) = host {
            let n = normalize_host_name(s);
            if !n.is_empty() && !out.contains(&n) {
                out.push(n);
            }
        }
    }
    out
}

fn ip_version(ip: &str) -> &'static str {
    if ip.parse::<std::net::Ipv4Addr>().is_ok() {
        "v4"
    } else {
        "v6"
    }
}

impl DomainNameApi {
    /// Creates a client.
    ///
    /// `reseller_id` is the Reseller ID (UUID) and `api_key` the API Key from
    /// the panel's API Settings (panel login credentials do not work). Both
    /// are trimmed. `test_mode` selects the OTE environment; it accepts a
    /// `bool` or a string (see [`TestModeFlag`]).
    pub fn new(reseller_id: &str, api_key: &str, test_mode: impl TestModeFlag) -> Self {
        let test_mode = test_mode.into_test_mode();
        Self {
            reseller_id: php::trim(reseller_id).to_string(),
            api_key: php::trim(api_key).to_string(),
            service_url: if test_mode { URL_OTE } else { URL_PROD }.to_string(),
            ssl_verify: true,
            transport: Box::new(UreqTransport::new()),
            telemetry: Telemetry::default(),
            last_request: json!([]),
            last_response: json!([]),
            last_response_headers: json!([]),
            last_parsed_response: json!([]),
            last_function: String::new(),
            start_at: php::microtime(),
            last_zone_request_at: None,
            zone_min_interval: Duration::from_secs_f64(ZONE_MIN_INTERVAL),
        }
    }

    // ------------------------------------------------------------------
    // Settings
    // ------------------------------------------------------------------

    /// Replace the HTTP transport (tests, proxies, custom clients).
    pub fn set_transport(&mut self, transport: Box<dyn Transport>) {
        self.transport = transport;
    }

    /// Turn TLS certificate verification on or off (on by default). Only
    /// switch it off on a host whose CA bundle cannot validate the gateway.
    pub fn set_ssl_verification(&mut self, enabled: bool) {
        self.ssl_verify = enabled;
    }

    /// Turn telemetry (error reports and performance samples) on or off.
    /// On by default.
    pub fn set_error_reporting_enabled(&mut self, enabled: bool) {
        self.telemetry.enabled = enabled;
    }

    /// Send only the minimal telemetry payload: error code and message,
    /// endpoint, timing, versions and reseller ID.
    pub fn set_telemetry_minimal(&mut self, minimal: bool) {
        self.telemetry.minimal = minimal;
    }

    /// Override the Sentry DSN used for telemetry (empty disables sending).
    pub fn set_telemetry_dsn(&mut self, dsn: &str) {
        self.telemetry.dsn = dsn.to_string();
    }

    /// Minimum gap between two `domains/zones*` calls (default one second;
    /// the gateway answers HTTP 429 to bursts). Tests may set it to zero.
    pub fn set_zone_min_interval(&mut self, interval: Duration) {
        self.zone_min_interval = interval;
    }

    // ------------------------------------------------------------------
    // Diagnostics
    // ------------------------------------------------------------------

    /// Last request: `{"url", "method", "payload"}`.
    pub fn get_request_data(&self) -> Value {
        self.last_request.clone()
    }

    /// Last decoded response body (or `{"raw_response": "<body>"}` when the
    /// body was not JSON).
    pub fn get_response_data(&self) -> Value {
        self.last_response.clone()
    }

    /// Transfer info of the last response: `{"url", "http_code",
    /// "total_time", "headers"}` (php-dna returns `curl_getinfo()`).
    pub fn get_response_headers(&self) -> Value {
        self.last_response_headers.clone()
    }

    /// Last parsed response: the decoded body on success, the error
    /// envelope (`Code`, `Message`, `Details`) on failure.
    pub fn get_parsed_response_data(&self) -> Value {
        self.last_parsed_response.clone()
    }

    /// Last low-level operation, e.g. `request:GET domains/info`.
    pub fn get_last_function(&self) -> String {
        self.last_function.clone()
    }

    /// Gateway base URL in use.
    pub fn get_service_url(&self) -> String {
        self.service_url.clone()
    }

    /// Override the gateway base URL.
    pub fn set_service_url(&mut self, url: &str) {
        self.service_url = url.to_string();
    }

    // ------------------------------------------------------------------
    // HTTP layer
    // ------------------------------------------------------------------

    fn pace_zone_request(&mut self, endpoint: &str) {
        if !endpoint
            .trim_start_matches('/')
            .starts_with("domains/zones")
        {
            return;
        }
        if let Some(last) = self.last_zone_request_at {
            let elapsed = last.elapsed();
            if elapsed < self.zone_min_interval {
                std::thread::sleep(self.zone_min_interval - elapsed);
            }
        }
        self.last_zone_request_at = Some(Instant::now());
    }

    fn call_context(&self) -> CallContext {
        CallContext {
            reseller_id: self.reseller_id.clone(),
            last_request: self.last_request.clone(),
            last_response: self.last_response.clone(),
            last_parsed_response: self.last_parsed_response.clone(),
            http_code: php::intval_or(get(&self.last_response_headers, "http_code"), 0),
            operation: self.last_function.clone(),
            service_url: self.service_url.clone(),
            elapsed: php::microtime() - self.start_at,
            minimal: self.telemetry.minimal,
        }
    }

    pub(crate) fn report_error(&mut self, message: &str, code: i64) {
        if !self.telemetry.enabled {
            return;
        }
        let ctx = self.call_context();
        self.telemetry.report_error(message, code, &ctx);
    }

    /// php-dna `DNARest::request()`.
    pub(crate) fn request(&mut self, method: &str, endpoint: &str, data: &Value) -> ApiResult {
        self.pace_zone_request(endpoint);
        self.start_at = php::microtime();
        let operation_endpoint = endpoint.split('?').next().unwrap_or("");
        self.last_function = format!("request:{method} {operation_endpoint}");

        let mut url = format!("{}/{}", self.service_url, endpoint.trim_start_matches('/'));
        self.last_request = obj([
            ("url", json!(url)),
            ("method", json!(method)),
            ("payload", data.clone()),
        ]);

        let mut body = None;
        if method == "GET" || method == "DELETE" {
            if php::truthy(data) {
                url.push('?');
                url.push_str(&php::http_build_query(data));
            }
        } else {
            body = Some(php::json_encode(data));
        }

        let request = HttpRequest {
            method: method.to_string(),
            url: url.clone(),
            headers: vec![
                ("Content-Type".into(), "application/json".into()),
                ("Accept".into(), "application/json".into()),
                ("X-API-KEY".into(), self.api_key.clone()),
                ("__reseller".into(), self.reseller_id.clone()),
                ("User-Agent".into(), USER_AGENT.into()),
            ],
            body,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT),
            connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
            verify_tls: self.ssl_verify,
        };

        let started = Instant::now();
        let result = self.transport.send(&request);
        let total_time = started.elapsed().as_secs_f64();

        let response = match result {
            Err(err) => {
                self.last_response_headers = obj([
                    ("url", json!(url)),
                    ("http_code", json!(0)),
                    ("total_time", json!(total_time)),
                    ("headers", json!({})),
                ]);
                self.last_response = obj([("raw_response", json!(false))]);
                let message = if err.code == 28 {
                    format!(
                        "Could not reach the DomainNameAPI service: the connection timed out. ({})",
                        err.message
                    )
                } else {
                    format!("Curl error during request: {}", err.message)
                };
                self.last_parsed_response =
                    set_error_str(&format!("CURL_{}", err.code), &message, &message);
                self.report_error(&message, err.code);
                return Err(ApiError {
                    code: err.code,
                    message,
                });
            }
            Ok(response) => response,
        };

        let status = i64::from(response.status);
        let raw_body = response.body;
        let mut headers = Map::new();
        for (k, v) in &response.headers {
            headers.insert(k.clone(), json!(v));
        }
        let content_type = response
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
            .map(|(_, v)| json!(v))
            .unwrap_or(Value::Null);
        self.last_response_headers = obj([
            ("url", json!(url)),
            ("content_type", content_type),
            ("http_code", json!(status)),
            ("total_time", json!(total_time)),
            ("headers", Value::Object(headers)),
        ]);

        let decoded: Value = serde_json::from_str(&raw_body).unwrap_or(Value::Null);
        self.last_response = if decoded.is_null() {
            obj([("raw_response", json!(raw_body))])
        } else {
            decoded.clone()
        };

        if (200..=299).contains(&status) {
            self.last_parsed_response = decoded.clone();
            let duration_ms = (php::microtime() - self.start_at) * 1000.0;
            if self.telemetry.enabled {
                let ctx = self.call_context();
                let start = self.start_at;
                self.telemetry.sample_success(duration_ms, start, &ctx);
            }
            return Ok(decoded);
        }

        let mut parsed = decoded;
        if status == 301 || status == 302 {
            parsed = json!({
                "message": "Invalid API credentials",
                "code": "CREDENTIALS",
                "details": "Authentication failed. Check your API key and reseller ID."
            });
        }
        let body_value = json!(raw_body);
        let error_message_value = get(&parsed, "message")
            .or_else(|| get_path(&parsed, &["error", "message"]))
            .cloned()
            .unwrap_or_else(|| body_value.clone());
        let error_code = get(&parsed, "code")
            .or_else(|| get_path(&parsed, &["error", "code"]))
            .cloned()
            .unwrap_or_else(|| json!(format!("HTTP_{status}")));
        let mut error_details = get(&parsed, "details")
            .or_else(|| get_path(&parsed, &["error", "details"]))
            .cloned()
            .unwrap_or_else(|| body_value.clone());

        let mut error_message = php::strval(&error_message_value);
        if php::trim(&error_message).is_empty() {
            error_message = format!("HTTP {status} - {}", http_status_reason(status));
            if php::trim(&php::strval(&error_details)).is_empty() {
                error_details = json!(error_message);
            }
        }

        let code_str = php::strval(&error_code);
        if is_abp_boilerplate(&error_message) || error_message == code_str {
            let details_str = php::strval(&error_details);
            let details_clean = php::trim(strip_dna_prefix(&details_str)).to_string();
            if !details_clean.is_empty()
                && details_clean != error_message
                && details_clean != code_str
                && !is_abp_boilerplate(&details_clean)
            {
                error_message = details_clean;
            } else {
                error_message = format!(
                    "The operation could not be completed by the registry (error: {code_str}). Please try again or contact support."
                );
            }
        }

        let validation = get_path(&parsed, &["error", "validationErrors"])
            .or_else(|| get(&parsed, "validationErrors"));
        if let Some(v) = validation {
            if php::is_array(v) && php::truthy(v) {
                let flattened = flatten_validation_errors(v);
                if !flattened.is_empty() {
                    error_details = json!(flattened);
                }
            }
        }

        self.last_parsed_response = set_error(error_code, json!(error_message), error_details);
        self.report_error(&error_message, status);
        Err(ApiError {
            code: status,
            message: error_message,
        })
    }

    /// The `catch (Exception $e)` block shared by every public method.
    pub(crate) fn error_envelope(&self, e: &ApiError, fallback: &str) -> Value {
        let mut code = format_error_code(e.code);
        if code.is_empty() {
            code = fallback.to_string();
        }
        let details = get(&self.last_parsed_response, "Details")
            .or_else(|| get(&self.last_response, "raw_response"))
            .cloned()
            .unwrap_or_else(|| json!(e.message));
        obj([
            ("result", json!(RESULT_ERROR)),
            ("error", set_error(json!(code), json!(e.message), details)),
        ])
    }

    // ------------------------------------------------------------------
    // Account
    // ------------------------------------------------------------------

    /// Account details with USD and TRY balances.
    pub fn get_reseller_details(&mut self) -> Value {
        match self.request("GET", "deposit/accounts/me", &json!([])) {
            Ok(response) => {
                if get(&response, "resellerId").is_some() {
                    let usd = php::strval(get(&response, "usdBalance").unwrap_or(&json!(0)));
                    let tl = php::strval(get(&response, "tryBalance").unwrap_or(&json!(0)));
                    obj([
                        ("result", json!(RESULT_OK)),
                        ("id", response["resellerId"].clone()),
                        ("active", json!(true)),
                        (
                            "name",
                            get(&response, "resellerName").cloned().unwrap_or(json!("")),
                        ),
                        ("balance", json!(usd)),
                        ("currency", json!("USD")),
                        ("symbol", json!("$")),
                        (
                            "balances",
                            json!([
                                {"balance": usd, "currency": "USD", "symbol": "$"},
                                {"balance": tl, "currency": "TL", "symbol": "TL"}
                            ]),
                        ),
                    ])
                } else {
                    obj([
                        ("result", json!(RESULT_ERROR)),
                        (
                            "error",
                            set_error_str(
                                "CREDENTIALS",
                                "Invalid response format",
                                "Response does not contain required fields",
                            ),
                        ),
                    ])
                }
            }
            Err(e) => self.error_envelope(&e, "RESELLER_DETAILS"),
        }
    }

    /// Balance in one currency (`USD`, `TRY`, `EUR`, `GBP`; php-dna default
    /// `USD`, see [`crate::DEFAULT_CURRENCY`]). Returns the
    /// `OperationResult` envelope, not `result`/`data`.
    pub fn get_current_balance(&mut self, currency: &str) -> Value {
        let name = php::upper(currency);
        let (id, label, symbol) = match name.as_str() {
            "USD" => (2, "USD".to_string(), "$"),
            "TRY" => (1, "TL".to_string(), "TL"),
            "EUR" => (3, "EUR".to_string(), "€"),
            "GBP" => (4, "GBP".to_string(), "£"),
            _ => (0, name.clone(), ""),
        };
        match self.request("GET", "deposit/accounts/me", &json!({"currency": name})) {
            Ok(response) => {
                let key = format!("{}Balance", php::lower(currency));
                let balance = php::floatval(get(&response, &key).unwrap_or(&json!(0)));
                obj([
                    ("ErrorCode", json!(0)),
                    ("OperationMessage", json!("Command completed succesfully.")),
                    ("OperationResult", json!("SUCCESS")),
                    ("Balance", json!(php::number_format(balance, 2))),
                    ("CurrencyId", json!(id)),
                    ("CurrencyInfo", Value::Null),
                    ("CurrencyName", json!(label)),
                    ("CurrencySymbol", json!(symbol)),
                ])
            }
            Err(e) => {
                let code = format_error_code(e.code);
                obj([
                    (
                        "ErrorCode",
                        json!(if code.is_empty() {
                            "BALANCE".to_string()
                        } else {
                            code
                        }),
                    ),
                    ("OperationMessage", json!(e.message)),
                    ("OperationResult", json!("FAILED")),
                    ("Balance", json!("0.00")),
                    ("CurrencyId", json!(id)),
                    ("CurrencyInfo", Value::Null),
                    ("CurrencyName", json!(label)),
                    ("CurrencySymbol", json!(symbol)),
                ])
            }
        }
    }

    // ------------------------------------------------------------------
    // Search and lists
    // ------------------------------------------------------------------

    /// Availability of every `domain × extension` pair. On success returns a
    /// bare list of results (php-dna shape); on failure the error envelope.
    /// `command` is php-dna's `$command` (default `"create"`).
    pub fn check_availability<D, E>(
        &mut self,
        domains: D,
        extensions: E,
        period: i64,
        command: &str,
    ) -> Value
    where
        D: IntoIterator,
        D::Item: AsRef<str>,
        E: IntoIterator,
        E::Item: AsRef<str>,
    {
        let extensions: Vec<String> = extensions
            .into_iter()
            .map(|e| e.as_ref().to_string())
            .collect();
        let mut queries = Vec::new();
        for domain in domains {
            for tld in &extensions {
                queries.push(json!({"domainName": format!("{}.{}", domain.as_ref(), tld.trim_start_matches('.'))}));
            }
        }
        if queries.is_empty() {
            return obj([
                ("result", json!(RESULT_ERROR)),
                (
                    "error",
                    set_error_str(
                        "AVAILABILITY",
                        "Domain names not found.",
                        "No domain names provided for availability check",
                    ),
                ),
            ]);
        }
        let response = match self.request("POST", "domains/bulk-search", &Value::Array(queries)) {
            Ok(r) => r,
            Err(e) => return self.error_envelope(&e, "AVAILABILITY"),
        };
        let items = get(&response, "infos").unwrap_or(&response);
        let mut out = Vec::new();
        for item in php::values(items) {
            if !php::is_array(item) {
                continue;
            }
            let tld = match get(item, "tld") {
                Some(t) => php::lower(&php::strval(t)),
                None => {
                    let name = php::strval_or(get(item, "domainName"), "");
                    php::lower(name.rfind('.').map(|i| &name[i + 1..]).unwrap_or(""))
                }
            };
            let raw = php::strval_or(get(item, "domainName"), "");
            let domain_name = php::lower(&raw).replace(&format!(".{tld}"), "");
            let status = php::lower(&php::strval_or(get(item, "status"), ""));
            let available = matches!(status.as_str(), "available" | "1" | "true");
            out.push(obj([
                ("TLD", json!(tld)),
                ("DomainName", json!(domain_name)),
                (
                    "Status",
                    json!(if available {
                        "available"
                    } else {
                        "notavailable"
                    }),
                ),
                ("Command", json!(command)),
                (
                    "Period",
                    get(item, "period").cloned().unwrap_or(json!(period)),
                ),
                (
                    "IsFee",
                    get(item, "isPremium").cloned().unwrap_or(json!(false)),
                ),
                (
                    "Price",
                    get(item, "price")
                        .map(|p| json!(php::number_format(php::floatval(p), 4)))
                        .unwrap_or(Value::Null),
                ),
                (
                    "Currency",
                    get(item, "currency").cloned().unwrap_or(Value::Null),
                ),
                (
                    "Reason",
                    get(item, "reason").cloned().unwrap_or(Value::Null),
                ),
            ]));
        }
        Value::Array(out)
    }

    /// Domains in the account. `extra_parameters` is a map of filters
    /// (`PageSize`, `PageNumber` (0-based), `OrderColumn`, `OrderDirection`,
    /// `DomainName`, ... or the gateway's `MaxResultCount`/`SkipCount`/
    /// `Sorting`). Pass `&json!({})` for php-dna's default.
    pub fn get_list(&mut self, extra_parameters: &Value) -> Value {
        let mut extra = match extra_parameters {
            Value::Object(map) => map.clone(),
            Value::Array(list) => list
                .iter()
                .enumerate()
                .map(|(i, v)| (i.to_string(), v.clone()))
                .collect(),
            _ => Map::new(),
        };
        let page_size = extra.get("PageSize").filter(|v| !v.is_null()).cloned();
        let page_number = extra.get("PageNumber").filter(|v| !v.is_null()).cloned();
        if let Some(ps) = &page_size {
            extra.insert("MaxResultCount".into(), json!(php::intval(ps)));
        }
        if let Some(pn) = &page_number {
            let size = page_size.as_ref().map(php::intval).unwrap_or(200);
            extra.insert("SkipCount".into(), json!(php::intval(pn) * size));
        }
        if let Some(col) = extra.get("OrderColumn").filter(|v| !v.is_null()).cloned() {
            let dir = php::upper(&php::strval_or(
                extra.get("OrderDirection").filter(|v| !v.is_null()),
                "ASC",
            ));
            let dir = if dir == "DESC" { "DESC" } else { "ASC" };
            extra.insert(
                "Sorting".into(),
                json!(format!("{} {dir}", php::strval(&col))),
            );
        }
        for k in ["PageNumber", "PageSize", "OrderColumn", "OrderDirection"] {
            extra.shift_remove(k);
        }
        let mut params = Map::new();
        params.insert("MaxResultCount".into(), json!(200));
        params.insert("SkipCount".into(), json!(0));
        for (k, v) in extra {
            params.insert(k, v);
        }

        let response = match self.request("GET", "domains", &Value::Object(params)) {
            Ok(r) => r,
            Err(e) => return self.error_envelope(&e, "DOMAIN_LIST"),
        };
        let domains: Vec<Value> = match get(&response, "items") {
            Some(items) => php::values(items).into_iter().map(list_item).collect(),
            None => Vec::new(),
        };
        obj([
            ("data", obj([("Domains", Value::Array(domains))])),
            ("result", json!(RESULT_OK)),
            (
                "TotalCount",
                json!(php::intval_or(get(&response, "totalCount"), 0)),
            ),
        ])
    }

    /// TLD list with pricing. php-dna default `count` is 20
    /// ([`crate::DEFAULT_TLD_COUNT`]).
    pub fn get_tld_list(&mut self, count: i64) -> Value {
        let response = match self.request(
            "GET",
            "products/tlds",
            &json!({"MaxResultCount": count, "SkipCount": 0}),
        ) {
            Ok(r) => r,
            Err(e) => return self.error_envelope(&e, "TLD_LIST"),
        };
        let mut data = Vec::new();
        let mut id = 1;
        if let Some(items @ (Value::Array(_) | Value::Object(_))) = get(&response, "items") {
            for tld in php::values(items) {
                data.push(tld_entry(tld, id));
                id += 1;
            }
        }
        obj([("data", Value::Array(data)), ("result", json!(RESULT_OK))])
    }

    // ------------------------------------------------------------------
    // Domain
    // ------------------------------------------------------------------

    /// Domain details.
    pub fn get_details(&mut self, domain_name: &str) -> Value {
        match self.request("GET", "domains/info", &json!({"DomainName": domain_name})) {
            Ok(r) => parse_domain_info(&r),
            Err(e) => self.error_envelope(&e, "DOMAIN_DETAILS"),
        }
    }

    /// Re-read the domain from the registry (same result as `get_details`).
    pub fn sync_from_registry(&mut self, domain_name: &str) -> Value {
        self.get_details(domain_name)
    }

    /// Replace the nameservers. `name_servers` is a list or a map (values
    /// are used), e.g. `&json!(["ns1.example.com", "ns2.example.com"])`.
    pub fn modify_name_server(&mut self, domain_name: &str, name_servers: &Value) -> Value {
        let payload = json!({
            "domainName": domain_name,
            "nameServers": normalize_host_name_list(name_servers),
        });
        match self.request("PUT", "domains/dns/name-server", &payload) {
            Ok(r) => obj([
                ("result", json!(RESULT_OK)),
                (
                    "data",
                    obj([(
                        "NameServers",
                        get(&r, "nameServers")
                            .cloned()
                            .unwrap_or_else(|| name_servers.clone()),
                    )]),
                ),
            ]),
            Err(e) => self.error_envelope(&e, "MODIFY_NS"),
        }
    }

    /// Turn the registrar (theft protection) lock on.
    pub fn enable_theft_protection_lock(&mut self, domain_name: &str) -> Value {
        match self.request("POST", "domains/lock", &json!({"domainName": domain_name})) {
            Ok(_) => obj([
                ("result", json!(RESULT_OK)),
                ("data", json!({"LockStatus": true})),
            ]),
            Err(e) => self.error_envelope(&e, "ENABLE_LOCK"),
        }
    }

    /// Turn the registrar (theft protection) lock off.
    pub fn disable_theft_protection_lock(&mut self, domain_name: &str) -> Value {
        match self.request(
            "POST",
            "domains/unlock",
            &json!({"domainName": domain_name}),
        ) {
            Ok(_) => obj([
                ("result", json!(RESULT_OK)),
                ("data", json!({"LockStatus": false})),
            ]),
            Err(e) => self.error_envelope(&e, "DISABLE_LOCK"),
        }
    }

    /// Turn privacy protection on or off. `reason` is accepted for parity
    /// with php-dna (default [`crate::DEFAULT_REASON`]) and not sent.
    pub fn modify_privacy_protection_status(
        &mut self,
        domain_name: &str,
        status: bool,
        reason: &str,
    ) -> Value {
        let _reason = if reason.is_empty() || reason == "0" {
            DEFAULT_REASON
        } else {
            reason
        };
        let payload = json!({"domainName": domain_name, "privacyStatus": status});
        match self.request("POST", "domains/privacy", &payload) {
            Ok(_) => obj([
                ("data", json!({"PrivacyProtectionStatus": status})),
                ("result", json!(RESULT_OK)),
            ]),
            Err(e) => self.error_envelope(&e, "MODIFY_PRIVACY"),
        }
    }

    /// Renew for `period` years.
    pub fn renew(&mut self, domain_name: &str, period: i64) -> Value {
        let payload = json!({"domainName": domain_name, "period": period});
        match self.request("POST", "domains/renew", &payload) {
            Ok(r) => {
                if php::truthy(get(&r, "expirationDate").unwrap_or(&json!(false))) {
                    obj([
                        ("result", json!(RESULT_OK)),
                        (
                            "data",
                            obj([(
                                "ExpirationDate",
                                json!(format_api_date(get(&r, "expirationDate"))),
                            )]),
                        ),
                    ])
                } else {
                    let (_, _, description) = DEFAULT_ERRORS[12];
                    self.report_error(&format!("[DOMAIN_RENEW] {description}"), 0);
                    obj([
                        ("result", json!(RESULT_ERROR)),
                        ("error", set_error_str("DOMAIN_RENEW", "", "")),
                    ])
                }
            }
            Err(e) => self.error_envelope(&e, "RENEW_DOMAIN"),
        }
    }

    /// Register a domain with full contact information (php-dna argument
    /// order). `contacts` maps `Administrative`, `Billing`, `Technical`,
    /// `Registrant` to contact maps; `name_servers` may be empty for the
    /// defaults; `additional_attributes` carries TRABIS fields for `.tr`.
    #[allow(clippy::too_many_arguments)]
    pub fn register_with_contact_info(
        &mut self,
        domain_name: &str,
        period: i64,
        contacts: &Value,
        name_servers: &Value,
        epp_lock: bool,
        privacy_lock: bool,
        additional_attributes: &Value,
    ) -> Value {
        let payload_contacts: Vec<Value> = php::entries(contacts)
            .into_iter()
            .map(|(kind, details)| {
                crate::contacts::parse_contact(details, &php::ucfirst(&php::lower(&kind)))
            })
            .collect();
        let mut ns = normalize_host_name_list(name_servers);
        if ns.is_empty() {
            ns = DEFAULT_NAMESERVERS.iter().map(|s| s.to_string()).collect();
        }
        let mut attrs = php::to_array(additional_attributes);
        if php::lower(domain_name).ends_with(".tr") {
            attrs = crate::contacts::normalize_tr_attributes(&attrs);
        }
        let attrs = crate::contacts::stringify_attributes(&attrs);
        let payload = obj([
            ("domainName", json!(domain_name)),
            ("period", json!(period)),
            ("nameServers", json!(ns)),
            ("isLocked", json!(epp_lock)),
            ("privacyEnabled", json!(privacy_lock)),
            ("contacts", Value::Array(payload_contacts)),
            ("tldAttributes", Value::Object(attrs)),
        ]);
        match self.request("POST", "domains/register-with-contacts", &payload) {
            Ok(r) => parse_domain_info(&r),
            Err(e) => self.error_envelope(&e, "REGISTER_DOMAIN"),
        }
    }

    /// [`register_with_contact_info`](Self::register_with_contact_info) with
    /// the optional arguments in a [`RegisterOptions`]
    /// (`RegisterOptions::default()` = php-dna defaults).
    pub fn register_with_contact_info_opts(
        &mut self,
        domain_name: &str,
        period: i64,
        contacts: &Value,
        options: &RegisterOptions,
    ) -> Value {
        self.register_with_contact_info(
            domain_name,
            period,
            contacts,
            &options.name_servers,
            options.epp_lock,
            options.privacy_lock,
            &options.additional_attributes,
        )
    }

    // ------------------------------------------------------------------
    // Child nameservers
    // ------------------------------------------------------------------

    /// Create a child nameserver (glue record).
    pub fn add_child_name_server(
        &mut self,
        domain_name: &str,
        name_server: &str,
        ip_address: &str,
    ) -> Value {
        let payload = json!({
            "domainName": domain_name,
            "hostName": normalize_host_name(name_server),
            "ipAddresses": [{"ipAddress": ip_address, "ipVersion": ip_version(ip_address)}],
        });
        match self.request("POST", "domains/dns/host", &payload) {
            Ok(_) => obj([
                ("result", json!(RESULT_OK)),
                (
                    "data",
                    json!({"NameServer": name_server, "IPAdresses": [ip_address]}),
                ),
            ]),
            Err(e) => self.error_envelope(&e, "ADD_CHILD_NS"),
        }
    }

    /// Delete a child nameserver.
    pub fn delete_child_name_server(&mut self, domain_name: &str, name_server: &str) -> Value {
        let payload =
            json!({"domainName": domain_name, "hostName": normalize_host_name(name_server)});
        match self.request("DELETE", "domains/dns/host", &payload) {
            Ok(_) => obj([
                ("result", json!(RESULT_OK)),
                ("data", json!({"NameServer": name_server})),
            ]),
            Err(e) => self.error_envelope(&e, "DELETE_CHILD_NS"),
        }
    }

    /// Change the IP address of a child nameserver.
    pub fn modify_child_name_server(
        &mut self,
        domain_name: &str,
        name_server: &str,
        ip_address: &str,
    ) -> Value {
        let host = normalize_host_name(name_server);
        let payload = json!({
            "domainName": domain_name,
            "hostName": host,
            "newHostName": host,
            "ipAddresses": [{"ipAddress": ip_address, "ipVersion": ip_version(ip_address)}],
        });
        match self.request("PUT", "domains/dns/host", &payload) {
            Ok(r) => obj([
                ("result", json!(RESULT_OK)),
                (
                    "data",
                    obj([
                        (
                            "NameServer",
                            get(&r, "hostName").cloned().unwrap_or(json!(host)),
                        ),
                        (
                            "IPAdresses",
                            get(&r, "ipAddresses")
                                .cloned()
                                .unwrap_or(json!([ip_address])),
                        ),
                    ]),
                ),
            ]),
            Err(e) => self.error_envelope(&e, "MODIFY_CHILD_NS"),
        }
    }

    // ------------------------------------------------------------------
    // Contacts
    // ------------------------------------------------------------------

    /// The domain's four contacts.
    pub fn get_contacts(&mut self, domain_name: &str) -> Value {
        let info = match self.request("GET", "domains/info", &json!({"DomainName": domain_name})) {
            Ok(r) => r,
            Err(e) => return self.error_envelope(&e, "GET_CONTACTS"),
        };
        let mut contacts = Map::new();
        if let Some(list @ (Value::Array(_) | Value::Object(_))) = get(&info, "contacts") {
            for item in php::values(list) {
                let api_type = php::strval_or(get(item, "contactType"), "");
                let key = match api_type.as_str() {
                    "Admin" => "Administrative".to_string(),
                    "Billing" => "Billing".to_string(),
                    "Tech" => "Technical".to_string(),
                    "Registrant" => "Registrant".to_string(),
                    other => php::ucfirst(&php::lower(other)),
                };
                let mut merged = match item {
                    Value::Object(m) => m.clone(),
                    _ => Map::new(),
                };
                merged.insert("type".into(), json!(key));
                contacts.insert(
                    key,
                    crate::contacts::parse_contact_info(&Value::Object(merged)),
                );
            }
        }
        let mut ordered = Map::new();
        for key in ["Administrative", "Billing", "Registrant", "Technical"] {
            if let Some(c) = contacts.get(key) {
                ordered.insert(key.to_string(), c.clone());
            }
        }
        obj([
            ("data", obj([("contacts", php::assoc_or_empty(ordered))])),
            ("result", json!(RESULT_OK)),
        ])
    }

    /// Update the domain's contacts. `contacts` maps contact types to
    /// contact maps (flat or nested shape).
    pub fn save_contacts(&mut self, domain_name: &str, contacts: &Value) -> Value {
        let payload_contacts: Vec<Value> = php::entries(contacts)
            .into_iter()
            .map(|(kind, details)| {
                crate::contacts::parse_contact(details, &php::ucfirst(&php::lower(&kind)))
            })
            .collect();
        let payload = json!({"domainName": domain_name, "contacts": payload_contacts});
        match self.request("PUT", "domains/contacts/update", &payload) {
            Ok(r) => {
                let mut parsed = Map::new();
                if let Some(list @ (Value::Array(_) | Value::Object(_))) = get(&r, "contacts") {
                    for contact in php::values(list) {
                        let key =
                            php::ucfirst(&php::lower(&php::strval_or(get(contact, "type"), "")));
                        parsed.insert(key, crate::contacts::parse_contact_info(contact));
                    }
                }
                obj([
                    ("result", json!(RESULT_OK)),
                    ("data", obj([("contacts", php::assoc_or_empty(parsed))])),
                ])
            }
            Err(e) => self.error_envelope(&e, "SAVE_CONTACTS"),
        }
    }

    // ------------------------------------------------------------------
    // Transfers
    // ------------------------------------------------------------------

    /// Start an incoming transfer.
    ///
    /// Like php-dna's facade (`DomainNameAPI_PHPLibrary::transfer`), this
    /// sends no contacts. Use
    /// [`transfer_with_contacts`](Self::transfer_with_contacts) to send them.
    pub fn transfer(&mut self, domain_name: &str, epp_code: &str, period: i64) -> Value {
        self.transfer_with_contacts(domain_name, epp_code, period, &json!([]))
    }

    /// Start an incoming transfer with contacts (php-dna `DNARest::transfer`
    /// with its fourth argument).
    pub fn transfer_with_contacts(
        &mut self,
        domain_name: &str,
        epp_code: &str,
        period: i64,
        contacts: &Value,
    ) -> Value {
        let mut payload_contacts = Vec::new();
        if php::truthy(contacts) {
            for (kind, details) in php::entries(contacts) {
                payload_contacts.push(crate::contacts::parse_contact(
                    details,
                    &php::ucfirst(&php::lower(&kind)),
                ));
            }
        }
        let payload = json!({
            "domainName": domain_name,
            "authCode": epp_code,
            "period": period,
            "contacts": payload_contacts,
        });
        match self.request("POST", "domains/transfer", &payload) {
            Ok(r) => parse_domain_info(&r),
            Err(e) => self.error_envelope(&e, "TRANSFER_DOMAIN"),
        }
    }

    fn transfer_action(
        &mut self,
        domain_name: &str,
        path: &str,
        default_status: &str,
        fallback: &str,
    ) -> Value {
        match self.request("POST", path, &json!({"domainName": domain_name})) {
            Ok(r) => obj([
                ("result", json!(RESULT_OK)),
                (
                    "data",
                    obj([
                        ("DomainName", json!(domain_name)),
                        (
                            "Status",
                            get(&r, "status").cloned().unwrap_or(json!(default_status)),
                        ),
                    ]),
                ),
            ]),
            Err(e) => self.error_envelope(&e, fallback),
        }
    }

    /// Cancel a pending incoming transfer.
    pub fn cancel_transfer(&mut self, domain_name: &str) -> Value {
        self.transfer_action(
            domain_name,
            "domains/transfers/cancel",
            "Cancelled",
            "CANCEL_TRANSFER",
        )
    }

    /// Approve a pending outgoing transfer.
    pub fn approve_transfer(&mut self, domain_name: &str) -> Value {
        self.transfer_action(
            domain_name,
            "domains/transfers/approve",
            "Approved",
            "APPROVE_TRANSFER",
        )
    }

    /// Reject a pending outgoing transfer.
    pub fn reject_transfer(&mut self, domain_name: &str) -> Value {
        self.transfer_action(
            domain_name,
            "domains/transfers/reject",
            "Rejected",
            "REJECT_TRANSFER",
        )
    }

    /// Check whether the domain can be transferred with this auth code.
    /// `result` is OK only when the registry reports it transferable.
    pub fn check_transfer(&mut self, domain_name: &str, auth_code: &str) -> Value {
        let payload = json!({"domainName": domain_name, "authCode": auth_code});
        let r = match self.request("POST", "domains/transfers/check", &payload) {
            Ok(r) => r,
            Err(e) => return self.error_envelope(&e, "CHECK_TRANSFER"),
        };
        let flag = |k: &str| json!(!php::empty(get(&r, k)));
        let available = !php::empty(get(&r, "transferAvailabilityStatus"));
        let mut envelope = obj([
            (
                "result",
                json!(if available { RESULT_OK } else { RESULT_ERROR }),
            ),
            (
                "data",
                obj([
                    ("TransferAvailabilityStatus", json!(available)),
                    ("AuthCodeIsRequired", flag("authCodeIsRequired")),
                    ("AuthCodeIsValid", flag("authCodeIsValid")),
                    ("UserTransferRequired", flag("userTransferRequired")),
                    ("TransferLock", flag("transferLock")),
                    ("Message", json!(php::strval_or(get(&r, "message"), ""))),
                    (
                        "MessageKey",
                        json!(php::strval_or(get(&r, "messageKey"), "")),
                    ),
                    (
                        "Additional",
                        get(&r, "additionalAttributes")
                            .map(php::to_array)
                            .unwrap_or(json!([])),
                    ),
                ]),
            ),
        ]);
        if !available {
            let message = get(&r, "message")
                .cloned()
                .unwrap_or(json!("Domain is not transferable"));
            let details = get(&r, "messageKey")
                .or_else(|| get(&r, "message"))
                .cloned()
                .unwrap_or(json!(""));
            envelope["error"] = set_error(json!("TRANSFER_NOT_AVAILABLE"), message, details);
        }
        envelope
    }

    // ------------------------------------------------------------------
    // Utilities (no network)
    // ------------------------------------------------------------------

    /// Whether the domain ends in `.tr`.
    pub fn is_tr_tld(&self, domain: &str) -> bool {
        let b = domain.as_bytes();
        let tail = if b.len() >= 3 { &b[b.len() - 3..] } else { b };
        tail.eq_ignore_ascii_case(b".tr")
    }

    /// Complete and normalise a contact the way registration does (names
    /// sanitised, placeholders for missing fields, phone split into number
    /// and country code, country as ISO code).
    pub fn validate_contact(&self, contact: &Value) -> Value {
        crate::contacts::validate_contact(contact)
    }
}

/// One `domains` listing item → php-dna's domain shape.
fn list_item(item: &Value) -> Value {
    let status_code = get(item, "statusCode");
    let status_text = get(item, "statusText");
    let usable = |v: Option<&Value>| match v {
        Some(v) => v.as_str() != Some("") && !php::is_numeric(v),
        None => false,
    };
    let status = if usable(status_code) {
        php::strval(status_code.unwrap_or(&Value::Null))
    } else if usable(status_text) {
        php::strval(status_text.unwrap_or(&Value::Null))
    } else {
        normalize_domain_status(get(item, "status").unwrap_or(&json!("")))
    };
    let empty_contact = || json!({"ID": ""});
    obj([
        ("ID", json!(php::intval_or(get(item, "id"), 0))),
        ("Status", json!(status)),
        (
            "DomainName",
            get(item, "domainName").cloned().unwrap_or(json!("")),
        ),
        (
            "AuthCode",
            get(item, "authCode").cloned().unwrap_or(json!("")),
        ),
        (
            "LockStatus",
            json!(if php::empty(get(item, "lockStatus")) {
                "false"
            } else {
                "true"
            }),
        ),
        (
            "PrivacyProtectionStatus",
            json!(if php::empty(get(item, "privacyProtectionStatus")) {
                "false"
            } else {
                "true"
            }),
        ),
        (
            "IsChildNameServer",
            json!(if php::empty(get(item, "hosts")) {
                "false"
            } else {
                "true"
            }),
        ),
        (
            "Contacts",
            obj([
                ("Billing", empty_contact()),
                ("Technical", empty_contact()),
                ("Administrative", empty_contact()),
                ("Registrant", empty_contact()),
            ]),
        ),
        (
            "Dates",
            obj([
                ("Start", json!(format_api_date(get(item, "startDate")))),
                (
                    "Expiration",
                    json!(format_api_date(get(item, "expirationDate"))),
                ),
                (
                    "RemainingDays",
                    json!(php::intval_or(get(item, "remainingDay"), 0)),
                ),
            ]),
        ),
        (
            "NameServers",
            get(item, "nameServers").cloned().unwrap_or(json!([])),
        ),
        ("Additional", json!([])),
        ("ChildNameServers", json!([])),
    ])
}

/// One `products/tlds` item → php-dna's TLD shape.
fn tld_entry(tld: &Value, id: i64) -> Value {
    const PRICE_TYPES: [(&str, &str); 6] = [
        ("register", "registration"),
        ("renew", "renew"),
        ("transfer", "transfer"),
        ("restore", "restore"),
        ("refund", "refund"),
        ("backorder", "backorder"),
    ];
    let mut pricing: Map<String, Value> = Map::new();
    let mut currencies: Map<String, Value> = Map::new();
    let mut periods: Vec<(String, Vec<(i64, String)>)> = Vec::new();

    let mut put = |out_type: &str, info: &Value, currencies: &mut Map<String, Value>| {
        let mut period = php::intval_or(get(info, "period"), 1);
        if period < 1 {
            period = 1;
        }
        let price = get(info, "price")
            .map(|p| php::number_format(php::floatval(p), 4))
            .unwrap_or_else(|| "0.0000".to_string());
        let entry = match periods.iter_mut().find(|(t, _)| t == out_type) {
            Some(e) => e,
            None => {
                periods.push((out_type.to_string(), Vec::new()));
                periods.last_mut().expect("just pushed")
            }
        };
        match entry.1.iter_mut().find(|(p, _)| *p == period) {
            Some(slot) => slot.1 = price,
            None => entry.1.push((period, price)),
        }
        currencies.insert(
            out_type.to_string(),
            get(info, "currency").cloned().unwrap_or(json!("")),
        );
    };

    if let Some(first @ (Value::Array(_) | Value::Object(_))) = get_path(tld, &["prices", "0"]) {
        for (api_type, out_type) in PRICE_TYPES {
            let Some(api_value) = get(first, api_type) else {
                continue;
            };
            match api_value {
                Value::Array(list) if get(api_value, "0").is_some() => {
                    for info in list {
                        if php::is_array(info) {
                            put(out_type, info, &mut currencies);
                        }
                    }
                }
                Value::Array(_) | Value::Object(_) => put(out_type, api_value, &mut currencies),
                _ => {}
            }
        }
    }
    for (out_type, mut list) in periods {
        list.sort_by_key(|(p, _)| *p);
        let mut map = Map::new();
        for (p, price) in list {
            map.insert(p.to_string(), json!(price));
        }
        pricing.insert(out_type, Value::Object(map));
    }

    obj([
        ("id", json!(id)),
        (
            "status",
            get(tld, "status").cloned().unwrap_or(json!("Active")),
        ),
        (
            "maxchar",
            get_path(tld, &["constraints", "maxLenght"])
                .cloned()
                .unwrap_or(json!(63)),
        ),
        (
            "maxperiod",
            get(tld, "maxRegistrationPeriod")
                .cloned()
                .unwrap_or(json!(10)),
        ),
        (
            "minchar",
            get_path(tld, &["constraints", "minLength"])
                .cloned()
                .unwrap_or(json!(1)),
        ),
        (
            "minperiod",
            get(tld, "minRegistrationPeriod")
                .cloned()
                .unwrap_or(json!(1)),
        ),
        ("tld", tld.get("name").cloned().unwrap_or(Value::Null)),
        ("pricing", php::assoc_or_empty(pricing)),
        ("currencies", php::assoc_or_empty(currencies)),
    ])
}

/// php-dna `parseContactIds()`.
fn parse_contact_ids(contacts: &Value) -> Value {
    let mut result = Map::new();
    for key in ["Billing", "Technical", "Administrative", "Registrant"] {
        result.insert(key.to_string(), json!({"ID": 0}));
    }
    for c in php::values(contacts) {
        let api_type = get(c, "ContactType")
            .or_else(|| get(c, "contactType"))
            .map(php::strval)
            .unwrap_or_default();
        let key = match api_type.as_str() {
            "Billing" => "Billing",
            "Tech" => "Technical",
            "Admin" => "Administrative",
            "Registrant" => "Registrant",
            _ => continue,
        };
        let id = get(c, "handle")
            .or_else(|| get(c, "id"))
            .cloned()
            .unwrap_or(json!(0));
        result.insert(key.to_string(), obj([("ID", id)]));
    }
    Value::Object(result)
}

/// php-dna `parseDomainInfo()`.
pub(crate) fn parse_domain_info(data: &Value) -> Value {
    if !php::truthy(data) {
        return obj([
            ("result", json!(RESULT_ERROR)),
            ("error", set_error_str("RESPONSE", "", "")),
        ]);
    }
    let name_servers = match get(data, "nameservers") {
        Some(Value::Array(list)) => {
            Value::Array(list.iter().map(|v| json!(php::strval(v))).collect())
        }
        Some(Value::Object(map)) => Value::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), json!(php::strval(v))))
                .collect(),
        ),
        _ => json!([]),
    };
    let child_name_servers = match get(data, "hosts") {
        Some(hosts) => {
            let mapped: Vec<(String, Value)> = php::entries(hosts)
                .into_iter()
                .map(|(k, ns)| {
                    let ips: Vec<Value> = get(ns, "ipAddresses")
                        .map(|list| {
                            php::values(list)
                                .into_iter()
                                .map(|ip| ip.get("ipAddress").cloned().unwrap_or(Value::Null))
                                .collect()
                        })
                        .unwrap_or_default();
                    let ip = ips.last().cloned().unwrap_or(json!(""));
                    (
                        k,
                        obj([
                            ("ns", ns.get("name").cloned().unwrap_or(Value::Null)),
                            ("ip", ip),
                        ]),
                    )
                })
                .collect();
            if hosts.is_object() {
                Value::Object(mapped.into_iter().collect())
            } else {
                Value::Array(mapped.into_iter().map(|(_, v)| v).collect())
            }
        }
        None => json!([]),
    };
    let remaining = match get(data, "remainingDay") {
        Some(v) => php::intval(v),
        None => remaining_days_from_date(get(data, "expirationDate")),
    };
    let flag = |k: &str| {
        json!(if php::empty(get(data, k)) {
            "false"
        } else {
            "true"
        })
    };
    obj([
        (
            "data",
            obj([
                ("ID", json!(php::intval_or(get(data, "id"), 0))),
                (
                    "Status",
                    json!(normalize_domain_status(
                        get(data, "status").unwrap_or(&json!(""))
                    )),
                ),
                (
                    "DomainName",
                    json!(php::strval_or(
                        get(data, "domainName").or_else(|| get(data, "name")),
                        ""
                    )),
                ),
                (
                    "AuthCode",
                    json!(php::strval_or(
                        get(data, "authCode").or_else(|| get(data, "eppCode")),
                        ""
                    )),
                ),
                ("LockStatus", flag("lockStatus")),
                ("PrivacyProtectionStatus", flag("privacyProtectionStatus")),
                ("IsChildNameServer", flag("hosts")),
                (
                    "Contacts",
                    parse_contact_ids(get(data, "contacts").unwrap_or(&json!([]))),
                ),
                (
                    "Dates",
                    obj([
                        ("Start", json!(format_api_date(get(data, "startDate")))),
                        (
                            "Expiration",
                            json!(format_api_date(get(data, "expirationDate"))),
                        ),
                        ("RemainingDays", json!(remaining)),
                    ]),
                ),
                ("NameServers", name_servers),
                (
                    "Additional",
                    get(data, "additionalAttributes")
                        .map(php::to_array)
                        .unwrap_or(json!([])),
                ),
                ("ChildNameServers", child_name_servers),
            ]),
        ),
        ("result", json!(RESULT_OK)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn humanizes_members() {
        assert_eq!(
            humanize_validation_member("$.administrativeContact.postalCode"),
            "Admin PostalCode"
        );
        assert_eq!(
            humanize_validation_member("$.tldAttributes.TRABISCOUNTRYID"),
            "TldAttributes TRABISCOUNTRYID"
        );
        assert_eq!(
            humanize_validation_member("$.contacts[0].phone"),
            "Contacts Phone"
        );
        assert_eq!(humanize_validation_member("  "), "");
    }

    #[test]
    fn flattens_validation_errors() {
        let v = json!([
            {"members": ["$.a.b", "$.c"], "message": "bad"},
            {"members": [], "message": "plain"},
            "loose",
            {"members": ["$.a.b", "$.c"], "message": "bad"}
        ]);
        assert_eq!(
            flatten_validation_errors(&v),
            "A B, C - bad | plain | loose"
        );
    }

    #[test]
    fn detects_boilerplate() {
        assert!(is_abp_boilerplate(
            "Exception of type 'Volo.Abp.BusinessException' was thrown."
        ));
        assert!(is_abp_boilerplate("x EXCEPTION OF TYPE 'A' WAS THROWN"));
        assert!(!is_abp_boilerplate("Exception of type Volo was thrown"));
        assert_eq!(strip_dna_prefix("Dna  Host"), "Host");
        assert_eq!(strip_dna_prefix("DnaHost"), "DnaHost");
    }

    #[test]
    fn statuses() {
        assert_eq!(
            normalize_domain_status(&json!(3)),
            "WaitingForIncomingTransfer"
        );
        assert_eq!(normalize_domain_status(&json!("4")), "TransferredOut");
        assert_eq!(normalize_domain_status(&json!(99)), "99");
        assert_eq!(normalize_domain_status(&json!("Active")), "Active");
    }

    #[test]
    fn set_error_prefers_given_text_for_known_codes() {
        let e = set_error_str("RESPONSE", "", "");
        assert_eq!(
            e["Message"],
            "Invalid response received from server! Response is empty."
        );
        let e = set_error_str("CREDENTIALS", "Invalid response format", "x");
        assert_eq!(e["Message"], "Invalid response format");
        assert_eq!(e["Code"], "CREDENTIALS");
    }

    #[test]
    fn test_mode_strings() {
        assert!(" Yes ".into_test_mode());
        assert!("ON".into_test_mode());
        assert!("1".into_test_mode());
        assert!(!"false".into_test_mode());
        assert!(!"off".into_test_mode());
        assert!(!"".into_test_mode());
        assert!(!"2".into_test_mode());
    }
}
