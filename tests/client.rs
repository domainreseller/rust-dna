//! Behaviour of the client against a fake API: headers, decoding, retries, read-only mode, errors and paging.

mod common;

use common::{FakeApi, Reply};
use domainnameapi::domains::{ContactHandles, DomainListRequest, DomainRegisterRequest};
use domainnameapi::{codes, Client, Empty, Endpoint, ErrorKind, OperationSafety, Query, Request, Validator};
use futures_util::TryStreamExt;
use serde_json::json;
use std::time::{Duration, Instant};

#[tokio::test]
async fn sends_credentials_and_tracing_headers() {
    let api = FakeApi::start().await;
    api.reply_json(200, json!({"resellerName": "Test", "usdBalance": "10.5", "tryBalance": 1}));

    let account = api.client().account().get().await.unwrap();
    assert_eq!(account.reseller_name, "Test");
    assert_eq!(account.usd_balance, 10.5);

    let request = api.last();
    assert_eq!(request.method, "GET");
    assert_eq!(request.path, "/api/v1/deposit/accounts/me");
    assert_eq!(request.header("x-api-key"), Some("test-key"));
    assert_eq!(request.header("__reseller"), Some("test-reseller"));
    assert_eq!(request.header("accept"), Some("application/json"));
    assert!(request.header("user-agent").unwrap().starts_with("DomainNameAPI-Rust/2."));
    let id = request.header("x-correlation-id").unwrap();
    assert_eq!(id.len(), 32);
    assert!(request.header("content-type").is_none());
    assert!(request.body.is_empty());
}

#[tokio::test]
async fn sends_app_headers_by_default() {
    let api = FakeApi::start().await;
    api.client().account().get().await.unwrap();
    let request = api.last();
    assert_eq!(request.header("x-app"), Some("rust"));
    assert_eq!(request.header("x-version"), Some(domainnameapi::VERSION));
    assert!(request.header("user-agent").unwrap().starts_with("DomainNameAPI-Rust/2."));
}

#[tokio::test]
async fn sends_the_application_set_on_the_builder() {
    let api = FakeApi::start().await;
    let client = api.builder().app("WiseCP5", "V2.0.20-beta").build().unwrap();
    assert_eq!((client.app(), client.app_version()), ("wisecp5", "2.0.20"));
    client.account().get().await.unwrap();
    let request = api.last();
    assert_eq!(request.header("x-app"), Some("wisecp5"));
    assert_eq!(request.header("x-version"), Some("2.0.20"));
    assert!(request.header("user-agent").unwrap().starts_with("DomainNameAPI-Rust/2."), "User-Agent is unchanged");

    let client = api.builder().app("  ", "").build().unwrap();
    client.account().get().await.unwrap();
    let request = api.last();
    assert_eq!(request.header("x-app"), Some("rust"));
    assert_eq!(request.header("x-version"), Some(domainnameapi::VERSION));

    let client = api.builder().app("my panel!", "beta").build().unwrap();
    client.account().get().await.unwrap();
    let request = api.last();
    assert_eq!(request.header("x-app"), Some("mypanel"));
    assert_eq!(request.header("x-version"), Some(domainnameapi::VERSION));
}

#[tokio::test]
async fn app_headers_are_sent_on_every_attempt() {
    let api = FakeApi::start().await;
    api.reply(Reply::text(429, "").header("Retry-After", "0"));
    api.reply(Reply::text(503, ""));
    api.reply_json(200, json!({"domainName": "example.com"}));
    let client = api.builder().app("WHMCS", "8.13.1").build().unwrap();
    client.domains().info("example.com").await.unwrap();
    let requests = api.requests();
    assert_eq!(requests.len(), 3);
    for request in requests {
        assert_eq!(request.header("x-app"), Some("whmcs"));
        assert_eq!(request.header("x-version"), Some("8.13.1"));
    }
}

