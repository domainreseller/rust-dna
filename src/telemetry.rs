//! Error reporting and performance sampling, ported from php-dna's
//! `SharedApiConfigAndUtilsTrait`.
//!
//! Errors are posted to Sentry's store endpoint from a background thread
//! (fire and forget: never blocks or fails the caller). Only credentials are
//! masked; everything else is sent, as in php-dna. With an empty DSN (the
//! default for now) telemetry is a no-op.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde_json::{json, Map, Value};

use crate::php;
use crate::{APPLICATION, VERSION};

/// Sentry DSN for DomainNameAPI-Rust. Filled in when the Sentry project for this port exists.
pub const DEFAULT_SENTRY_DSN: &str = "";

/// Share of successful calls sent as performance records, out of 1000 (4%).
pub const PERFORMANCE_SAMPLE_RATE: u64 = 40;

/// Successful calls slower than this are always sampled.
pub const SLOW_CALL_MS: f64 = 1000.0;

/// Error messages that are expected noise and never reported.
pub const DEFAULT_IGNORED_ERRORS: [&str; 10] = [
    "Domain not found",
    "ERR_DOMAIN_NOT_FOUND",
    "Reseller not found",
    "Domain is not in updateable status",
    "balance is not sufficient",
    "Price definition not found",
    "TLD is not supported",
    "Invalid API credentials",
    "could not be found",
    "Insufficent reseller balance",
];

/// Seconds an external IP lookup is cached (php-dna `DEFAULT_CACHE_TTL`).
const IP_CACHE_TTL: Duration = Duration::from_secs(512);

/// Everything the telemetry layer needs to know about the last call.
#[derive(Debug, Clone, Default)]
pub struct CallContext {
    /// Reseller ID (UUID).
    pub reseller_id: String,
    /// `{url, method, payload}` of the last request.
    pub last_request: Value,
    /// Decoded last response (or `{raw_response}`).
    pub last_response: Value,
    /// Error envelope or parsed body of the last response.
    pub last_parsed_response: Value,
    /// HTTP status of the last response (0 when none).
    pub http_code: i64,
    /// `request:METHOD endpoint`.
    pub operation: String,
    /// Configured gateway URL.
    pub service_url: String,
    /// Seconds since the call started.
    pub elapsed: f64,
    /// Minimal payload requested.
    pub minimal: bool,
}

/// A stable grouping of an error message (see php-dna `buildSentryClassification`).
#[derive(Debug, Clone, PartialEq)]
pub struct Classification {
    /// Normalised message used as the event title.
    pub value: String,
    /// Volatile data extracted as tags.
    pub tags: Map<String, Value>,
    /// Explicit Sentry fingerprint.
    pub fingerprint: Vec<String>,
}

/// Parsed Sentry DSN.
#[derive(Debug, Clone, PartialEq)]
pub struct Dsn {
    /// Store endpoint, `https://<host>/api/<project>/store/`.
    pub store_url: String,
    /// Public key.
    pub public_key: String,
    /// Secret key (legacy DSNs only).
    pub secret_key: Option<String>,
}

impl Dsn {
    /// Parses `scheme://public[:secret]@host/project`. Returns `None` for an
    /// empty or invalid DSN.
    pub fn parse(dsn: &str) -> Option<Dsn> {
        let (_, rest) = dsn.split_once("://")?;
        let (auth, host_path) = rest.split_once('@')?;
        let (host, path) = host_path.split_once('/')?;
        let project = path.trim_start_matches('/');
        let (public_key, secret_key) = match auth.split_once(':') {
            Some((p, s)) => (p.to_string(), Some(s.to_string()).filter(|s| !s.is_empty())),
            None => (auth.to_string(), None),
        };
        if host.is_empty() || public_key.is_empty() || project.is_empty() {
            return None;
        }
        Some(Dsn {
            store_url: format!("https://{host}/api/{project}/store/"),
            public_key,
            secret_key,
        })
    }

    /// `X-Sentry-Auth` header value.
    pub fn auth_header(&self, client: &str) -> String {
        let mut parts = vec![
            "sentry_version=7".to_string(),
            format!("sentry_client={client}/{VERSION}"),
            format!("sentry_key={}", self.public_key),
        ];
        if let Some(secret) = &self.secret_key {
            parts.push(format!("sentry_secret={secret}"));
        }
        format!("Sentry {}", parts.join(", "))
    }
}

