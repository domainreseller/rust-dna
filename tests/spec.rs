//! Checks every request type against the API's OpenAPI document in `tests/testdata/openapi.json` (the document the
//! `dna` CLI embeds) and against the CLI's operation registry:
//!
//! - every operation of the document has exactly one request type, with the CLI's method, path, safety class and
//!   idempotency;
//! - query parameters and body properties have the exact names the document uses, nested objects included, and the
//!   required ones are sent;
//! - the JSON names of the answer types exist in the document's response schemas.
//!
//! When the API gains an operation, update the document from the CLI repository; this test then fails until the
//! request type exists.

use domainnameapi::account::*;
use domainnameapi::contacts::*;
use domainnameapi::dns::*;
use domainnameapi::dnssec::*;
use domainnameapi::domains::*;
use domainnameapi::hosting::*;
use domainnameapi::orders::*;
use domainnameapi::products::*;
use domainnameapi::ssl::*;
use domainnameapi::{Endpoint, Method, OperationSafety, Request};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

use OperationSafety::{Billable, Destructive, Read, Write};

/// The CLI's registry (`internal/ops/registry.go`): id, method, path, safety, idempotent.
const CLI_OPERATIONS: &[(&str, &str, &str, OperationSafety, bool)] = &[
    ("account.get", "GET", "/api/v1/deposit/accounts/me", Read, false),
    ("account.profile", "GET", "/api/v1/resellers/me", Read, false),
    ("account.transactions", "GET", "/api/v1/deposit/transactions", Read, false),
    ("contact.get", "GET", "/api/v1/contacts/{handleCode}", Read, false),
    ("contact.create", "POST", "/api/v1/contacts", Write, false),
    ("contact.update", "PUT", "/api/v1/contacts/{handleCode}", Write, false),
    ("contact.verification", "GET", "/api/v1/contacts/verification/check", Read, false),
    ("contact.resend-verification", "POST", "/api/v1/contacts/verification/request", Write, false),
    ("domain.search", "POST", "/api/v1/domains/search", Read, false),
    ("domain.bulk-search", "POST", "/api/v1/domains/bulk-search", Read, false),
    ("domain.list", "GET", "/api/v1/domains", Read, false),
    ("domain.info", "GET", "/api/v1/domains/info", Read, false),
    ("domain.register", "POST", "/api/v1/domains/register", Billable, false),
    ("domain.register-with-contacts", "POST", "/api/v1/domains/register-with-contacts", Billable, false),
    ("domain.renew", "POST", "/api/v1/domains/renew", Billable, false),
    ("domain.renew-check", "POST", "/api/v1/domains/renew/check", Read, false),
    ("domain.transfer", "POST", "/api/v1/domains/transfer", Billable, false),
    ("domain.transfer-check", "POST", "/api/v1/domains/transfers/check", Read, false),
    ("domain.transfer-status", "POST", "/api/v1/domains/transfers/query", Read, false),
    ("domain.restore", "POST", "/api/v1/domains/restore", Billable, false),
    ("domain.lock", "POST", "/api/v1/domains/lock", Write, true),
    ("domain.unlock", "POST", "/api/v1/domains/unlock", Write, true),
    ("domain.privacy", "POST", "/api/v1/domains/privacy", Write, true),
    ("domain.update-auth-code", "PUT", "/api/v1/domains/auth", Write, false),
    ("domain.contacts.update", "PUT", "/api/v1/domains/contacts/update", Write, false),
    ("domain.contacts.set-handles", "PUT", "/api/v1/domains/contacts/update-with-handle", Write, false),
    ("domain.documents.list", "GET", "/api/v1/domains/documents", Read, false),
    ("domain.documents.upload", "POST", "/api/v1/domains/documents", Write, false),
    ("domain.forward.get", "GET", "/api/v1/domains/forwards", Read, false),
    ("domain.forward.set", "POST", "/api/v1/domains/forwards", Write, false),
    ("domain.forward.delete", "DELETE", "/api/v1/domains/forwards", Destructive, false),
    ("domain.nameservers.set", "PUT", "/api/v1/domains/dns/name-server", Write, true),
    ("domain.host.add", "POST", "/api/v1/domains/dns/host", Write, false),
    ("domain.host.update", "PUT", "/api/v1/domains/dns/host", Write, false),
    ("domain.host.delete", "DELETE", "/api/v1/domains/dns/host", Destructive, false),
    ("dns.list", "GET", "/api/v1/domains/zones", Read, false),
    ("dns.add", "POST", "/api/v1/domains/zones", Write, false),
    ("dns.update", "PUT", "/api/v1/domains/zones", Write, true),
    ("dns.delete", "DELETE", "/api/v1/domains/zones", Destructive, false),
    ("dns.apply", "POST", "/api/v1/domains/zones/apply", Write, true),
    ("dnssec.get", "GET", "/api/v1/domains/dns/dnssec", Read, false),
    ("dnssec.add", "POST", "/api/v1/domains/dns/dnssec", Write, false),
    ("dnssec.remove", "DELETE", "/api/v1/domains/dns/dnssec", Destructive, false),
    ("dnssec.disable", "POST", "/api/v1/domains/dns/dnssec/disable", Destructive, false),
    ("ssl.list", "GET", "/api/v1/ssls", Read, false),
    ("ssl.generate-csr", "POST", "/api/v1/ssls/generate-csr", Read, false),
    ("ssl.check-csr", "POST", "/api/v1/ssls/check-csr", Read, false),
    ("ssl.initialize", "POST", "/api/v1/ssls/initialize", Billable, false),
    ("ssl.renew", "POST", "/api/v1/ssls/renew", Billable, false),
    ("ssl.configure", "PUT", "/api/v1/ssls/configure", Write, false),
    ("ssl.status", "PUT", "/api/v1/ssls/check-status", Read, true),
    ("ssl.get", "GET", "/api/v1/ssls/certificate", Read, false),
    ("ssl.content", "GET", "/api/v1/ssls/certificate-content", Read, false),
    ("ssl.reissue", "POST", "/api/v1/ssls/reissue", Write, false),
    ("ssl.approvers", "GET", "/api/v1/ssls/approvers", Read, false),
    ("ssl.revoke", "DELETE", "/api/v1/ssls/revoke", Destructive, false),
    ("hosting.packages", "GET", "/api/v1/hostings/packages", Read, false),
    ("hosting.plans", "GET", "/api/v1/hostings/plans", Read, false),
    ("hosting.create", "POST", "/api/v1/hostings/create", Billable, false),
    ("hosting.get", "GET", "/api/v1/hostings", Read, false),
    ("hosting.get-by-token", "GET", "/api/v1/hostings/by-token", Read, false),
    ("hosting.status", "GET", "/api/v1/hostings/check-status", Read, false),
    ("order.get", "GET", "/api/v1/order/{orderCode}", Read, false),
    ("product.list", "GET", "/api/v1/products", Read, false),
    ("product.info", "GET", "/api/v1/products/info", Read, false),
    ("product.tlds", "GET", "/api/v1/products/tlds", Read, false),
    ("product.ssls", "GET", "/api/v1/products/ssls", Read, false),
];