#[tokio::test]
async fn app_headers_come_from_the_environment() {
    // The only test in this binary that reads or changes the process environment.
    std::env::set_var("DNA_RESELLER_ID", "env-reseller");
    std::env::set_var("DNA_API_KEY", "env-key");
    std::env::set_var("DNA_APP", "HostBill");
    std::env::set_var("DNA_APP_VERSION", "v2024.11.0");
    let api = FakeApi::start().await;
    let client = domainnameapi::ClientBuilder::from_env().unwrap().base_url(api.base_url()).build().unwrap();
    client.account().get().await.unwrap();
    let request = api.last();
    assert_eq!(request.header("x-app"), Some("hostbill"));
    assert_eq!(request.header("x-version"), Some("2024.11.0"));

    std::env::set_var("DNA_APP", "");
    std::env::remove_var("DNA_APP_VERSION");
    let client = domainnameapi::ClientBuilder::from_env().unwrap().base_url(api.base_url()).build().unwrap();
    assert_eq!((client.app(), client.app_version()), ("rust", domainnameapi::VERSION));
    for name in ["DNA_RESELLER_ID", "DNA_API_KEY", "DNA_APP"] {
        std::env::remove_var(name);
    }
}

#[tokio::test]
async fn sends_json_bodies() {
    let api = FakeApi::start().await;
    api.reply_json(200, json!({"success": true, "status": "Active", "expirationDate": "2027-01-01T00:00:00"}));

    let result = api.client().domains().renew(" example.com ", 2).await.unwrap();
    assert!(result.success);
    assert!(result.expiration_date.is_some());

    let request = api.last();
    assert_eq!(request.path, "/api/v1/domains/renew");
    assert_eq!(request.header("content-type"), Some("application/json; charset=utf-8"));
    assert_eq!(request.json(), json!({"domainName": "example.com", "period": 2}));
}

#[tokio::test]
async fn returns_response_details() {
    let api = FakeApi::start().await;
    api.reply(
        Reply::json(200, json!({"domainName": "example.com"}))
            .header("X-Correlation-Id", "abc123")
            .header("X-Quota-Scope", "domain")
            .header("X-Quota-Cost", "1")
            .header("X-Quota-Used", "12.5")
            .header("X-Quota-Limit", "200"),
    );

    let response =
        api.client().send_with_response(&domainnameapi::domains::DomainInfoRequest::new("example.com")).await.unwrap();
    assert_eq!(response.value.domain_name, "example.com");
    assert_eq!(response.status, 200);
    assert_eq!(response.correlation_id, "abc123");
    assert_eq!(response.attempts, 1);
    let quota = response.quota.unwrap();
    assert_eq!(quota.scope, "domain");
    assert_eq!(quota.used, 12.5);
    assert_eq!(quota.limit, 200.0);
}

#[tokio::test]
async fn invalid_requests_are_not_sent() {
    let api = FakeApi::start().await;
    let e = api.client().domains().renew("", 0).await.unwrap_err();
    assert_eq!(e.kind(), ErrorKind::InvalidInput);
    assert_eq!(e.code(), codes::INVALID_INPUT);
    let fields: Vec<&str> = e.validation_errors().iter().map(|f| f.field.as_str()).collect();
    assert_eq!(fields, ["domainName", "years"]);
    assert_eq!(e.operation(), Some("POST /api/v1/domains/renew"));
    assert!(e.hint().is_some());
    assert!(api.requests().is_empty());
}

#[tokio::test]
async fn read_only_blocks_everything_but_reads() {
    let api = FakeApi::start().await;
    let client = api.builder().read_only(true).build().unwrap();

    let e = client.domains().renew("example.com", 1).await.unwrap_err();
    assert_eq!(e.kind(), ErrorKind::ReadOnly);
    assert_eq!(e.code(), codes::READ_ONLY);
    assert_eq!(e.safety(), Some(OperationSafety::Billable));
    let e = client.domains().lock("example.com").await.unwrap_err();
    assert_eq!(e.kind(), ErrorKind::ReadOnly);
    assert!(api.requests().is_empty());

    // Reads go through, even when they use POST or PUT.
    client.domains().search("example.com").await.unwrap();
    client.ssl().status("6f1c0000-0000-4000-8000-000000000001").await.unwrap();
    assert_eq!(api.requests().len(), 2);
}

