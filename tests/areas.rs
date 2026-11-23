//! Every method of the area clients sends the operation it is named after.

mod common;

use common::FakeApi;
use domainnameapi::account::AccountTransactionsRequest;
use domainnameapi::contacts::Contact;
use domainnameapi::dns::{DnsAddRequest, DnsDeleteRequest, DnsUpdateRequest, RecordType};
use domainnameapi::dnssec::{DnssecAlgorithm, DsData, DsDigestType};
use domainnameapi::domains::*;
use domainnameapi::hosting::{HostingGetRequest, HostingStatusRequest};
use domainnameapi::products::{OrderType, ProductListRequest, ProductSslsRequest, ProductTldsRequest};
use domainnameapi::ssl::*;
use futures_util::TryStreamExt;
use serde_json::json;

const ID: &str = "6f1c0000-0000-4000-8000-000000000001";

fn contact() -> Contact {
    Contact {
        first_name: "Ada".into(),
        last_name: "Lovelace".into(),
        email: "ada@example.com".into(),
        address: "1 Example Street".into(),
        city: "Istanbul".into(),
        state: "Istanbul".into(),
        postal_code: "34000".into(),
        country: "TR".into(),
        phone_country_code: "90".into(),
        phone: "5551234567".into(),
        ..Default::default()
    }
}

fn ssl_contact() -> SslContact {
    SslContact {
        first_name: "Ada".into(),
        last_name: "Lovelace".into(),
        company_name: "Example".into(),
        email: "ada@example.com".into(),
        address: "1 Example Street".into(),
        city: "Istanbul".into(),
        postal_code: "34000".into(),
        country: "TR".into(),
        phone_country_code: "90".into(),
        phone: "5551234567".into(),
        ..Default::default()
    }
}