const CERTIFICATE_ID: &str = "6f1c0000-0000-4000-8000-000000000001";

/// What the test needs to know about one request: its endpoint, query, body and the JSON keys of its answer type.
struct Case {
    id: &'static str,
    name: String,
    endpoint: Endpoint,
    query: Vec<(String, String)>,
    body: Option<Value>,
    response: Value,
    valid: bool,
}

fn case<R>(id: &'static str, request: R) -> Case
where
    R: Request,
    R::Response: serde::Serialize + Default,
{
    let mut v = domainnameapi::Validator::new();
    request.validate(&mut v);
    Case {
        id,
        name: std::any::type_name::<R>().rsplit("::").next().unwrap().to_string(),
        endpoint: request.endpoint(),
        query: request.query().pairs().to_vec(),
        body: request.body(),
        response: serde_json::to_value(R::Response::default()).unwrap(),
        valid: v.is_valid() || panic!("{} is not valid: {:?}", std::any::type_name::<R>(), v.errors()),
    }
}

fn contact() -> Contact {
    Contact {
        first_name: "Ada".into(),
        last_name: "Lovelace".into(),
        company_name: "Example Ltd".into(),
        email: "ada@example.com".into(),
        address: "1 Example Street".into(),
        city: "Istanbul".into(),
        state: "Istanbul".into(),
        postal_code: "34000".into(),
        country: "TR".into(),
        phone_country_code: "90".into(),
        phone: "5551234567".into(),
        fax_country_code: "90".into(),
        fax: "5551234568".into(),
        disclose: Some(false),
        ..Default::default()
    }
}

fn ssl_contact() -> SslContact {
    SslContact {
        title: "Ms".into(),
        first_name: "Ada".into(),
        last_name: "Lovelace".into(),
        company_name: "Example Ltd".into(),
        email: "ada@example.com".into(),
        address: "1 Example Street".into(),
        city: "Istanbul".into(),
        state: "Istanbul".into(),
        postal_code: "34000".into(),
        country: "TR".into(),
        phone_country_code: "90".into(),
        phone: "5551234567".into(),
        fax_country_code: "90".into(),
        fax: "5551234568".into(),
        ..Default::default()
    }
}

/// One request per operation, with every optional value set.
fn cases() -> Vec<Case> {
    let from = chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z").unwrap().to_utc();
    let ds = DsData::new(2371, DnssecAlgorithm::EcdsaP256Sha256, DsDigestType::Sha256, "1F98");
    vec![
        case("account.get", AccountGetRequest::new()),
        case("account.profile", AccountProfileRequest::new()),
        case(
            "account.transactions",
            AccountTransactionsRequest::new()
                .transaction_types([TransactionType::Register, TransactionType::Renew])
                .product_types([ProductType::Domain])
                .charge_type(ChargeType::Subtract)
                .currencies(["USD"])
                .start_date(from)
                .end_date(from + chrono::Duration::days(30))
                .trace_id("abc")
                .description("renew")
                .filter_text("example")
                .amount(9.99)
                .skip(10)
                .limit(20)
                .sort("creationTime desc"),
        ),
        case("contact.get", ContactGetRequest::new("DNA-1")),
        case("contact.create", ContactCreateRequest::new(contact())),
        case("contact.update", ContactUpdateRequest::new("DNA-1", contact())),
        case("contact.verification", ContactVerificationRequest::new("DNA-1")),
        case("contact.resend-verification", ContactResendVerificationRequest::new("DNA-1")),
        case("domain.search", DomainSearchRequest::new("example.com").currency("USD")),
        case("domain.bulk-search", DomainBulkSearchRequest::new(["example.com", "example.net"]).currency("USD")),
        case(
            "domain.list",
            DomainListRequest::new()
                .domain_name("example")
                .statuses([DomainStatus::Active, DomainStatus::PendingDelete])
                .skip(5)
                .limit(50)
                .sort("expirationDate"),
        ),
        case("domain.info", DomainInfoRequest::new("example.com")),
        case(
            "domain.register",
            DomainRegisterRequest::new(
                "example.com",
                ContactHandles::same("DNA-1"),
                ["ns1.example.net", "ns2.example.net"],
            )
            .years(2)
            .tld_attribute("x", "y"),
        ),
        case(
            "domain.register-with-contacts",
            DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact()), ["ns1.example.net"])
                .years(1)
                .tld_attribute("x", "y")
                .use_trustee_contact(true),
        ),
        case("domain.renew", DomainRenewRequest::new("example.com", 1)),
        case("domain.renew-check", DomainRenewCheckRequest::new("example.com").years(1)),
        case("domain.transfer", DomainTransferRequest::new("example.com", "Xy7!kP2q").years(1)),
        case("domain.transfer-check", DomainTransferCheckRequest::new("example.com", "Xy7!kP2q")),
        case("domain.transfer-status", DomainTransferStatusRequest::new("example.com")),
        case("domain.restore", DomainRestoreRequest::new("example.com")),
        case("domain.lock", DomainLockRequest::new("example.com")),
        case("domain.unlock", DomainUnlockRequest::new("example.com")),
        case("domain.privacy", DomainPrivacyRequest::new("example.com", true)),
        case("domain.update-auth-code", DomainUpdateAuthCodeRequest::new("example.com")),
        case(
            "domain.contacts.update",
            DomainContactsUpdateRequest::new("example.com", DomainContacts::same(contact())),
        ),
        case(
            "domain.contacts.set-handles",
            DomainContactsSetHandlesRequest::new("example.com", ContactHandles::same("DNA-1")),
        ),
        case(
            "domain.documents.list",
            DomainDocumentsListRequest::new("example.com.tr")
                .document_types([DocumentType::IdentityCard])
                .skip(1)
                .limit(10)
                .sort("uploadedDate"),
        ),
        case(
            "domain.documents.upload",
            DomainDocumentsUploadRequest::new(
                "example.com.tr",
                [DocumentUpload::new(DocumentType::IdentityCard, "id.pdf", b"%PDF-1.4".to_vec())],
            ),
        ),
        case("domain.forward.get", DomainForwardGetRequest::new("example.com")),
        case(
            "domain.forward.set",
            DomainForwardSetRequest::new("example.com", "https://www.example.org", ForwardType::Frame),
        ),
        case("domain.forward.delete", DomainForwardDeleteRequest::new("example.com")),
        case(
            "domain.nameservers.set",
            DomainNameServersSetRequest::new("example.com", ["ns1.example.net"]).dnssec_mode(DnssecMode::KeepDnssec),
        ),
        case(
            "domain.host.add",
            DomainHostAddRequest::new("example.com", "ns1.example.com", ["192.0.2.1", "2001:db8::1"]),
        ),
        case(
            "domain.host.update",
            DomainHostUpdateRequest::new("example.com", "ns1.example.com", ["192.0.2.2"])
                .new_host_name("ns3.example.com"),
        ),
        case("domain.host.delete", DomainHostDeleteRequest::new("example.com", "ns1.example.com")),
        case("dns.list", DnsListRequest::new("example.com")),
        case("dns.add", DnsAddRequest::new("example.com", "www", "a", ["192.0.2.1"]).ttl(600)),
        case(
            "dns.update",
            DnsUpdateRequest::new("example.com", "www", RecordType::A, ["192.0.2.2"]).new_name("web").ttl(600),
        ),
        case("dns.delete", DnsDeleteRequest::new("example.com", "www", RecordType::A, "192.0.2.1")),
        case("dns.apply", DnsApplyRequest::new("example.com")),
        case("dnssec.get", DnssecGetRequest::new("example.com")),
        case("dnssec.add", DnssecAddRequest::new("example.com", [ds.clone()])),
        case("dnssec.remove", DnssecRemoveRequest::new("example.com", ds)),
        case("dnssec.disable", DnssecDisableRequest::new("example.com")),
        case("ssl.list", SslListRequest::new().domain_name("example.com").skip(5).limit(50)),
        case(
            "ssl.generate-csr",
            SslGenerateCsrRequest::new("example.com")
                .country("TR")
                .state("Istanbul")
                .locality("Istanbul")
                .organization("Example Ltd")
                .organization_unit("IT")
                .alternative_names(["www.example.com"]),
        ),
        case("ssl.check-csr", SslCheckCsrRequest::new("-----BEGIN CERTIFICATE REQUEST-----")),
        case("ssl.initialize", SslInitializeRequest::new("example.com", "PositiveSSL")),
        case("ssl.renew", SslRenewRequest::new(CERTIFICATE_ID)),
        case(
            "ssl.configure",
            SslConfigureRequest::new(CERTIFICATE_ID, "csr", ssl_contact())
                .private_key("key")
                .approver_mail("admin@example.com")
                .authentication_type(AuthenticationType::Email)
                .web_server_type(WebServerType::Apache)
                .san_list(["a.example.com"])
                .wildcard_list(["*.example.com"]),
        ),
        case("ssl.status", SslStatusRequest::new(CERTIFICATE_ID)),
        case("ssl.get", SslGetRequest::new(CERTIFICATE_ID)),
        case("ssl.content", SslContentRequest::new(CERTIFICATE_ID)),
        case(
            "ssl.reissue",
            SslReissueRequest::new(CERTIFICATE_ID).approver_mail("admin@example.com").csr("csr").private_key("key"),
        ),
        case("ssl.approvers", SslApproversRequest::for_certificate(CERTIFICATE_ID)),
        case("ssl.revoke", SslRevokeRequest::new(CERTIFICATE_ID)),
        case("hosting.packages", HostingPackagesRequest::new()),
        case("hosting.plans", HostingPlansRequest::new().hosting_id("6f1c0000-0000-4000-8000-000000000002")),
        case("hosting.create", HostingCreateRequest::new("example.com", "Linux-Starter")),
        case("hosting.get", HostingGetRequest::by_service_id("HST-1")),
        case("hosting.get-by-token", HostingGetByTokenRequest::new("token")),
        case("hosting.status", HostingStatusRequest::by_domain("example.com")),
        case("order.get", OrderGetRequest::new("ORD-1")),
        case(
            "product.list",
            ProductListRequest::new()
                .filter_text("com")
                .product_type(ProductType::Domain)
                .skip(1)
                .limit(10)
                .sort("productName"),
        ),
        case("product.info", ProductInfoRequest::new("com", OrderType::Renew, 2)),
        case(
            "product.tlds",
            ProductTldsRequest::new().filter_text("com.tr").currency("USD").skip(1).limit(10).sort("name"),
        ),
        case(
            "product.ssls",
            ProductSslsRequest::new()
                .filter_text("ssl")
                .vendor_name("Sectigo")
                .currency("USD")
                .validation_type(1)
                .is_wildcard(true)
                .is_multi_domain(false)
                .skip(1)
                .limit(10)
                .sort("name"),
        ),
    ]
}