#[tokio::test]
async fn rate_limits_are_retried_for_every_request() {
    let api = FakeApi::start().await;
    api.reply(Reply::text(429, "").header("Retry-After", "0"));
    api.reply_json(200, json!({"success": true}));

    let response = api
        .client()
        .send_with_response(&DomainRegisterRequest::new(
            "example.com",
            ContactHandles::same("DNA-1"),
            ["ns1.example.net"],
        ))
        .await
        .unwrap();
    assert!(response.value.success);
    assert_eq!(response.attempts, 2);
    assert_eq!(api.requests().len(), 2);
}

#[tokio::test]
async fn rate_limit_errors_after_the_retries() {
    let api = FakeApi::start().await;
    let client = api.builder().max_retries(1).build().unwrap();
    api.reply(Reply::text(429, "").header("Retry-After", "0"));
    api.reply(Reply::text(429, "").header("Retry-After", "0"));

    let e = client.account().get().await.unwrap_err();
    assert_eq!(e.kind(), ErrorKind::RateLimited);
    assert_eq!(e.code(), codes::RATE_LIMITED);
    assert_eq!(e.retry_after(), Some(Duration::ZERO));
    assert!(e.is_retryable());
    assert_eq!(api.requests().len(), 2);
}

#[tokio::test]
async fn long_retry_after_is_not_waited_for() {
    let api = FakeApi::start().await;
    let client = api.builder().max_retry_delay(Duration::from_secs(1)).build().unwrap();
    api.reply(Reply::text(429, "").header("Retry-After", "120"));

    let e = client.account().get().await.unwrap_err();
    assert_eq!(e.retry_after(), Some(Duration::from_secs(120)));
    assert_eq!(api.requests().len(), 1);
}

#[tokio::test]
async fn server_errors_are_retried_only_when_safe() {
    let api = FakeApi::start().await;
    api.reply(Reply::text(503, ""));
    api.reply_json(200, json!({"domainName": "example.com"}));
    api.client().domains().info("example.com").await.unwrap();
    assert_eq!(api.requests().len(), 2);

    // Idempotent writes are retried too.
    api.reply(Reply::text(502, ""));
    api.client().domains().lock("example.com").await.unwrap();
    assert_eq!(api.requests().len(), 4);

    // Billable requests are not.
    api.reply(Reply::text(503, ""));
    let e = api.client().domains().renew("example.com", 1).await.unwrap_err();
    assert_eq!(e.kind(), ErrorKind::Server);
    assert_eq!(e.status(), Some(503));
    assert_eq!(e.code(), "HTTP_503");
    assert_eq!(api.requests().len(), 5);
}

#[tokio::test]
async fn network_errors_are_retried_only_for_reads() {
    let api = FakeApi::start().await;
    api.reply(Reply::hang_up());
    api.reply_json(200, json!({"domainName": "example.com"}));
    api.client().domains().info("example.com").await.unwrap();
    assert_eq!(api.requests().len(), 2);

    api.reply(Reply::hang_up());
    let e = api.client().domains().renew("example.com", 1).await.unwrap_err();
    assert_eq!(e.kind(), ErrorKind::Network);
    assert_eq!(e.code(), codes::NETWORK);
    assert!(e.is_outcome_unknown(), "a billable request that failed on the network may have run");
    assert!(e.is_retryable());
    assert!(e.hint().unwrap().contains("may have reached the API"));
    assert!(std::error::Error::source(&e).is_some());
    assert_eq!(api.requests().len(), 3);
}

