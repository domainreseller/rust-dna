//! Shared test helpers: a fake transport that serves queued responses and
//! records every request, and a php-dna-tolerant JSON comparator.
#![allow(dead_code)]

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use domainnameapi::php;
use domainnameapi::{DomainNameApi, HttpRequest, HttpResponse, Transport, TransportError};
use serde_json::{Map, Value};

pub const RESELLER: &str = "00000000-0000-4000-8000-000000000001";
pub const API_KEY: &str = "conformance-api-key";
pub const BASE_URL: &str = "http://mock.test/api/v1";

/// What the fake transport answers.
#[derive(Clone, Debug)]
pub enum Reply {
    Http(u16, String),
    Fail(TransportError),
}

#[derive(Default)]
pub struct State {
    pub queue: VecDeque<Reply>,
    pub requests: Vec<HttpRequest>,
    pub exhausted: bool,
}

#[derive(Clone, Default)]
pub struct FakeTransport {
    pub state: Arc<Mutex<State>>,
}

impl FakeTransport {
    pub fn new(replies: Vec<Reply>) -> Self {
        let t = FakeTransport::default();
        t.state.lock().unwrap().queue = replies.into();
        t
    }

    pub fn requests(&self) -> Vec<HttpRequest> {
        self.state.lock().unwrap().requests.clone()
    }

    pub fn exhausted(&self) -> bool {
        self.state.lock().unwrap().exhausted
    }
}

impl Transport for FakeTransport {
    fn send(&self, request: &HttpRequest) -> Result<HttpResponse, TransportError> {
        let mut state = self.state.lock().unwrap();
        state.requests.push(request.clone());
        match state.queue.pop_front() {
            Some(Reply::Http(status, body)) => Ok(HttpResponse {
                status,
                body,
                headers: vec![("content-type".into(), "application/json".into())],
            }),
            Some(Reply::Fail(err)) => Err(err),
            None => {
                state.exhausted = true;
                Ok(HttpResponse {
                    status: 500,
                    body: r#"{"error":{"message":"conformance: no queued response"}}"#.into(),
                    headers: vec![],
                })
            }
        }
    }
}

/// A client wired to a fake transport, telemetry off (and its DSN emptied,
/// so the suite can never reach the real Sentry project), no zone pacing.
pub fn client(replies: Vec<Reply>) -> (DomainNameApi, FakeTransport) {
    let transport = FakeTransport::new(replies);
    let mut dna = DomainNameApi::new(RESELLER, API_KEY, false);
    dna.set_error_reporting_enabled(false);
    dna.set_telemetry_dsn("");
    dna.set_zone_min_interval(std::time::Duration::ZERO);
    dna.set_service_url(BASE_URL);
    dna.set_transport(Box::new(transport.clone()));
    (dna, transport)
}

/// Split a recorded URL into (path relative to /api/v1/, decoded query map).
pub fn split_url(url: &str) -> (String, Map<String, Value>) {
    let rest = url
        .strip_prefix(BASE_URL)
        .unwrap_or(url)
        .trim_start_matches('/');
    let (path, query) = rest.split_once('?').unwrap_or((rest, ""));
    let mut map = Map::new();
    for pair in query.split('&').filter(|p| !p.is_empty()) {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        map.insert(php::urldecode(k), Value::String(php::urldecode(v)));
    }
    (path.to_string(), map)
}

/// Equality with php-dna's JSON quirks tolerated: `[]` equals `{}`, numbers
/// compare by value, object key order is ignored.
pub fn loose_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (Value::Array(x), Value::Object(y)) | (Value::Object(y), Value::Array(x)) => {
            x.is_empty() && y.is_empty()
        }
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| loose_eq(p, q))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).map(|w| loose_eq(v, w)).unwrap_or(false))
        }
        _ => a == b,
    }
}

/// First differing path between two values (for failure messages).
pub fn first_diff(a: &Value, b: &Value, path: &str) -> Option<String> {
    if loose_eq(a, b) {
        return None;
    }
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            for (k, v) in x {
                let p = format!("{path}.{k}");
                match y.get(k) {
                    None => return Some(format!("{p}: missing on expected side")),
                    Some(w) => {
                        if let Some(d) = first_diff(v, w, &p) {
                            return Some(d);
                        }
                    }
                }
            }
            for k in y.keys() {
                if !x.contains_key(k) {
                    return Some(format!("{path}.{k}: missing on actual side"));
                }
            }
            None
        }
        (Value::Array(x), Value::Array(y)) if x.len() == y.len() => {
            for (i, (v, w)) in x.iter().zip(y).enumerate() {
                if let Some(d) = first_diff(v, w, &format!("{path}[{i}]")) {
                    return Some(d);
                }
            }
            None
        }
        _ => Some(format!("{path}: actual {a} != expected {b}")),
    }
}