// ------------------------------------------------------------------------------------------------ the document

struct Spec {
    doc: Value,
}

impl Spec {
    fn load() -> Spec {
        let text =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/testdata/openapi.json")).unwrap();
        Spec { doc: serde_json::from_str(&text).unwrap() }
    }

    /// Every (method, path) of the document.
    fn operations(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for (path, item) in self.doc["paths"].as_object().unwrap() {
            for method in item.as_object().unwrap().keys() {
                out.push((method.to_ascii_uppercase(), path.clone()));
            }
        }
        out
    }

    fn operation(&self, method: &str, path: &str) -> &Value {
        &self.doc["paths"][path][method.to_ascii_lowercase()]
    }

    /// Follows `$ref` and single-element `allOf`/`oneOf`/`anyOf`.
    fn resolve<'a>(&'a self, mut schema: &'a Value) -> &'a Value {
        loop {
            if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
                let name = reference.trim_start_matches("#/components/schemas/");
                schema = &self.doc["components"]["schemas"][name];
                continue;
            }
            let single = ["allOf", "oneOf", "anyOf"]
                .iter()
                .find_map(|k| schema.get(*k).and_then(Value::as_array).filter(|a| a.len() == 1));
            match single {
                Some(list) => schema = &list[0],
                None => return schema,
            }
        }
    }
}