#[tokio::test]
async fn timeouts() {
    let api = FakeApi::start().await;
    let client = api.builder().timeout(Duration::from_millis(200)).max_retries(0).build().unwrap();
    api.reply(Reply::text(200, "{}").delay(Duration::from_secs(2)));

    let e = client.account().get().await.unwrap_err();
    assert_eq!(e.kind(), ErrorKind::Timeout);
    assert_eq!(e.code(), codes::TIMEOUT);
    assert!(!e.is_outcome_unknown(), "a read is harmless to repeat");
}

#[tokio::test]
async fn authentication_errors() {
    let api = FakeApi::start().await;
    api.reply(Reply::text(401, "Invalid API key"));
    let e = api.client().account().get().await.unwrap_err();
    assert_eq!(e.kind(), ErrorKind::Auth);
    assert_eq!(e.code(), codes::INVALID_API_KEY);
    assert!(e.hint().unwrap().contains("different API keys"));

    api.reply(Reply::text(401, "API Key is missing"));
    assert_eq!(api.client().account().get().await.unwrap_err().code(), codes::MISSING_API_KEY);

    api.reply(Reply::text(302, "").header("Location", "/Error/Suspend"));
    let e = api.client().account().get().await.unwrap_err();
    assert_eq!(e.kind(), ErrorKind::Auth);
    assert_eq!(e.code(), codes::INVALID_RESELLER);
    assert_eq!(e.status(), Some(302));
}

#[tokio::test]
async fn api_errors_are_normalised() {
    let api = FakeApi::start().await;
    api.reply(
        Reply::json(
            400,
            json!({"error": {
                "code": "Dna.DomainService:Domain:1018",
                "message": "Exception of type 'Volo.Abp.BusinessException' was thrown.",
                "details": "Dna Invalid forward type",
                "data": {},
                "validationErrors": [{"message": "The field is invalid", "members": ["$.forwardType"]}, "plain"]
            }}),
        )
        .header("X-Correlation-Id", "corr-1"),
    );

    let e = api.client().domains().unlock("example.com").await.unwrap_err();
    assert_eq!(e.kind(), ErrorKind::Validation);
    assert_eq!(e.code(), "Dna.DomainService:Domain:1018");
    assert_eq!(e.message(), "Invalid forward type");
    assert_eq!(e.details(), Some("Dna Invalid forward type"));
    assert_eq!(e.correlation_id(), Some("corr-1"));
    assert_eq!(e.validation_errors()[0].field, "forwardType");
    assert_eq!(e.validation_errors()[1].message, "plain");
    assert!(e.response_body().unwrap().contains("1018"));
    assert_eq!(e.hint(), Some("Fix the listed fields and send the request again."));
    assert!(e.to_string().starts_with("[Dna.DomainService:Domain:1018] Invalid forward type (HTTP 400)"));
}

#[tokio::test]
async fn not_found_and_server_errors_have_hints() {
    let api = FakeApi::start().await;
    api.reply_json(404, json!({"error": {"code": codes::DOMAIN_NOT_FOUND, "message": "Domain not found"}}));
    let e = api.client().domains().info("nope.com").await.unwrap_err();
    assert!(e.is_not_found());
    assert!(e.hint().unwrap().contains("environment"));

    let client = api.builder().max_retries(0).build().unwrap();
    api.reply(Reply::text(500, "<html>oops</html>").header("X-Correlation-Id", "c-9"));
    let e = client.domains().info("example.com").await.unwrap_err();
    assert_eq!(e.kind(), ErrorKind::Server);
    assert_eq!(e.message(), "HTTP 500 Internal Server Error.");
    assert!(e.hint().unwrap().contains("c-9"));

    api.reply_json(500, json!({"error": {"code": codes::RENEWED_NOT_RECORDED, "message": "renewed"}}));
    let e = client.domains().renew("example.com", 1).await.unwrap_err();
    assert!(e.is_outcome_unknown());
    assert!(!e.is_retryable());

    api.reply_json(402, json!({"error": {"code": codes::INSUFFICIENT_BALANCE, "message": "Insufficient balance"}}));
    assert!(client.domains().renew("example.com", 1).await.unwrap_err().is_insufficient_balance());
}