#[tokio::test]
async fn every_area_method_sends_its_operation() {
    let api = FakeApi::start().await;
    let client = api.client();
    let mut sent = Vec::new();
    macro_rules! check {
        ($call:expr, $method:literal, $path:literal) => {{
            let before = api.requests().len();
            $call.await.unwrap();
            let request = api.last();
            assert_eq!(api.requests().len(), before + 1, "{}", stringify!($call));
            assert_eq!((request.method.as_str(), request.path.as_str()), ($method, $path), "{}", stringify!($call));
            sent.push(format!("{} {}", $method, $path));
        }};
    }

    let account = client.account();
    check!(account.get(), "GET", "/api/v1/deposit/accounts/me");
    check!(account.profile(), "GET", "/api/v1/resellers/me");
    check!(account.transactions(AccountTransactionsRequest::new()), "GET", "/api/v1/deposit/transactions");

    let contacts = client.contacts();
    check!(contacts.get("DNA-1"), "GET", "/api/v1/contacts/DNA-1");
    check!(contacts.create(contact()), "POST", "/api/v1/contacts");
    check!(contacts.update("DNA-1", contact()), "PUT", "/api/v1/contacts/DNA-1");
    check!(contacts.verification("DNA-1"), "GET", "/api/v1/contacts/verification/check");
    check!(contacts.resend_verification("DNA-1"), "POST", "/api/v1/contacts/verification/request");

    let domains = client.domains();
    let handles = ContactHandles::same("DNA-1");
    check!(domains.search("example.com"), "POST", "/api/v1/domains/search");
    check!(
        domains.search_with(DomainSearchRequest::new("example.com").currency("USD")),
        "POST",
        "/api/v1/domains/search"
    );
    check!(domains.bulk_search(["a.com", "b.com"]), "POST", "/api/v1/domains/bulk-search");
    check!(domains.bulk_search_with(DomainBulkSearchRequest::new(["a.com"])), "POST", "/api/v1/domains/bulk-search");
    check!(domains.list(DomainListRequest::new()), "GET", "/api/v1/domains");
    check!(domains.info("example.com"), "GET", "/api/v1/domains/info");
    check!(
        domains.register(DomainRegisterRequest::new("example.com", handles.clone(), ["ns1.example.net"])),
        "POST",
        "/api/v1/domains/register"
    );
    check!(
        domains.register_with_contacts(DomainRegisterWithContactsRequest::new(
            "example.com",
            DomainContacts::same(contact()),
            ["ns1.example.net"]
        )),
        "POST",
        "/api/v1/domains/register-with-contacts"
    );
    check!(domains.renew("example.com", 1), "POST", "/api/v1/domains/renew");
    check!(domains.renew_check("example.com", Some(1)), "POST", "/api/v1/domains/renew/check");
    check!(domains.transfer(DomainTransferRequest::new("example.com", "code")), "POST", "/api/v1/domains/transfer");
    check!(domains.transfer_check("example.com", "code"), "POST", "/api/v1/domains/transfers/check");
    check!(domains.transfer_status("example.com"), "POST", "/api/v1/domains/transfers/query");
    check!(domains.restore("example.com"), "POST", "/api/v1/domains/restore");
    check!(domains.lock("example.com"), "POST", "/api/v1/domains/lock");
    check!(domains.unlock("example.com"), "POST", "/api/v1/domains/unlock");
    check!(domains.set_privacy("example.com", true), "POST", "/api/v1/domains/privacy");
    check!(domains.update_auth_code("example.com"), "PUT", "/api/v1/domains/auth");
    check!(
        domains.set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net"])),
        "PUT",
        "/api/v1/domains/dns/name-server"
    );
    check!(
        domains.update_contacts("example.com", DomainContacts { tech: Some(contact()), ..Default::default() }),
        "PUT",
        "/api/v1/domains/contacts/update"
    );
    check!(domains.set_contact_handles("example.com", handles), "PUT", "/api/v1/domains/contacts/update-with-handle");

    let forwarding = domains.forwarding();
    check!(forwarding.get("example.com"), "GET", "/api/v1/domains/forwards");
    check!(
        forwarding.set("example.com", "https://example.org", ForwardType::Standard),
        "POST",
        "/api/v1/domains/forwards"
    );
    check!(forwarding.delete("example.com"), "DELETE", "/api/v1/domains/forwards");

    let hosts = domains.child_name_servers();
    check!(hosts.add("example.com", "ns1.example.com", ["192.0.2.1"]), "POST", "/api/v1/domains/dns/host");
    check!(
        hosts.update(DomainHostUpdateRequest::new("example.com", "ns1.example.com", ["192.0.2.2"])),
        "PUT",
        "/api/v1/domains/dns/host"
    );
    check!(hosts.delete("example.com", "ns1.example.com"), "DELETE", "/api/v1/domains/dns/host");

    let documents = domains.documents();
    check!(documents.list(DomainDocumentsListRequest::new("example.com.tr")), "GET", "/api/v1/domains/documents");
    check!(
        documents.upload("example.com.tr", [DocumentUpload::new(DocumentType::Passport, "p.pdf", b"%PDF".to_vec())]),
        "POST",
        "/api/v1/domains/documents"
    );

    let dns = client.dns();
    check!(dns.list("example.com"), "GET", "/api/v1/domains/zones");
    check!(
        dns.add(DnsAddRequest::new("example.com", "www", RecordType::A, ["192.0.2.1"])),
        "POST",
        "/api/v1/domains/zones"
    );
    check!(
        dns.update(DnsUpdateRequest::new("example.com", "www", RecordType::A, ["192.0.2.2"])),
        "PUT",
        "/api/v1/domains/zones"
    );
    check!(
        dns.delete(DnsDeleteRequest::new("example.com", "www", "A", "192.0.2.2")),
        "DELETE",
        "/api/v1/domains/zones"
    );
    check!(dns.apply("example.com"), "POST", "/api/v1/domains/zones/apply");

    let dnssec = client.dnssec();
    let ds = DsData::new(1, DnssecAlgorithm::Ed25519, DsDigestType::Sha256, "AB");
    check!(dnssec.get("example.com"), "GET", "/api/v1/domains/dns/dnssec");
    check!(dnssec.add("example.com", [ds.clone()]), "POST", "/api/v1/domains/dns/dnssec");
    check!(dnssec.remove("example.com", ds), "DELETE", "/api/v1/domains/dns/dnssec");
    check!(dnssec.disable("example.com"), "POST", "/api/v1/domains/dns/dnssec/disable");

    let ssl = client.ssl();
    check!(ssl.list(SslListRequest::new()), "GET", "/api/v1/ssls");
    check!(
        ssl.generate_csr(
            SslGenerateCsrRequest::new("example.com")
                .country("TR")
                .state("Istanbul")
                .locality("Istanbul")
                .organization("Example")
                .organization_unit("IT")
        ),
        "POST",
        "/api/v1/ssls/generate-csr"
    );
    check!(ssl.check_csr("csr"), "POST", "/api/v1/ssls/check-csr");
    check!(ssl.initialize("example.com", "PositiveSSL"), "POST", "/api/v1/ssls/initialize");
    check!(ssl.renew(ID), "POST", "/api/v1/ssls/renew");
    check!(ssl.configure(SslConfigureRequest::new(ID, "csr", ssl_contact())), "PUT", "/api/v1/ssls/configure");
    check!(ssl.status(ID), "PUT", "/api/v1/ssls/check-status");
    check!(ssl.get(ID), "GET", "/api/v1/ssls/certificate");
    check!(ssl.content(ID), "GET", "/api/v1/ssls/certificate-content");
    check!(ssl.reissue(SslReissueRequest::new(ID)), "POST", "/api/v1/ssls/reissue");
    check!(
        ssl.approvers(SslApproversRequest::for_purchase("example.com", "PositiveSSL")),
        "GET",
        "/api/v1/ssls/approvers"
    );
    check!(ssl.revoke(ID), "DELETE", "/api/v1/ssls/revoke");

    let hosting = client.hosting();
    check!(hosting.packages(), "GET", "/api/v1/hostings/packages");
    check!(hosting.plans(None), "GET", "/api/v1/hostings/plans");
    check!(hosting.create("example.com", "Starter"), "POST", "/api/v1/hostings/create");
    check!(hosting.get(HostingGetRequest::by_domain("example.com")), "GET", "/api/v1/hostings");
    check!(hosting.get_by_token("token"), "GET", "/api/v1/hostings/by-token");
    check!(hosting.status(HostingStatusRequest::by_service_id("HST-1")), "GET", "/api/v1/hostings/check-status");

    check!(client.orders().get("ORD-1"), "GET", "/api/v1/order/ORD-1");

    let products = client.products();
    check!(products.list(ProductListRequest::new()), "GET", "/api/v1/products");
    check!(products.info("com", OrderType::Register, 1), "GET", "/api/v1/products/info");
    check!(products.tlds(ProductTldsRequest::new()), "GET", "/api/v1/products/tlds");
    check!(products.ssls(ProductSslsRequest::new()), "GET", "/api/v1/products/ssls");

    let distinct: std::collections::BTreeSet<_> = sent.iter().collect();
    assert_eq!(distinct.len(), 67, "every operation is reachable from an area client");
}