/// Finds the template path (`/api/v1/contacts/{handleCode}`) a concrete path belongs to.
fn template_matches(template: &str, path: &str) -> bool {
    let t: Vec<&str> = template.split('/').collect();
    let p: Vec<&str> = path.split('/').collect();
    t.len() == p.len() && t.iter().zip(&p).all(|(a, b)| a == b || (a.starts_with('{') && !b.is_empty()))
}

/// Checks the keys of a JSON value against a schema, recursively. `request` also checks required properties.
fn check_schema(spec: &Spec, schema: &Value, value: &Value, at: &str, request: bool, problems: &mut Vec<String>) {
    let schema = spec.resolve(schema);
    match value {
        Value::Object(map) => check_object(spec, schema, map, at, request, problems),
        Value::Array(items) => {
            if let Some(item_schema) = schema.get("items") {
                for (i, item) in items.iter().enumerate() {
                    check_schema(spec, item_schema, item, &format!("{at}[{i}]"), request, problems);
                }
            }
        }
        _ => {}
    }
}

fn check_object(
    spec: &Spec,
    schema: &Value,
    map: &Map<String, Value>,
    at: &str,
    request: bool,
    problems: &mut Vec<String>,
) {
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        // A free-form object, such as tldAttributes.
        return;
    };
    for (key, value) in map {
        match properties.get(key) {
            Some(property) => check_schema(spec, property, value, &format!("{at}.{key}"), request, problems),
            None => problems.push(format!("{at}.{key} is not in the document")),
        }
    }
    if request {
        for required in schema.get("required").and_then(Value::as_array).into_iter().flatten() {
            let name = required.as_str().unwrap();
            if !map.contains_key(name) {
                problems.push(format!("{at}.{name} is required but not sent"));
            }
        }
    }
}