/// Telemetry settings and per-client state.
#[derive(Debug)]
pub(crate) struct Telemetry {
    pub enabled: bool,
    pub minimal: bool,
    pub dsn: String,
    session_trace_id: Option<String>,
}

impl Default for Telemetry {
    fn default() -> Self {
        Self {
            enabled: true,
            minimal: false,
            dsn: DEFAULT_SENTRY_DSN.to_string(),
            session_trace_id: None,
        }
    }
}

impl Telemetry {
    /// Report a failed call (fire and forget).
    pub fn report_error(&mut self, message: &str, code: i64, ctx: &CallContext) {
        if !self.enabled || is_ignored(message) {
            return;
        }
        let Some(dsn) = Dsn::parse(&self.dsn) else {
            return;
        };
        let event = build_error_event(message, code, ctx);
        let auth = dsn.auth_header("phplib-rust");
        send_in_background(dsn.store_url, auth, event, !ctx.minimal);
    }

    /// Possibly report a successful call as a performance transaction.
    pub fn sample_success(&mut self, duration_ms: f64, start: f64, ctx: &CallContext) {
        if !self.enabled {
            return;
        }
        if !(duration_ms > SLOW_CALL_MS || random_u64() % 1000 < PERFORMANCE_SAMPLE_RATE) {
            return;
        }
        let Some(dsn) = Dsn::parse(&self.dsn) else {
            return;
        };
        let trace_id = self
            .session_trace_id
            .get_or_insert_with(|| random_hex(16))
            .clone();
        let event = build_transaction_event(duration_ms, start, &trace_id, ctx);
        let auth = dsn.auth_header("php-api-rust");
        send_in_background(dsn.store_url, auth, event, !ctx.minimal);
    }
}

/// Whether a message matches one of [`DEFAULT_IGNORED_ERRORS`].
pub fn is_ignored(message: &str) -> bool {
    DEFAULT_IGNORED_ERRORS.iter().any(|e| message.contains(e))
}

fn key_is_sensitive(key: &str) -> bool {
    let k = key.to_ascii_lowercase();
    if ["auth", "epp", "password", "passwd", "secret", "token"]
        .iter()
        .any(|w| k.contains(w))
    {
        return true;
    }
    // api[_-]?key
    k.match_indices("api").any(|(i, _)| {
        let rest = &k[i + 3..];
        rest.starts_with("key") || rest.starts_with("_key") || rest.starts_with("-key")
    })
}

/// Masks credentials (auth/EPP codes, API keys, passwords, secrets, tokens)
/// with `***`. Everything else is kept.
pub fn mask(value: &Value) -> Value {
    mask_depth(value, 0)
}

fn mask_depth(value: &Value, depth: usize) -> Value {
    if depth > 8 {
        return Value::String("[…]".to_string());
    }
    match value {
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                if key_is_sensitive(k) {
                    let empty = v.is_null() || v.as_str() == Some("");
                    out.insert(k.clone(), if empty { v.clone() } else { json!("***") });
                } else {
                    out.insert(k.clone(), mask_depth(v, depth + 1));
                }
            }
            Value::Object(out)
        }
        Value::Array(list) => Value::Array(list.iter().map(|v| mask_depth(v, depth + 1)).collect()),
        other => other.clone(),
    }
}

// ---------------------------------------------------------------------------
// Message classification (regex-free port of the PCRE patterns)
// ---------------------------------------------------------------------------

fn is_word(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn starts_with_ci(hay: &[u8], at: usize, needle: &str) -> bool {
    let n = needle.as_bytes();
    at + n.len() <= hay.len() && hay[at..at + n.len()].eq_ignore_ascii_case(n)
}

fn skip_ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && (b[i] as char).is_ascii_whitespace() {
        i += 1;
    }
    i
}