#[tokio::test]
async fn unreadable_answers() {
    let api = FakeApi::start().await;
    api.reply(Reply::text(200, ""));
    let e = api.client().account().get().await.unwrap_err();
    assert_eq!(e.code(), codes::EMPTY_RESPONSE);
    assert_eq!(e.kind(), ErrorKind::Unknown);

    api.reply(Reply::text(200, "[1,2]"));
    let e = api.client().account().get().await.unwrap_err();
    assert_eq!(e.code(), codes::INVALID_RESPONSE);

    // Operations without an answer accept an empty body, or any body.
    api.reply(Reply::text(200, ""));
    api.client().domains().lock("example.com").await.unwrap();
    api.reply(Reply::text(200, "true"));
    api.client().domains().unlock("example.com").await.unwrap();
}

#[tokio::test]
async fn success_false_is_returned_as_it_is() {
    let api = FakeApi::start().await;
    api.reply_json(200, json!({"success": false, "message": "The registry refused the domain"}));
    let result = api
        .client()
        .domains()
        .register(DomainRegisterRequest::new("example.com", ContactHandles::same("DNA-1"), ["ns1.example.net"]))
        .await
        .unwrap();
    assert!(!result.success);
    assert_eq!(result.message, "The registry refused the domain");
}

#[tokio::test]
async fn list_all_fetches_every_page() {
    let api = FakeApi::start().await;
    api.reply_json(200, json!({"items": [{"domainName": "a.com"}, {"domainName": "b.com"}], "totalCount": 5}));
    api.reply_json(200, json!({"items": [{"domainName": "c.com"}, {"domainName": "d.com"}], "totalCount": 5}));
    api.reply_json(200, json!({"items": [{"domainName": "e.com"}], "totalCount": 5}));

    let client = api.client();
    let domains: Vec<_> =
        client.domains().list_all(DomainListRequest::new().domain_name("x").limit(2)).try_collect().await.unwrap();
    let names: Vec<&str> = domains.iter().map(|d| d.domain_name.as_str()).collect();
    assert_eq!(names, ["a.com", "b.com", "c.com", "d.com", "e.com"]);

    let skips: Vec<String> = api.requests().iter().map(|r| r.query("SkipCount").unwrap().to_string()).collect();
    assert_eq!(skips, ["0", "2", "4"]);
    assert!(api
        .requests()
        .iter()
        .all(|r| r.query("MaxResultCount") == Some("2") && r.query("DomainName") == Some("x")));
}

#[tokio::test]
async fn list_all_stops_on_an_empty_page_and_on_errors() {
    let api = FakeApi::start().await;
    api.reply_json(200, json!({"items": [{"domainName": "a.com"}]}));
    let client = api.client();
    let all: Vec<_> = client.domains().list_all(DomainListRequest::new()).try_collect().await.unwrap();
    assert_eq!(all.len(), 1, "a short page without a total ends the list");
    assert_eq!(api.last().query("MaxResultCount"), Some("100"));

    api.reply_json(200, json!({"items": [{"domainName": "a.com"}], "totalCount": 10}));
    api.reply_json(404, json!({"error": {"message": "gone"}}));
    let mut stream = client.domains().list_all(DomainListRequest::new().limit(1));
    assert_eq!(stream.try_next().await.unwrap().unwrap().domain_name, "a.com");
    assert!(stream.try_next().await.unwrap_err().is_not_found());
}

#[tokio::test]
async fn forwarding_that_is_not_set_is_none() {
    let api = FakeApi::start().await;
    api.reply_json(
        404,
        json!({"error": {"code": codes::FORWARDING_NOT_FOUND, "message": "Redirect information not found"}}),
    );
    assert!(api.client().domains().forwarding().get("example.com").await.unwrap().is_none());

    api.reply_json(200, json!({"redirectAlias": "https://example.org", "redirectType": "Frame"}));
    let forwarding = api.client().domains().forwarding().get("example.com").await.unwrap().unwrap();
    assert_eq!(forwarding.url, "https://example.org");
    assert_eq!(forwarding.forward_type, domainnameapi::domains::ForwardType::Frame);
}