// ------------------------------------------------------------------------------------------------ tests

#[test]
fn every_operation_has_one_request_type() {
    let spec = Spec::load();
    let cases = cases();
    assert_eq!(CLI_OPERATIONS.len(), 67, "the CLI registry has 67 operations");
    assert_eq!(spec.operations().len(), CLI_OPERATIONS.len(), "the document and the CLI registry differ");

    let mut problems = Vec::new();
    for (method, path) in spec.operations() {
        let matching: Vec<&Case> = cases
            .iter()
            .filter(|c| c.endpoint.method.as_str() == method && template_matches(&path, &c.endpoint.path))
            .collect();
        match matching.len() {
            1 => {}
            0 => problems.push(format!("{method} {path} has no request type")),
            _ => problems.push(format!(
                "{method} {path} has {} request types: {:?}",
                matching.len(),
                matching.iter().map(|c| &c.name).collect::<Vec<_>>()
            )),
        }
    }
    let ids: BTreeSet<&str> = cases.iter().map(|c| c.id).collect();
    assert_eq!(ids.len(), cases.len(), "an operation appears twice in cases()");
    assert!(problems.is_empty(), "{problems:#?}");
}

#[test]
fn requests_match_the_cli_registry() {
    let mut problems = Vec::new();
    for case in cases() {
        let Some(&(_, method, path, safety, idempotent)) = CLI_OPERATIONS.iter().find(|op| op.0 == case.id) else {
            problems.push(format!("{} is not a CLI operation", case.id));
            continue;
        };
        let e = &case.endpoint;
        if e.method.as_str() != method || !template_matches(path, &e.path) {
            problems.push(format!("{}: {e} should be {method} {path}", case.name));
        }
        if e.safety != safety {
            problems.push(format!("{}: safety {} should be {}", case.name, e.safety, safety));
        }
        if e.idempotent != idempotent {
            problems.push(format!("{}: idempotent {} should be {}", case.name, e.idempotent, idempotent));
        }
        if e.safety == Billable && e.is_retry_safe() {
            problems.push(format!("{}: a billable request must never be retried", case.name));
        }
        if e.method == Method::Get && e.safety != Read {
            problems.push(format!("{}: GET must be a read", case.name));
        }
        assert!(case.valid);
    }
    assert!(problems.is_empty(), "{problems:#?}");
}