/// `/API_(-?\d+)_ERROR/` → the number.
fn api_error_number(msg: &str) -> Option<String> {
    let b = msg.as_bytes();
    let mut i = 0;
    while let Some(pos) = msg[i..].find("API_") {
        let start = i + pos + 4;
        let mut j = start;
        if j < b.len() && b[j] == b'-' {
            j += 1;
        }
        let digits = j;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
        }
        if j > digits && msg[j..].starts_with("_ERROR") {
            return Some(msg[start..j].to_string());
        }
        i = start;
    }
    None
}

/// Matches `\d{1,3}(?:\.\d{1,3}){3}` at `i` followed by a word boundary.
fn match_ipv4(b: &[u8], i: usize) -> Option<usize> {
    fn group(b: &[u8], i: usize, left: usize) -> Option<usize> {
        for len in (1..=3).rev() {
            if i + len > b.len() || !b[i..i + len].iter().all(u8::is_ascii_digit) {
                continue;
            }
            let end = i + len;
            if left == 0 {
                if end == b.len() || !is_word(b[end]) {
                    return Some(end);
                }
            } else if end < b.len() && b[end] == b'.' {
                if let Some(e) = group(b, end + 1, left - 1) {
                    return Some(e);
                }
            }
        }
        None
    }
    if i > 0 && is_word(b[i - 1]) {
        return None;
    }
    group(b, i, 3)
}

fn first_ipv4(msg: &str) -> Option<String> {
    let b = msg.as_bytes();
    (0..b.len()).find_map(|i| match_ipv4(b, i).map(|end| msg[i..end].to_string()))
}