#[tokio::test]
async fn streams_and_shortcuts() {
    let api = FakeApi::start().await;
    let client = api.client();

    api.reply_json(200, json!({"items": [{"name": "com"}, {"name": "com.tr"}], "totalCount": 2}));
    let tld = client.products().tld(".COM.TR").await.unwrap().unwrap();
    assert_eq!(tld.name, "com.tr");
    assert_eq!(api.last().query("FilterText"), Some("com.tr"));

    api.reply_json(200, json!({"items": [], "totalCount": 0}));
    assert!(client.products().tld("nope").await.unwrap().is_none());

    api.reply_json(200, json!({"items": [{"traceId": "t1"}], "totalCount": 1}));
    let all: Vec<_> = client.account().all_transactions(AccountTransactionsRequest::new()).try_collect().await.unwrap();
    assert_eq!(all[0].trace_id, "t1");

    api.reply_json(200, json!({"items": [{"certificateId": ID}], "totalCount": 1}));
    let certificates: Vec<_> =
        client.ssl().list_all(SslListRequest::new().domain_name("example.com")).try_collect().await.unwrap();
    assert_eq!(certificates[0].certificate_id, ID);
    assert!(api.last().query("Sorting").is_none());

    api.reply_json(200, json!({"items": [{"name": "id.pdf", "status": 3}], "totalCount": 1}));
    let documents: Vec<_> = client.domains().documents().list_all("example.com.tr").try_collect().await.unwrap();
    assert_eq!(documents[0].status, DocumentStatus::Approved);

    api.reply_json(200, json!({"items": [{"productName": "com"}], "totalCount": 1}));
    let products: Vec<_> = client.products().list_all(ProductListRequest::new()).try_collect().await.unwrap();
    assert_eq!(products[0].name, "com");

    api.reply_json(200, json!({"items": [{"productCode": "PositiveSSL"}], "totalCount": 1}));
    let ssls: Vec<_> = client.products().all_ssls(ProductSslsRequest::new()).try_collect().await.unwrap();
    assert_eq!(ssls[0].product_code, "PositiveSSL");

    api.reply_json(200, json!({"results": [{"fieldType": "Email", "isUserVerified": true}]}));
    assert!(client.contacts().verification("DNA-1").await.unwrap()[0].verified);

    api.reply_json(200, json!({"mails": ["admin@example.com"]}));
    assert_eq!(client.ssl().approvers(SslApproversRequest::for_certificate(ID)).await.unwrap(), ["admin@example.com"]);

    api.reply_json(200, json!({"items": [{"keyTag": 1, "isSuccess": true}]}));
    let results = client.dnssec().add("example.com", ["1 13 2 AB".parse::<DsData>().unwrap()]).await.unwrap();
    assert!(results[0].success);
}