#[test]
fn requests_send_what_the_document_describes() {
    let spec = Spec::load();
    let mut problems = Vec::new();
    for case in cases() {
        let template = spec
            .operations()
            .into_iter()
            .find(|(m, p)| m == case.endpoint.method.as_str() && template_matches(p, &case.endpoint.path))
            .map(|(_, p)| p)
            .unwrap();
        let op = spec.operation(case.endpoint.method.as_str(), &template);
        let name = &case.name;

        // Query parameters
        let parameters: Vec<&Value> = op["parameters"].as_array().into_iter().flatten().collect();
        let query_names: BTreeSet<&str> =
            parameters.iter().filter(|p| p["in"] == "query").filter_map(|p| p["name"].as_str()).collect();
        for (key, _) in &case.query {
            if !query_names.contains(key.as_str()) {
                problems.push(format!("{name}: query parameter {key} is not in the document"));
            }
        }
        for p in parameters.iter().filter(|p| p["in"] == "query" && p["required"] == true) {
            let required = p["name"].as_str().unwrap();
            if !case.query.iter().any(|(k, _)| k == required) {
                problems.push(format!("{name}: required query parameter {required} is not sent"));
            }
        }

        // Body
        let body_schema = op["requestBody"]["content"]
            .as_object()
            .and_then(|content| content.values().next())
            .map(|media| &media["schema"]);
        match (&case.body, body_schema) {
            (Some(body), Some(schema)) => check_schema(&spec, schema, body, "body", true, &mut problems),
            (Some(_), None) => problems.push(format!("{name}: sends a body, the document has none")),
            (None, Some(_)) => problems.push(format!("{name}: sends no body, the document has one")),
            (None, None) => {}
        }

        // Answer
        let response_schema = ["200", "202"]
            .iter()
            .find_map(|code| op["responses"][code]["content"].as_object())
            .and_then(|content| content.values().next())
            .map(|media| &media["schema"]);
        match (response_schema, &case.response) {
            (Some(schema), response) => {
                let mut found = Vec::new();
                check_schema(&spec, schema, response, "answer", false, &mut found);
                problems.extend(found.into_iter().map(|p| format!("{name}: {p}")));
            }
            (None, Value::Null) => {}
            (None, other) => problems.push(format!("{name}: the document has no answer, the type has {other}")),
        }
    }
    assert!(problems.is_empty(), "{problems:#?}");
}

