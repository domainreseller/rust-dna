//! HTTP layer: headers, query/body encoding, error mapping, transport
//! failures, test mode, diagnostics and zone pacing. Uses a fake transport
//! and, for the real `UreqTransport`, a throwaway HTTP server on 127.0.0.1.

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

use common::{client, split_url, Reply};
use domainnameapi::{
    json, DomainNameApi, HttpRequest, Transport, TransportError, UreqTransport, URL_OTE, URL_PROD,
    USER_AGENT,
};
use serde_json::Value;

fn header(req: &HttpRequest, name: &str) -> Option<String> {
    req.headers
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.clone())
}

#[test]
fn sends_required_headers() {
    let (mut dna, t) = client(vec![Reply::Http(200, "{}".into())]);
    dna.get_details("example.com");
    let req = &t.requests()[0];
    assert_eq!(
        header(req, "Content-Type").as_deref(),
        Some("application/json")
    );
    assert_eq!(header(req, "Accept").as_deref(), Some("application/json"));
    assert_eq!(header(req, "X-API-KEY").as_deref(), Some(common::API_KEY));
    assert_eq!(header(req, "__reseller").as_deref(), Some(common::RESELLER));
    assert_eq!(header(req, "User-Agent").as_deref(), Some(USER_AGENT));
    assert_eq!(USER_AGENT, "DomainNameAPI-Rust/1.0.0");
    assert_eq!(req.timeout, Duration::from_secs(30));
    assert_eq!(req.connect_timeout, Duration::from_secs(10));
    assert!(req.verify_tls);
}

