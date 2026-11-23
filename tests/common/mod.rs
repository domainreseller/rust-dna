//! A fake Domain Name API: a small HTTP/1.1 server on 127.0.0.1 that records every request and answers with queued
//! replies. Tests never call the real API.

#![allow(dead_code)]

use domainnameapi::Client;
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// A request the fake API received.
#[derive(Debug, Clone)]
pub struct Recorded {
    pub method: String,
    pub path: String,
    pub query: Vec<(String, String)>,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Recorded {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_ascii_lowercase()).map(String::as_str)
    }

    pub fn query(&self, name: &str) -> Option<&str> {
        self.query.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
    }

    pub fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.body).expect("request body is JSON")
    }
}

/// An answer of the fake API.
#[derive(Debug, Clone)]
pub struct Reply {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
    /// Waits this long before answering.
    pub delay: Option<Duration>,
    /// Closes the connection without answering: a network error for the client.
    pub hang_up: bool,
}

impl Reply {
    pub fn json(status: u16, body: serde_json::Value) -> Reply {
        Reply::text(status, &body.to_string())
    }

    pub fn text(status: u16, body: &str) -> Reply {
        Reply { status, headers: Vec::new(), body: body.to_string(), delay: None, hang_up: false }
    }

    pub fn hang_up() -> Reply {
        Reply { hang_up: true, ..Reply::text(200, "") }
    }

    pub fn header(mut self, name: &str, value: &str) -> Reply {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    pub fn delay(mut self, delay: Duration) -> Reply {
        self.delay = Some(delay);
        self
    }
}

#[derive(Default)]
struct State {
    requests: Vec<Recorded>,
    replies: VecDeque<Reply>,
}

pub struct FakeApi {
    pub addr: SocketAddr,
    state: Arc<Mutex<State>>,
}

impl FakeApi {
    pub async fn start() -> FakeApi {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let state = Arc::new(Mutex::new(State::default()));
        let shared = state.clone();
        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let state = shared.clone();
                tokio::spawn(async move {
                    let _ = serve(stream, state).await;
                });
            }
        });
        FakeApi { addr, state }
    }

    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Queues an answer. Without queued answers the fake API answers 200 with `{}` (or `[]` for the endpoints that
    /// return a list).
    pub fn reply(&self, reply: Reply) -> &Self {
        self.state.lock().unwrap().replies.push_back(reply);
        self
    }

    pub fn reply_json(&self, status: u16, body: serde_json::Value) -> &Self {
        self.reply(Reply::json(status, body))
    }

    pub fn requests(&self) -> Vec<Recorded> {
        self.state.lock().unwrap().requests.clone()
    }

    pub fn last(&self) -> Recorded {
        self.requests().pop().expect("a request was sent")
    }

    /// A client for this fake API: test credentials, no pacing.
    pub fn client(&self) -> Client {
        self.builder().build().unwrap()
    }

    pub fn builder(&self) -> domainnameapi::ClientBuilder {
        Client::builder().credentials("test-reseller", "test-key").base_url(self.base_url()).pace_requests(false)
    }
}

async fn serve(mut stream: TcpStream, state: Arc<Mutex<State>>) -> std::io::Result<()> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 8192];
    let head_end = loop {
        let n = stream.read(&mut chunk).await?;
        if n == 0 {
            return Ok(());
        }
        buffer.extend_from_slice(&chunk[..n]);
        if let Some(i) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
            break i + 4;
        }
    };
    let head = String::from_utf8_lossy(&buffer[..head_end]).to_string();
    let mut lines = head.split("\r\n");
    let mut first = lines.next().unwrap_or_default().split(' ');
    let method = first.next().unwrap_or_default().to_string();
    let target = first.next().unwrap_or_default().to_string();
    let mut headers = HashMap::new();
    for line in lines.filter(|l| !l.is_empty()) {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    let length: usize = headers.get("content-length").and_then(|v| v.parse().ok()).unwrap_or(0);
    while buffer.len() < head_end + length {
        let n = stream.read(&mut chunk).await?;
        if n == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..n]);
    }
    let body = String::from_utf8_lossy(&buffer[head_end..]).to_string();
    let (path, query) = match target.split_once('?') {
        Some((path, query)) => (path.to_string(), parse_query(query)),
        None => (target.clone(), Vec::new()),
    };

    let reply = {
        let mut state = state.lock().unwrap();
        state.requests.push(Recorded { method: method.clone(), path: path.clone(), query, headers, body });
        state.replies.pop_front().unwrap_or_else(|| default_reply(&method, &path))
    };
    if let Some(delay) = reply.delay {
        tokio::time::sleep(delay).await;
    }
    if reply.hang_up {
        return Ok(());
    }
    let mut response = format!(
        "HTTP/1.1 {} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
        reply.status,
        reply.body.len()
    );
    for (name, value) in &reply.headers {
        response.push_str(&format!("{name}: {value}\r\n"));
    }
    response.push_str("\r\n");
    response.push_str(&reply.body);
    stream.write_all(response.as_bytes()).await?;
    stream.shutdown().await
}

fn default_reply(method: &str, path: &str) -> Reply {
    let list = (method == "GET" && path.ends_with("/domains/zones"))
        || path.ends_with("/hostings/packages")
        || path.ends_with("/hostings/plans");
    Reply::text(200, if list { "[]" } else { "{}" })
}

fn parse_query(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|pair| {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            (decode(k), decode(v))
        })
        .collect()
}

fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap();
                out.push(u8::from_str_radix(hex, 16).unwrap());
                i += 3;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).unwrap()
}