#[test]
fn integer_enums_match_the_document() {
    fn codes<E: Copy>(all: &[E], code: fn(E) -> i64) -> Vec<i64> {
        all.iter().map(|v| code(*v)).collect()
    }
    let spec = Spec::load();
    let schemas = &spec.doc["components"]["schemas"];
    let cases: Vec<(&str, &str, Vec<i64>)> = vec![
        (
            "Dna.DepositService.Accounts.AccountTransactionDto",
            "type",
            codes(TransactionType::ALL, TransactionType::code),
        ),
        (
            "Dna.DepositService.Accounts.AccountTransactionDto",
            "status",
            codes(TransactionStatus::ALL, TransactionStatus::code),
        ),
        ("Dna.DepositService.Accounts.AccountTransactionDto", "chargeType", codes(ChargeType::ALL, ChargeType::code)),
        (
            "Dna.DepositService.Accounts.AccountTransactionDto",
            "productType",
            codes(ProductType::ALL, ProductType::code),
        ),
        ("Dna.DomainService.Dns.DnsEditNameServerInput", "dnssecMode", codes(DnssecMode::ALL, DnssecMode::code)),
        ("Dna.DomainService.Dnssec.DnssecInfoBaseDto", "status", codes(DnssecStatus::ALL, DnssecStatus::code)),
        (
            "Dna.DomainService.Dnssec.DnssecOperationResultBaseDto",
            "operationType",
            codes(DnssecOperationType::ALL, DnssecOperationType::code),
        ),
        ("Dna.DomainService.Dnssec.DsRecordBaseDto", "status", codes(DsRecordStatus::ALL, DsRecordStatus::code)),
        ("Dna.DomainService.Domains.DomainDocumentInfoDto", "type", codes(DocumentType::ALL, DocumentType::code)),
        ("Dna.DomainService.Domains.DomainDocumentInfoDto", "status", codes(DocumentStatus::ALL, DocumentStatus::code)),
        (
            "Dna.DomainService.Domains.DomainDocumentInfoDto",
            "operationType",
            codes(DocumentOperationType::ALL, DocumentOperationType::code),
        ),
        ("Dna.DomainService.Domains.DomainDto", "status", codes(DomainStatus::ALL, DomainStatus::code)),
        ("Dna.OrderService.Orders.OrderDto", "status", codes(OrderStatus::ALL, OrderStatus::code)),
        ("Dna.OrderService.Orders.OrderProductDto", "orderType", codes(OrderType::ALL, OrderType::code)),
        ("Dna.OrderService.Orders.OrderProductDto", "status", codes(OrderProductStatus::ALL, OrderProductStatus::code)),
        (
            "Dna.ResellerService.Resellers.ResellerMeDto",
            "accountCurrency",
            codes(AccountCurrency::ALL, AccountCurrency::code),
        ),
        ("Dna.ResellerService.Resellers.ResellerMeDto", "status", codes(ResellerStatus::ALL, ResellerStatus::code)),
    ];
    for (schema, property, expected) in cases {
        let property = spec.resolve(&schemas[schema]["properties"][property]);
        let documented: Vec<i64> = property["enum"]
            .as_array()
            .unwrap_or_else(|| panic!("{schema} has no enum"))
            .iter()
            .map(|v| v.as_i64().unwrap())
            .collect();
        assert_eq!(documented, expected, "{schema}");
    }
}