#[test]
fn get_uses_query_string_and_post_uses_json_body() {
    let (mut dna, t) = client(vec![
        Reply::Http(200, "{}".into()),
        Reply::Http(200, "".into()),
    ]);
    dna.get_list(&json!({"Flag": true, "Off": false, "Skip": null, "Name": "a b"}));
    dna.enable_theft_protection_lock("example.com");
    let reqs = t.requests();
    assert_eq!(reqs[0].method, "GET");
    assert!(reqs[0].body.is_none());
    assert_eq!(
        reqs[0].url,
        format!(
            "{}/domains?MaxResultCount=200&SkipCount=0&Flag=1&Off=0&Name=a+b",
            common::BASE_URL
        )
    );
    assert_eq!(reqs[1].method, "POST");
    assert_eq!(
        reqs[1].body.as_deref(),
        Some(r#"{"domainName":"example.com"}"#)
    );
}

#[test]
fn credentials_are_trimmed_and_test_mode_parsed() {
    let dna = DomainNameApi::new("  reseller \n", "\tkey ", false);
    assert_eq!(dna.get_service_url(), URL_PROD);
    for yes in ["true", "1", "on", "yes", " YES ", "On"] {
        assert_eq!(
            DomainNameApi::new("r", "k", yes).get_service_url(),
            URL_OTE,
            "{yes}"
        );
    }
    for no in ["false", "0", "off", "no", "", "maybe"] {
        assert_eq!(
            DomainNameApi::new("r", "k", no).get_service_url(),
            URL_PROD,
            "{no}"
        );
    }
    assert_eq!(
        DomainNameApi::new("r", "k", true).get_service_url(),
        URL_OTE
    );
    assert_eq!(
        DomainNameApi::new("r", "k", String::from("yes")).get_service_url(),
        URL_OTE
    );

    let mut dna = DomainNameApi::new("  reseller \n", "\tkey ", false);
    let t = common::FakeTransport::new(vec![Reply::Http(200, "{}".into())]);
    dna.set_error_reporting_enabled(false);
    dna.set_transport(Box::new(t.clone()));
    dna.get_details("example.com");
    let req = &t.requests()[0];
    assert_eq!(header(req, "X-API-KEY").as_deref(), Some("key"));
    assert_eq!(header(req, "__reseller").as_deref(), Some("reseller"));
}

#[test]
fn ssl_verification_switch_reaches_transport() {
    let (mut dna, t) = client(vec![Reply::Http(200, "{}".into())]);
    dna.set_ssl_verification(false);
    dna.get_details("example.com");
    assert!(!t.requests()[0].verify_tls);
}

#[test]
fn timeout_is_an_error_not_an_empty_success() {
    let (mut dna, _t) = client(vec![Reply::Fail(TransportError::new(
        28,
        "Operation timed out after 30001 milliseconds",
    ))]);
    let out = dna.modify_name_server("example.com", &json!(["ns1.example.net"]));
    assert_eq!(out["result"], "ERROR");
    let message = out["error"]["Message"].as_str().unwrap();
    assert!(
        message
            .starts_with("Could not reach the DomainNameAPI service: the connection timed out. ("),
        "{message}"
    );
    assert_eq!(out["error"]["Code"], "CURL_28");
    assert_eq!(dna.get_parsed_response_data()["Code"], "CURL_28");
    assert_eq!(out["error"]["Details"], json!(message));
}

#[test]
fn other_transport_failures_use_curl_message() {
    let (mut dna, _t) = client(vec![Reply::Fail(TransportError::new(
        6,
        "Could not resolve host: nowhere.invalid",
    ))]);
    let out = dna.get_reseller_details();
    assert_eq!(out["result"], "ERROR");
    assert_eq!(
        out["error"]["Message"],
        "Curl error during request: Could not resolve host: nowhere.invalid"
    );
    assert_eq!(out["error"]["Code"], "CURL_6");
    assert_eq!(dna.get_parsed_response_data()["Code"], "CURL_6");
}

#[test]
fn error_mapping_rules() {
    // Boilerplate message with no useful details → synthesised message.
    let body = json!({"error": {"code": "Dna.X:1", "message": "Exception of type 'Volo.Abp.BusinessException' was thrown.", "details": null}}).to_string();
    let (mut dna, _t) = client(vec![Reply::Http(400, body)]);
    let out = dna.get_details("example.com");
    // details fall back to the raw body, which carries the boilerplate too.
    assert_eq!(
        out["error"]["Message"],
        "The operation could not be completed by the registry (error: Dna.X:1). Please try again or contact support."
    );

    let body = json!({"error": {"code": "Dna.X:1", "message": "Exception of type 'A' was thrown.", "details": "Exception of type 'B' was thrown."}}).to_string();
    let (mut dna, _t) = client(vec![Reply::Http(400, body)]);
    let out = dna.get_details("example.com");
    assert_eq!(
        out["error"]["Message"],
        "The operation could not be completed by the registry (error: Dna.X:1). Please try again or contact support."
    );

    // Top-level validationErrors are flattened too.
    let body = json!({"message": "Validation failed.", "validationErrors": [{"members": ["$.registrantContact.city"], "message": "City is required"}]}).to_string();
    let (mut dna, _t) = client(vec![Reply::Http(422, body)]);
    let out = dna.get_details("example.com");
    assert_eq!(out["error"]["Code"], "API_422");
    assert_eq!(
        out["error"]["Details"],
        "Registrant City - City is required"
    );

    // Unknown status classes.
    let (mut dna, _t) = client(vec![Reply::Http(418, String::new())]);
    assert_eq!(
        dna.get_details("example.com")["error"]["Message"],
        "HTTP 418 - Client Error"
    );
    let (mut dna, _t) = client(vec![Reply::Http(599, String::new())]);
    assert_eq!(
        dna.get_details("example.com")["error"]["Message"],
        "HTTP 599 - Server Error"
    );
}

#[test]
fn diagnostics_reflect_last_call() {
    let (mut dna, _t) = client(vec![Reply::Http(502, "<html>bad</html>".into())]);
    dna.get_details("example.com");
    assert_eq!(dna.get_last_function(), "request:GET domains/info");
    assert_eq!(
        dna.get_request_data()["url"],
        format!("{}/domains/info", common::BASE_URL)
    );
    assert_eq!(
        dna.get_request_data()["payload"],
        json!({"DomainName": "example.com"})
    );
    assert_eq!(
        dna.get_response_data(),
        json!({"raw_response": "<html>bad</html>"})
    );
    assert_eq!(dna.get_response_headers()["http_code"], 502);

    let (mut dna, _t) = client(vec![Reply::Http(200, String::new())]);
    dna.add_resource_record("example.com", "www", "A", &json!("192.0.2.1"), 300, false);
    assert_eq!(
        dna.get_last_function(),
        "request:POST domains/zones",
        "query string stays out of the operation"
    );
}

#[test]
fn zone_calls_are_paced_other_calls_are_not() {
    let replies = vec![Reply::Http(200, "[]".into()); 4];
    let (mut dna, _t) = client(replies);
    dna.set_zone_min_interval(Duration::from_millis(250));
    let start = Instant::now();
    dna.get_details("example.com");
    dna.get_details("example.com");
    assert!(start.elapsed() < Duration::from_millis(200));
    let start = Instant::now();
    dna.get_resource_records("example.com");
    dna.apply_resource_records("example.com");
    assert!(
        start.elapsed() >= Duration::from_millis(240),
        "{:?}",
        start.elapsed()
    );
}

#[test]
fn local_validation_makes_no_request() {
    let (mut dna, t) = client(vec![]);
    assert_eq!(
        dna.set_forwarding("example.com", "", "Standard")["error"]["Message"],
        "Redirect address is required."
    );
    assert_eq!(
        dna.add_resource_record("example.com", "x", "", &json!("y"), 3600, true)["error"]
            ["Message"],
        "Record type is required."
    );
    let out = dna.check_availability(Vec::<String>::new(), ["com"], 1, "create");
    assert_eq!(out["error"]["Code"], "AVAILABILITY");
    assert!(t.requests().is_empty());
}

#[test]
fn transfer_never_sends_contacts() {
    let ok = json!({"success": true, "status": "Active", "expirationDate": "2030-01-01T00:00:00"})
        .to_string();
    let (mut dna, t) = client(vec![Reply::Http(200, ok)]);
    let out = dna.transfer("example.com", "Ab1", 1);
    assert_eq!(out["result"], "OK");
    let body: Value = serde_json::from_str(t.requests()[0].body.as_deref().unwrap()).unwrap();
    assert_eq!(
        body,
        json!({"domainName": "example.com", "authCode": "Ab1", "period": 1, "contacts": []})
    );
}

#[test]
fn refusals_and_gateway_codes() {
    // HTTP 200 with success:false is a refusal, not a success.
    let refused = json!({"success": false, "message": "Domain is not available"}).to_string();
    let (mut dna, _t) = client(vec![
        Reply::Http(200, refused.clone()),
        Reply::Http(200, refused),
    ]);
    let out = dna.renew("example.com", 1);
    assert_eq!(
        out["error"],
        json!({"Code": "RENEW_DOMAIN", "Message": "Domain is not available", "Details": "Domain is not available"})
    );
    let out = dna.transfer("example.com", "x", 1);
    assert_eq!(out["error"]["Code"], "TRANSFER_DOMAIN");

    // The gateway's own code travels next to API_<status>.
    let body =
        json!({"error": {"code": "Dna.DomainService:Domain:10007", "message": "Domain not found"}})
            .to_string();
    let (mut dna, _t) = client(vec![
        Reply::Http(404, body),
        Reply::Http(500, String::new()),
    ]);
    let out = dna.get_details("missing.com");
    assert_eq!(out["error"]["Code"], "API_404");
    assert_eq!(
        out["error"]["GatewayCode"],
        "Dna.DomainService:Domain:10007"
    );
    // ...and is not carried over to the next failure.
    let out = dna.get_details("example.com");
    assert!(out["error"].get("GatewayCode").is_none(), "{out}");

    // Privacy flags from panels.
    let (mut dna, t) = client(vec![Reply::Http(200, String::new()); 3]);
    assert_eq!(
        dna.modify_privacy_protection_status("example.com", "on", "x")["data"]
            ["PrivacyProtectionStatus"],
        true
    );
    assert_eq!(
        dna.modify_privacy_protection_status("example.com", 0, "x")["data"]
            ["PrivacyProtectionStatus"],
        false
    );
    assert_eq!(
        dna.modify_privacy_protection_status("example.com", true, "x")["data"]
            ["PrivacyProtectionStatus"],
        true
    );
    let body: Value = serde_json::from_str(t.requests()[0].body.as_deref().unwrap()).unwrap();
    assert_eq!(
        body,
        json!({"domainName": "example.com", "privacyStatus": true}),
        "reason is not sent"
    );
}

#[test]
fn query_round_trip_helper() {
    let (path, q) = split_url(&format!("{}/domains?a=1&b=x+y", common::BASE_URL));
    assert_eq!(path, "domains");
    assert_eq!(q["b"], "x y");
}

// ---------------------------------------------------------------------------
// The real transport against a local server
// ---------------------------------------------------------------------------

fn serve_once(
    status_line: &'static str,
    body: &'static str,
) -> (String, std::thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut buf = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let n = stream.read(&mut chunk).unwrap_or(0);
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
            let text = String::from_utf8_lossy(&buf);
            if let Some(head_end) = text.find("\r\n\r\n") {
                let len = text[..head_end]
                    .lines()
                    .find_map(|l| {
                        l.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|v| v.trim().parse::<usize>().unwrap_or(0))
                    })
                    .unwrap_or(0);
                if buf.len() >= head_end + 4 + len {
                    break;
                }
            }
        }
        let response = format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
        String::from_utf8_lossy(&buf).into_owned()
    });
    (format!("http://{addr}/api/v1"), handle)
}