/// A request for an endpoint the SDK does not cover, as a user would write it.
struct DomainNotesRequest {
    domain_name: String,
}

#[derive(Debug, serde::Deserialize)]
struct DomainNotes {
    notes: Vec<String>,
}

impl Request for DomainNotesRequest {
    type Response = DomainNotes;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("api/v1/domains/notes")
    }

    fn query(&self) -> Query {
        Query::new().set_str("DomainName", &self.domain_name)
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
    }
}

struct PurgeRequest;

impl Request for PurgeRequest {
    type Response = Empty;

    fn endpoint(&self) -> Endpoint {
        Endpoint::delete("/api/v1/domains/purge", OperationSafety::Destructive)
    }
}

#[tokio::test]
async fn custom_requests_get_the_same_treatment() {
    let api = FakeApi::start().await;
    api.reply_json(200, json!({"notes": ["a", "b"]}));
    let client = api.client();
    let notes = client.send(&DomainNotesRequest { domain_name: "example.com".into() }).await.unwrap();
    assert_eq!(notes.notes, ["a", "b"]);
    assert_eq!(api.last().path, "/api/v1/domains/notes");
    assert_eq!(api.last().query("DomainName"), Some("example.com"));

    let e = client.send(&DomainNotesRequest { domain_name: " ".into() }).await.unwrap_err();
    assert_eq!(e.kind(), ErrorKind::InvalidInput);

    // Destructive custom requests are not retried after a server error.
    api.reply(Reply::text(503, ""));
    assert_eq!(client.send(&PurgeRequest).await.unwrap_err().status(), Some(503));
    assert_eq!(api.requests().len(), 2);
}

#[tokio::test]
async fn base_url_path_prefix_is_kept() {
    let api = FakeApi::start().await;
    let client = Client::builder()
        .credentials("id", "key")
        .base_url(format!("{}/proxy/", api.base_url()))
        .pace_requests(false)
        .build()
        .unwrap();
    client.domains().info("example.com").await.unwrap();
    assert_eq!(api.last().path, "/proxy/api/v1/domains/info");
}

#[tokio::test]
async fn query_values_are_encoded() {
    let api = FakeApi::start().await;
    api.client()
        .domains()
        .list(DomainListRequest::new().domain_name("a b&c").sort("expirationDate desc"))
        .await
        .unwrap();
    let request = api.last();
    assert_eq!(request.query("DomainName"), Some("a b&c"));
    assert_eq!(request.query("Sorting"), Some("expirationDate desc"));
}

#[tokio::test]
async fn searches_are_paced() {
    let api = FakeApi::start().await;
    let client = Client::builder().credentials("id", "key").base_url(api.base_url()).build().unwrap();
    let started = Instant::now();
    client.domains().search("a.com").await.unwrap();
    client.domains().search("b.com").await.unwrap();
    client.domains().search("c.com").await.unwrap();
    assert!(started.elapsed() >= Duration::from_millis(950), "3 searches at 2/second take at least 1 second");

    // Domain info is not rate limited.
    let started = Instant::now();
    for _ in 0..3 {
        client.domains().info("example.com").await.unwrap();
    }
    assert!(started.elapsed() < Duration::from_millis(500));
}

#[tokio::test]
async fn clients_are_shared_between_tasks() {
    let api = FakeApi::start().await;
    let client = api.client();
    let tasks: Vec<_> = (0..5)
        .map(|i| {
            let client = client.clone();
            tokio::spawn(async move { client.domains().info(&format!("d{i}.com")).await })
        })
        .collect();
    for task in tasks {
        task.await.unwrap().unwrap();
    }
    assert_eq!(api.requests().len(), 5);
}