fn replace_ipv4(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if let Some(end) = match_ipv4(b, i) {
            out.extend_from_slice(b"{ip}");
            i = end;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn replace_quoted(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\'' || b[i] == b'"' {
            let q = b[i];
            let limit = (i + 82).min(b.len());
            if let Some(close) = (i + 1..limit).find(|&j| b[j] == q) {
                out.extend_from_slice(b"{v}");
                i = close + 1;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn replace_numbers(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i].is_ascii_digit() && (i == 0 || !is_word(b[i - 1])) {
            let mut j = i;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            if j - i >= 2 && (j == b.len() || !is_word(b[j])) {
                out.extend_from_slice(b"{n}");
                i = j;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `/API_-?\d+_ERROR\s*-\s*Failed\s*-\s*/i` → ''.
fn strip_api_failed(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    'outer: while i < b.len() {
        if starts_with_ci(b, i, "API_") {
            let mut j = i + 4;
            if j < b.len() && b[j] == b'-' {
                j += 1;
            }
            let d = j;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            if j > d && starts_with_ci(b, j, "_ERROR") {
                j = skip_ws(b, j + 6);
                if j < b.len() && b[j] == b'-' {
                    j = skip_ws(b, j + 1);
                    if starts_with_ci(b, j, "Failed") {
                        j = skip_ws(b, j + 6);
                        if j < b.len() && b[j] == b'-' {
                            i = skip_ws(b, j + 1);
                            continue 'outer;
                        }
                    }
                }
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `/\s*\(Reseller Id[^)]*\)\s*/i` → ''.
fn strip_reseller_id(s: &str) -> String {
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let j = skip_ws(b, i);
        if starts_with_ci(b, j, "(Reseller Id") {
            if let Some(close) = (j..b.len()).find(|&k| b[k] == b')') {
                i = skip_ws(b, close + 1);
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn collapse_ws(s: &str) -> String {
    s.split_ascii_whitespace().collect::<Vec<_>>().join(" ")
}

/// Tags shared by error and transaction events.
pub fn common_tags(ctx: &CallContext, error: Option<(&str, i64)>) -> Map<String, Value> {
    let mut tags = Map::new();
    if !ctx.reseller_id.is_empty() {
        tags.insert("reseller".into(), json!(ctx.reseller_id));
    }
    if let Some(Value::String(domain)) =
        php::get_path(&ctx.last_request, &["payload", "domainName"])
    {
        if !domain.is_empty() && !ctx.minimal {
            tags.insert("domain".into(), json!(domain));
        }
    }
    if !ctx.operation.is_empty() {
        tags.insert("operation".into(), json!(ctx.operation));
    }
    tags.insert("api_type".into(), json!("REST"));
    if let Some((message, code)) = error {
        if let Some(n) = api_error_number(message) {
            tags.insert("api_code".into(), json!(format!("API_{n}")));
        } else if code != 0 {
            tags.insert("api_code".into(), json!(format!("API_{code}")));
        }
        if code != 0 {
            tags.insert("status_code".into(), json!(code.to_string()));
        }
    }
    tags
}

/// Builds the stable title, tags and fingerprint for an error.
pub fn classify(message: &str, code: i64, ctx: &CallContext) -> Classification {
    let mut tags = common_tags(ctx, Some((message, code)));
    if let Some(ip) = first_ipv4(message) {
        tags.insert("ip_value".into(), json!(ip));
    }
    let api_code = tags
        .get("api_code")
        .and_then(Value::as_str)
        .map(str::to_string);

    let mut n = message.to_string();
    let lower = n.to_ascii_lowercase();
    if lower.starts_with("[api_error]:") {
        n = n[12..].trim_start().to_string();
    }
    n = strip_api_failed(&n);
    n = strip_reseller_id(&n);
    n = replace_ipv4(&n);
    n = replace_quoted(&n);
    n = replace_numbers(&n);
    n = collapse_ws(&n);

    if n.is_empty() {
        n = api_code.clone().unwrap_or_else(|| "Exception".to_string());
    } else if let Some(code) = &api_code {
        if !n.starts_with(code.as_str()) {
            n = format!("{code}: {n}");
        }
    }
    if n.len() > 160 {
        let mut cut = 157;
        while !n.is_char_boundary(cut) {
            cut -= 1;
        }
        n = format!("{}...", &n[..cut]);
    }
    let operation = tags
        .get("operation")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    Classification {
        fingerprint: vec!["dna".into(), APPLICATION.into(), operation, n.clone()],
        value: n,
        tags,
    }
}

fn environment(service_url: &str) -> &'static str {
    if service_url.to_ascii_lowercase().contains("ote.") {
        "ote"
    } else {
        "production"
    }
}

fn vhost_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_default()
}

fn url_path(url: &str) -> String {
    let after_scheme = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let path = after_scheme
        .find('/')
        .map(|i| &after_scheme[i..])
        .unwrap_or("");
    path.split(['?', '#']).next().unwrap_or("").to_string()
}

fn url_host(url: &str) -> String {
    let after_scheme = url.split_once("://").map(|(_, r)| r).unwrap_or("");
    let host_port = after_scheme.split(['/', '?', '#']).next().unwrap_or("");
    let host = host_port
        .rsplit_once('@')
        .map(|(_, h)| h)
        .unwrap_or(host_port);
    host.split(':').next().unwrap_or("").to_string()
}

fn iso_now_millis() -> String {
    iso_millis(php::microtime())
}

fn iso_millis(t: f64) -> String {
    let secs = t.floor() as i64;
    let ms = ((t - t.floor()) * 1000.0).round().min(999.0) as i64;
    format!("{}.{ms:03}Z", php::format_date(secs))
}

/// Builds the Sentry error event php-dna would send for this failure.
pub fn build_error_event(message: &str, code: i64, ctx: &CallContext) -> Value {
    let class = classify(message, code, ctx);
    let mut tags = obj_map([
        ("handled", json!("yes")),
        ("level", json!("error")),
        ("url", json!("NA")),
        ("transaction", json!("NA")),
        ("trace_id", json!(random_hex(8))),
        ("runtime_name", json!("Rust")),
        (
            "runtime_version",
            json!(option_env!("CARGO_PKG_RUST_VERSION").unwrap_or("")),
        ),
        ("elapsed_time", json!(php::number_format(ctx.elapsed, 4))),
        ("application", json!(APPLICATION)),
        ("library_version", json!(VERSION)),
        ("ip_address", json!("")),
        ("vhost_user", json!(vhost_user())),
    ]);
    for (k, v) in class.tags.clone() {
        tags.insert(k, v);
    }
    let culprit = format!("DomainNameApi via {}", ctx.operation);
    let mut event = json!({
        "event_id": random_hex(16),
        "timestamp": php::format_date(php::now()) + "Z",
        "level": "error",
        "logger": "rust",
        "platform": "native",
        "culprit": culprit,
        "message": {"formatted": class.value},
        "exception": {"values": [{
            "type": format!("{APPLICATION} Exception"),
            "value": class.value,
            "stacktrace": {"frames": [
                {"filename": "src/client.rs", "lineno": 0, "function": format!("DomainNameApi::{}", ctx.operation)}
            ]}
        }]},
        "tags": Value::Object(tags),
        "extra": {
            "request_data": mask(&ctx.last_request),
            "response_data": mask(&ctx.last_response),
            "original_message": message,
        },
        "release": VERSION,
        "environment": environment(&ctx.service_url),
        "fingerprint": class.fingerprint,
    });
    if ctx.minimal {
        if let Some(tags) = event["tags"].as_object_mut() {
            for k in ["url", "ip_address", "vhost_user", "ip_value"] {
                tags.remove(k);
            }
        }
        let method = php::strval_or(php::get(&ctx.last_request, "method"), "");
        let endpoint = url_path(&php::strval_or(php::get(&ctx.last_request, "url"), ""));
        event["extra"]["request_data"] = json!({"method": method, "endpoint": endpoint});
        event["extra"]["response_data"] = json!({"http_code": ctx.http_code});
    }
    event
}

fn map_trace_status(success: bool, api_code: Option<&str>) -> &'static str {
    if success {
        return "ok";
    }
    let Some(code) = api_code else {
        return "unknown";
    };
    // first -?\d+
    let b = code.as_bytes();
    let Some(start) = (0..b.len()).find(|&i| b[i].is_ascii_digit()) else {
        return "unknown";
    };
    let neg = start > 0 && b[start - 1] == b'-';
    let mut end = start;
    while end < b.len() && b[end].is_ascii_digit() {
        end += 1;
    }
    let mut n: i64 = code[start..end].parse().unwrap_or(0);
    if neg {
        n = -n;
    }
    match n {
        401 | 2202 => "unauthenticated",
        403 => "permission_denied",
        404 | 510 => "not_found",
        408 | 504 => "deadline_exceeded",
        -1 => "unavailable",
        500..=599 => "internal_error",
        400..=499 => "invalid_argument",
        _ => "unknown",
    }
}

/// Builds the performance transaction for a successful call.
pub fn build_transaction_event(
    duration_ms: f64,
    start: f64,
    trace_id: &str,
    ctx: &CallContext,
) -> Value {
    let api_code = php::get(&ctx.last_parsed_response, "Code")
        .and_then(Value::as_str)
        .or_else(|| php::get_path(&ctx.last_response, &["error", "code"]).and_then(Value::as_str))
        .map(str::to_string);
    let status = map_trace_status(true, api_code.as_deref());
    let mut tags = obj_map([
        ("application", json!(APPLICATION)),
        ("library_version", json!(VERSION)),
        ("ip_address", json!("")),
        (
            "vhost_user",
            json!(if ctx.minimal {
                String::new()
            } else {
                vhost_user()
            }),
        ),
    ]);
    for (k, v) in common_tags(ctx, None) {
        tags.insert(k, v);
    }
    if let Some(code) = &api_code {
        tags.entry("api_code").or_insert(json!(code));
    }
    if ctx.http_code != 0 {
        tags.insert("status_code".into(), json!(ctx.http_code.to_string()));
    }
    let method = php::get(&ctx.last_request, "method")
        .map(php::strval)
        .filter(|m| !m.is_empty())
        .unwrap_or_else(|| "GET".to_string());
    let url = php::strval_or(php::get(&ctx.last_request, "url"), "");
    let host = if url.is_empty() {
        url_host(&ctx.service_url)
    } else {
        url_host(&url)
    };
    let mut span_data = obj_map([("http.request.method", json!(method))]);
    if !host.is_empty() {
        span_data.insert("server.address".into(), json!(host));
    }
    if !url.is_empty() && !ctx.minimal {
        span_data.insert(
            "url.full".into(),
            json!(url.split('?').next().unwrap_or("")),
        );
    }
    if ctx.http_code != 0 {
        span_data.insert("http.response.status_code".into(), json!(ctx.http_code));
    }
    let description = if host.is_empty() {
        ctx.operation.clone()
    } else {
        format!("{method} https://{host}")
    };
    let root_span = random_hex(8);
    let start_ts = iso_millis(start);
    let end_ts = iso_now_millis();
    json!({
        "event_id": random_hex(16),
        "platform": "native",
        "level": "info",
        "type": "transaction",
        "transaction": format!("API.{}", ctx.operation),
        "transaction_info": {"source": "custom"},
        "release": VERSION,
        "environment": environment(&ctx.service_url),
        "timestamp": end_ts,
        "start_timestamp": start_ts,
        "contexts": {
            "trace": {"trace_id": trace_id, "span_id": root_span, "op": "http.client", "status": status},
            "runtime": {"name": "rust", "version": option_env!("CARGO_PKG_RUST_VERSION").unwrap_or("")},
        },
        "tags": Value::Object(tags),
        "measurements": {"duration": {"value": duration_ms, "unit": "millisecond"}},
        "spans": [{
            "span_id": random_hex(8),
            "parent_span_id": root_span,
            "trace_id": trace_id,
            "op": "http.client",
            "description": description,
            "status": status,
            "data": Value::Object(span_data),
            "start_timestamp": start_ts,
            "timestamp": end_ts,
        }],
    })
}

fn obj_map<const N: usize>(pairs: [(&str, Value); N]) -> Map<String, Value> {
    let mut map = Map::new();
    for (k, v) in pairs {
        map.insert(k.to_string(), v);
    }
    map
}

// ---------------------------------------------------------------------------
// Sending
// ---------------------------------------------------------------------------

static IP_CACHE: Mutex<Option<(String, Instant)>> = Mutex::new(None);

fn server_ip() -> String {
    if let Ok(guard) = IP_CACHE.lock() {
        if let Some((ip, at)) = guard.as_ref() {
            if at.elapsed() < IP_CACHE_TTL {
                return ip.clone();
            }
        }
    }
    let agent = crate::transport::build_agent(Duration::from_secs(2), Duration::from_secs(1), true);
    let fetched = agent
        .get("https://api.ipify.org")
        .call()
        .ok()
        .and_then(|r| r.into_string().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| s.parse::<std::net::IpAddr>().is_ok());
    match fetched {
        Some(ip) => {
            if let Ok(mut guard) = IP_CACHE.lock() {
                *guard = Some((ip.clone(), Instant::now()));
            }
            ip
        }
        None => "unknown".to_string(),
    }
}

fn send_in_background(url: String, auth: String, mut event: Value, lookup_ip: bool) {
    let spawned = std::thread::Builder::new()
        .name("dna-telemetry".into())
        .spawn(move || {
            if lookup_ip {
                if let Some(tags) = event["tags"].as_object_mut() {
                    tags.insert("ip_address".into(), json!(server_ip()));
                }
            }
            let agent = crate::transport::build_agent(
                Duration::from_millis(500),
                Duration::from_millis(200),
                true,
            );
            let _ = agent
                .post(&url)
                .set("Content-Type", "application/json")
                .set("X-Sentry-Auth", &auth)
                .send_string(&event.to_string());
        });
    // Telemetry must never fail the caller.
    drop(spawned);
}

// ---------------------------------------------------------------------------
// Randomness without a dependency
// ---------------------------------------------------------------------------

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn random_u64() -> u64 {
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u64(COUNTER.fetch_add(1, Ordering::Relaxed));
    hasher.write_u128(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
    );
    hasher.finish()
}

/// `bin2hex(random_bytes($n))`.
pub fn random_hex(bytes: usize) -> String {
    let mut out = String::with_capacity(bytes * 2);
    while out.len() < bytes * 2 {
        out.push_str(&format!("{:016x}", random_u64()));
    }
    out.truncate(bytes * 2);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> CallContext {
        CallContext {
            reseller_id: "00000000-0000-4000-8000-000000000001".into(),
            last_request: json!({"url": "https://api.domainresellerapi.com/api/v1/domains/info", "method": "GET", "payload": {"domainName": "example.com", "authCode": "secret"}}),
            last_response: json!({"error": {"message": "x"}}),
            operation: "request:GET domains/info".into(),
            service_url: "https://api.domainresellerapi.com/api/v1".into(),
            http_code: 500,
            ..Default::default()
        }
    }

    #[test]
    fn masks_only_credentials() {
        let masked = mask(&json!({
            "domainName": "example.com", "authCode": "Ab1", "eppCode": "", "X-API-KEY": "k",
            "api_key": "k", "password": "p", "contacts": [{"token": "t", "email": "e"}], "apikeyx": "z"
        }));
        assert_eq!(masked["domainName"], "example.com");
        assert_eq!(masked["authCode"], "***");
        assert_eq!(masked["eppCode"], "");
        assert_eq!(masked["X-API-KEY"], "***");
        assert_eq!(masked["api_key"], "***");
        assert_eq!(masked["password"], "***");
        assert_eq!(masked["contacts"][0]["token"], "***");
        assert_eq!(masked["contacts"][0]["email"], "e");
        assert_eq!(masked["apikeyx"], "***");
    }

    #[test]
    fn classification_normalises_volatile_parts() {
        let c = classify(
            "Your current IP address (192.0.2.44) is not authorized for 'example.com' 12345 (Reseller Id : abc)",
            500,
            &ctx(),
        );
        assert_eq!(
            c.value,
            "API_500: Your current IP address ({ip}) is not authorized for {v} {n}"
        );
        assert_eq!(c.tags["ip_value"], "192.0.2.44");
        assert_eq!(c.tags["api_code"], "API_500");
        assert_eq!(c.tags["status_code"], "500");
        assert_eq!(c.tags["domain"], "example.com");
        assert_eq!(c.fingerprint[1], APPLICATION);
        assert_eq!(c.fingerprint[2], "request:GET domains/info");
    }

    #[test]
    fn classification_of_empty_message_uses_code() {
        assert_eq!(classify("", 429, &ctx()).value, "API_429");
        assert_eq!(
            classify("API_2302_ERROR - Failed - Object exists", 0, &ctx()).value,
            "API_2302: Object exists"
        );
    }

    #[test]
    fn ignored_errors() {
        assert!(is_ignored(
            "The domain name you requested could not be found."
        ));
        assert!(is_ignored("Invalid API credentials"));
        assert!(!is_ignored("HTTP 500 - Internal Server Error"));
    }

    #[test]
    fn error_event_masks_and_tags() {
        let event = build_error_event("HTTP 500 - Internal Server Error", 500, &ctx());
        assert_eq!(event["tags"]["application"], APPLICATION);
        assert_eq!(event["tags"]["library_version"], VERSION);
        assert_eq!(event["release"], VERSION);
        assert_eq!(event["environment"], "production");
        assert_eq!(event["extra"]["request_data"]["payload"]["authCode"], "***");
        assert_eq!(
            event["extra"]["request_data"]["payload"]["domainName"],
            "example.com"
        );
    }

    #[test]
    fn minimal_event_drops_request_data() {
        let mut c = ctx();
        c.minimal = true;
        let event = build_error_event("HTTP 500 - Internal Server Error", 500, &c);
        assert_eq!(
            event["extra"]["request_data"],
            json!({"method": "GET", "endpoint": "/api/v1/domains/info"})
        );
        assert_eq!(event["extra"]["response_data"], json!({"http_code": 500}));
        assert!(event["tags"].get("domain").is_none());
        assert!(event["tags"].get("ip_address").is_none());
    }

    #[test]
    fn dsn_parsing() {
        assert!(Dsn::parse("").is_none());
        let d = Dsn::parse("https://abc@sentry.example.com/9").unwrap();
        assert_eq!(d.store_url, "https://sentry.example.com/api/9/store/");
        assert_eq!(
            d.auth_header("x"),
            format!("Sentry sentry_version=7, sentry_client=x/{VERSION}, sentry_key=abc")
        );
    }

    #[test]
    fn transaction_event_shape() {
        let e = build_transaction_event(12.5, 1_700_000_000.25, "t", &ctx());
        assert_eq!(e["type"], "transaction");
        assert_eq!(
            e["spans"][0]["description"],
            "GET https://api.domainresellerapi.com"
        );
        assert_eq!(e["tags"]["application"], APPLICATION);
        assert_eq!(e["start_timestamp"], "2023-11-14T22:13:20.250Z");
    }
}