fn real_client(url: &str) -> DomainNameApi {
    let mut dna = DomainNameApi::new(common::RESELLER, common::API_KEY, false);
    dna.set_error_reporting_enabled(false);
    dna.set_service_url(url);
    dna
}

#[test]
fn ureq_transport_sends_headers_and_parses_success() {
    let (url, server) = serve_once(
        "200 OK",
        r#"{"resellerId":"r-1","resellerName":"n","usdBalance":601.64,"tryBalance":1250.5}"#,
    );
    let mut dna = real_client(&url);
    let out = dna.get_reseller_details();
    let raw = server.join().unwrap();
    assert_eq!(out["result"], "OK");
    assert_eq!(out["balance"], "601.64");
    let lower = raw.to_ascii_lowercase();
    assert!(
        raw.starts_with("GET /api/v1/deposit/accounts/me HTTP/1.1"),
        "{raw}"
    );
    assert!(
        lower.contains("user-agent: domainnameapi-rust/1.0.0"),
        "{raw}"
    );
    assert!(lower.contains("x-api-key: conformance-api-key"), "{raw}");
    assert!(
        lower.contains("__reseller: 00000000-0000-4000-8000-000000000001"),
        "{raw}"
    );
}

#[test]
fn ureq_transport_reports_error_statuses_and_redirects() {
    let (url, server) = serve_once(
        "404 Not Found",
        r#"{"error":{"code":"X","message":"The domain name you requested could not be found."}}"#,
    );
    let out = real_client(&url).get_details("missing.com");
    server.join().unwrap();
    assert_eq!(out["error"]["Code"], "API_404");
    assert_eq!(
        out["error"]["Message"],
        "The domain name you requested could not be found."
    );

    let (url, server) = serve_once("302 Found", "");
    let out = real_client(&url).get_reseller_details();
    server.join().unwrap();
    assert_eq!(out["error"]["Code"], "API_302");
    assert_eq!(out["error"]["Message"], "Invalid API credentials");

    let (url, server) = serve_once("200 OK", "");
    let mut dna = real_client(&url);
    let out = dna.renew("example.com", 1);
    let raw = server.join().unwrap();
    assert_eq!(out["error"]["Code"], "DOMAIN_RENEW");
    assert!(
        raw.contains(r#"{"domainName":"example.com","period":1}"#),
        "{raw}"
    );
}

#[test]
fn ureq_transport_refused_connection_and_timeout() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let out = real_client(&format!("http://{addr}/api/v1")).get_details("example.com");
    assert_eq!(out["result"], "ERROR");
    assert!(
        out["error"]["Message"]
            .as_str()
            .unwrap()
            .starts_with("Curl error during request: "),
        "{out}"
    );

    // A server that accepts but never answers: the transport times out.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let hold = std::thread::spawn(move || {
        let conn = listener.accept();
        std::thread::sleep(Duration::from_millis(1500));
        drop(conn);
    });
    let transport = UreqTransport::new();
    let err = transport
        .send(&HttpRequest {
            method: "GET".into(),
            url: format!("http://{addr}/api/v1/domains"),
            headers: vec![],
            body: None,
            timeout: Duration::from_millis(300),
            connect_timeout: Duration::from_millis(300),
            verify_tls: true,
        })
        .unwrap_err();
    hold.join().unwrap();
    assert!(err.is_timeout(), "{err:?}");
}
