<div align="center">
  <a href="README-TR.md"   >   TR <img style="padding-top: 8px" src="https://raw.githubusercontent.com/yammadev/flag-icons/master/png/TR.png" alt="TR" height="20" /></a>
  <a href="README.md"> | EN <img style="padding-top: 8px" src="https://raw.githubusercontent.com/yammadev/flag-icons/master/png/US.png" alt="EN" height="20" /></a>
   <a href="README-DE.md"> | DE <img style="padding-top: 8px" src="https://raw.githubusercontent.com/yammadev/flag-icons/master/png/DE.png" alt="DE" height="20" /></a>
  <a href="README-SA.md"> | SA <img style="padding-top: 8px" src="https://raw.githubusercontent.com/yammadev/flag-icons/master/png/SA.png" alt="AR" height="20" /></a>
  <a href="README-NL.md"> | NL <img style="padding-top: 8px" src="https://raw.githubusercontent.com/yammadev/flag-icons/master/png/NL.png" alt="NL" height="20" /></a>
  <a href="README-AZ.md"> | AZ <img style="padding-top: 8px" src="https://raw.githubusercontent.com/yammadev/flag-icons/master/png/AZ.png" alt="AZ" height="20" /></a>
  <a href="README-CN.md"> | CN <img style="padding-top: 8px" src="https://raw.githubusercontent.com/yammadev/flag-icons/master/png/CN.png" alt="CN" height="20" /></a>
  <a href="README-FR.md"> | FR <img style="padding-top: 8px" src="https://raw.githubusercontent.com/yammadev/flag-icons/master/png/FR.png" alt="FR" height="20" /></a>
  <a href="README-IT.md"> | IT <img style="padding-top: 8px" src="https://raw.githubusercontent.com/yammadev/flag-icons/master/png/IT.png" alt="IT" height="20" /></a>
</div>

# DomainNameAPI Rust SDK (rust-dna)

<p>
  <a href="https://github.com/domainreseller/rust-dna/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/domainreseller/rust-dna/ci.yml?branch=main&label=CI&style=flat-square" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-0B8DB5?style=flat-square" alt="Apache-2.0"></a>
  <img src="https://img.shields.io/badge/Rust-1.83%2B-0B8DB5?style=flat-square" alt="Rust 1.83+">
</p>

The official Rust SDK for the [Domain Name API](https://www.domainnameapi.com) reseller platform. It covers all
67 operations of the REST API with typed requests and models: domains, contacts, DNS, DNSSEC, SSL certificates,
hosting, orders, products and your account.

```rust
let client = Client::builder().credentials(reseller_id, api_key).environment(Environment::Ote).build()?;

let search = client.domains().search("example.com").await?;
if search.info.is_available() {
    let request = DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact), ["tr.apiname.com", "eu.apiname.com"]);
    client.domains().register_with_contacts(request).await?;
}
```

- **Typed everything.** Every operation is a request type and every answer a struct with documentation. No maps, no JSON parsing.
- **Your own requests.** Every request implements the `Request` trait. Implement it for an endpoint the SDK does not cover yet.
- **Safe with money.** Every request declares whether it reads, writes, charges your balance or deletes something. Billable requests are never retried, and a read-only mode blocks every change.
- **Built-in resilience.** Rate limits are respected automatically, 429 answers are retried, and network errors are retried for reads.
- **Clear errors.** One `Error` type with a stable kind, the API's error code, field-level validation errors, the correlation ID and a hint.
- **Modern Rust.** Async on Tokio, `reqwest` with rustls (no OpenSSL), serde models, lazy paging as a `Stream`, a cloneable `Send + Sync` client, `#![deny(missing_docs)]`.
- **Private.** No telemetry. Nothing leaves your process except the requests you send to the API.

It is the Rust counterpart of [`dna`, the Domain Name API CLI](https://github.com/domainreseller/cli): the same API
document, the same operations and groups, the same safety classes, retry rules and error codes.

## Contents

- [Install](#install)
- [Credentials](#credentials)
- [Quick start](#quick-start)
- [Configuration](#configuration)
- [Usage](#usage): [account](#account) · [domains](#domains) · [contacts](#contacts) · [DNS](#dns) · [DNSSEC](#dnssec) · [SSL](#ssl) · [hosting](#hosting) · [orders and products](#orders-and-products)
- [Requests and the `Request` trait](#requests-and-the-request-trait)
- [Errors](#errors)
- [Safety, retries and rate limits](#safety-retries-and-rate-limits)
- [Async and blocking code](#async-and-blocking-code)
- [Testing your code](#testing-your-code)
- [Project layout](#project-layout)
- [Development](#development)
- [Upgrading from 1.x](#upgrading-from-1x)

## Install

> [!IMPORTANT]
> **Always use the [latest release](https://github.com/domainreseller/rust-dna/releases/latest).** Release packages are versioned and tested.
> The green **Code → Download ZIP** button downloads the development branch instead.

```toml
[dependencies]
domainnameapi = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"   # for reading the list_all streams
```

Until the crate is published on crates.io, take it from the release tag:

```toml
domainnameapi = { git = "https://github.com/domainreseller/rust-dna", tag = "v2.0.0" }
```

**Requirements:** Rust 1.83 or later and a Tokio runtime. The dependencies are `reqwest` (with rustls, no OpenSSL),
`serde`/`serde_json`, `tokio` (timers only), `futures-util`, `chrono`, `base64` and `httpdate`.

## Credentials

The API authenticates every request with two values, which the SDK sends for you:

| Value | Header | Where to find it |
|---|---|---|
| Reseller ID (a UUID) | `__reseller` | Reseller panel → **API settings** |
| API key | `X-API-KEY` | Reseller panel → **API settings** |

> [!NOTE]
> These are **API credentials**: your panel login does not work here. **Production and OTE (the test environment) use
> different API keys.** Using one with the other fails with `INVALID_API_KEY` or `INVALID_RESELLER`.

## Quick start

```rust
use domainnameapi::contacts::Contact;
use domainnameapi::domains::{DomainContacts, DomainListRequest, DomainRegisterWithContactsRequest};
use domainnameapi::{Client, Environment};
use futures_util::TryStreamExt;

#[tokio::main]
async fn main() -> Result<(), domainnameapi::Error> {
    // OTE is the test environment: real API, its own data, no real money.
    let client = Client::builder()
        .credentials("your-reseller-id", "your-api-key")
        .environment(Environment::Ote)
        .build()?;

    // 1. Check the credentials and the balance.
    let account = client.account().get().await?;
    println!("{}: {} USD", account.reseller_name, account.usd_balance);

    // 2. Is the domain available, and at what price?
    let search = client.domains().search("my-new-domain.com").await?;
    println!("{} {} {}", search.info.status, search.info.price, search.info.currency);

    // 3. Register it with one contact for all four roles. This charges the balance.
    let contact = Contact {
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
    };
    let request = DomainRegisterWithContactsRequest::new(
        "my-new-domain.com",
        DomainContacts::same(contact),
        ["tr.apiname.com", "eu.apiname.com"],
    );
    let result = client.domains().register_with_contacts(request).await?;
    println!("{} {}, expires {:?}", result.success, result.status, result.expiration_date);

    // 4. List every domain; pages are fetched as the stream is read.
    let mut domains = client.domains().list_all(DomainListRequest::new());
    while let Some(domain) = domains.try_next().await? {
        println!("{} expires in {} days", domain.domain_name, domain.remaining_days);
    }
    Ok(())
}
```

The [examples](examples) folder has runnable programs for these scenarios (`quickstart`, `register`, `dns`, `ssl`,
`custom_request`, `error_handling`). They read the `DNA_*` environment variables, so point them at OTE:

```bash
export DNA_RESELLER_ID=... DNA_API_KEY=... DNA_ENV=ote
cargo run --example quickstart -- example.com
```

## Configuration

There are three ways to create a client:

```rust
// Production, default settings
let client = Client::new(reseller_id, api_key)?;

// All settings
let client = Client::builder()
    .credentials(reseller_id, api_key)
    .environment(Environment::Ote)
    .timeout(Duration::from_secs(30))
    .read_only(true)
    .build()?;

// The same environment variables the dna CLI reads ...
let client = Client::from_env()?;

// ... plus your own settings
let client = ClientBuilder::from_env()?.http_client(my_reqwest_client).build()?;
```

| `ClientBuilder` method | Default | Meaning |
|---|---|---|
| `credentials(reseller_id, api_key)` | (required) | The credentials. `build()` fails with `NO_CREDENTIALS` when one is empty. |
| `environment(...)` | `Production` | `Production` (`https://api.domainresellerapi.com`) or `Ote` (`https://ote.domainresellerapi.com`). |
| `base_url(...)` | none | Overrides the environment's address, for example for a proxy. A path prefix is kept. |
| `timeout(...)` | 60 s | Time limit of one attempt. |
| `max_retries(...)` | 2 | How often a failed request is repeated (see [retries](#safety-retries-and-rate-limits)). 0 turns retries off. |
| `max_retry_delay(...)` | 30 s | The longest wait before a retry. |
| `pace_requests(...)` | `true` | Spaces requests to stay under the gateway's rate limits. |
| `read_only(...)` | `false` | Blocks every request that is not a read, before it is sent. |
| `user_agent(...)` | `DomainNameAPI-Rust/<version>` | The `User-Agent` header. |
| `http_client(...)` | the SDK's own | Your own `reqwest::Client` (proxy, TLS settings). |

`from_env()` reads:

| Variable | Meaning |
|---|---|
| `DNA_RESELLER_ID`, `DNA_API_KEY` | The credentials. |
| `DNA_ENV` | `production` or `ote` (also `prod`, `live`, `test`, `sandbox`). |
| `DNA_TEST_MODE` | `true`, `1`, `on` or `yes` selects OTE. Used when `DNA_ENV` is not set. |
| `DNA_BASE_URL` | Optional base address override. |
| `DNA_READ_ONLY` | `true` turns on read-only mode. |

`Client` is `Clone + Send + Sync` and cheap to clone (it holds an `Arc`). Create one per set of credentials and share
it for the lifetime of the application: the rate-limit pacing is per client.

## Usage

Each area of the API has its own client, named like the CLI's command groups: `client.account()`,
`client.contacts()`, `client.domains()` (with `forwarding()`, `child_name_servers()` and `documents()`),
`client.dns()`, `client.dnssec()`, `client.ssl()`, `client.hosting()`, `client.orders()` and `client.products()`.
Required inputs are arguments; operations with optional inputs take a request built with `new(...)` and chained
setters. Methods that charge your balance say so in their documentation.

### Account

```rust
let account = client.account().get().await?;          // reseller_id, reseller_name, try_balance, usd_balance
let profile = client.account().profile().await?;      // name, company_name, account_currency, status, ...

// This month's charges, across all pages
let request = AccountTransactionsRequest::new()
    .charge_type(ChargeType::Subtract)
    .start_date("2026-11-01T00:00:00Z".parse()?)
    .transaction_types([TransactionType::Register, TransactionType::Renew]);
let mut transactions = client.account().all_transactions(request);
while let Some(t) = transactions.try_next().await? {
    println!("{:?} {} {} {}", t.creation_time, t.transaction_type, t.amount, t.amount_currency);
}
```

### Domains

**Search**

```rust
let one = client.domains().search("example.com").await?;
let available = one.success && one.info.is_available();

let many = client.domains().bulk_search(["example.com", "example.net", "example.com.tr"]).await?;
for domain in many.available() {
    println!("{}: {} {}", domain.domain_name, domain.price, domain.currency);
}

// With a currency
client.domains().search_with(DomainSearchRequest::new("example.com").currency("USD")).await?;
```

**List and details**

```rust
let page = client.domains().list(DomainListRequest::new().limit(50).statuses([DomainStatus::Active])).await?;

let all: Vec<Domain> = client.domains().list_all(DomainListRequest::new().domain_name("example")).try_collect().await?;

let info = client.domains().info("example.com").await?;
println!("{} {:?}", info.auth_code, info.expiration_date);
if let Some(registrant) = info.contact(ContactType::Registrant) {
    println!("{}", registrant.email);
}
```

**Register, renew, transfer, restore** (these charge your balance)

```rust
// One contact for all four roles, two years
let request = DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact.clone()), ["tr.apiname.com", "eu.apiname.com"])
    .years(2);
client.domains().register_with_contacts(request).await?;

// Different contacts per role, registry attributes, the trustee service
let request = DomainRegisterWithContactsRequest::new(
    "example.com.tr",
    DomainContacts { registrant: Some(owner), admin: Some(staff.clone()), tech: Some(staff.clone()), billing: Some(staff) },
    ["tr.apiname.com", "eu.apiname.com"],
)
.tld_attribute("<key>", "<value>"); // the keys: client.products().tld("com.tr").await?.unwrap().attributes

// With existing contact handles
let request = DomainRegisterRequest::new("example.com", ContactHandles::same("DNA-12345"), ["tr.apiname.com", "eu.apiname.com"]);
client.domains().register(request).await?;

let renewal = client.domains().renew_check("example.com", Some(1)).await?;   // free
if renewal.renewable {
    client.domains().renew("example.com", 1).await?;
}

let check = client.domains().transfer_check("example.com", "EPP-CODE").await?;  // free
if check.transferable {
    client.domains().transfer(DomainTransferRequest::new("example.com", "EPP-CODE")).await?;
}

client.domains().restore("example.com").await?;
```

Requests are checked before they are sent. A registration without all four contact roles, a contact without an
e-mail address or a period of 0 years returns an `ErrorKind::InvalidInput` error listing every problem, and nothing
reaches the API.

> [!NOTE]
> The API can answer a registration, renewal or transfer with HTTP 200 and `"success": false` (for example when the
> registry refuses the domain). The SDK returns that answer as it is, like the Java SDK: check
> `DomainOperationResult::success` and `message`. The same holds for `SslOperationResult` and `ContactCreateResult`.

**Settings**

```rust
client.domains().lock("example.com").await?;
client.domains().unlock("example.com").await?;
client.domains().set_privacy("example.com", true).await?;
client.domains()
    .set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net", "ns2.example.net"]))
    .await?;
client.domains()   // a DNSSEC-signed domain
    .set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net"]).dnssec_mode(DnssecMode::DisableDnssec))
    .await?;
let auth_code = client.domains().update_auth_code("example.com").await?;

client.domains().update_contacts("example.com", DomainContacts::same(contact)).await?;
client.domains().set_contact_handles("example.com", ContactHandles { tech: "DNA-12345".into(), ..Default::default() }).await?;
```

**URL forwarding, child nameservers and documents**

```rust
client.domains().forwarding().set("example.com", "https://www.example.net", ForwardType::Standard).await?;
let forwarding = client.domains().forwarding().get("example.com").await?;   // None when not set
client.domains().forwarding().delete("example.com").await?;

// The IP version of each address is detected
client.domains().child_name_servers().add("example.com", "ns1.example.com", ["192.0.2.10", "2001:db8::10"]).await?;
client.domains().child_name_servers()
    .update(DomainHostUpdateRequest::new("example.com", "ns1.example.com", ["192.0.2.20"]))
    .await?;
client.domains().child_name_servers().delete("example.com", "ns1.example.com").await?;

// Registry documents (mostly .tr)
let document = DocumentUpload::from_file(DocumentType::IdentityCard, "id.pdf")?;
client.domains().documents().upload("example.com.tr", [document]).await?;
let documents: Vec<Document> = client.domains().documents().list_all("example.com.tr").try_collect().await?;
```

### Contacts

Contact handles are reusable contacts. Create one once and use it for many domains.

```rust
let created = client.contacts().create(contact.clone()).await?;
let handle = created.handle;                                         // for example DNA-12345

let details = client.contacts().get(&handle).await?;                 // the contact and its verification state
let result = client.contacts().update(&handle, contact).await?;
println!("{} of {} domains updated", result.stats.success_count, result.stats.total_domain_count);

let verifications = client.contacts().verification(&handle).await?;
client.contacts().resend_verification(&handle).await?;
```

### DNS

Zone records of domains on the platform nameservers. Records resolve publicly once the domain uses those nameservers,
which `apply` takes care of.

```rust
for set in client.dns().list("example.com").await? {
    println!("{} {} {} {:?}", set.name, set.record_type, set.ttl, set.values());
}

let dns = client.dns();
dns.add(DnsAddRequest::new("example.com", "www", RecordType::A, ["192.0.2.10"])).await?;
dns.add(DnsAddRequest::new("example.com", "@", "txt", ["v=spf1 include:_spf.example.net ~all"])).await?;   // "txt" is TXT
dns.add(DnsAddRequest::new("example.com", "@", RecordType::Mx, ["10 mx1.example.net", "20 mx2.example.net"]).ttl(3600)).await?;
dns.update(DnsUpdateRequest::new("example.com", "www", RecordType::A, ["192.0.2.20"]).ttl(300)).await?;
dns.delete(DnsDeleteRequest::new("example.com", "www", RecordType::A, "192.0.2.20")).await?;

let applied = dns.apply("example.com").await?;
```

The SDK checks what the API would refuse: a CNAME at the apex, an MX without its priority, a CAA value without its
three parts, an A record that is not IPv4, a TTL outside 1–86400. Zone calls are kept 1 second apart, the gateway's
limit. The OTE test environment answers zone calls with HTTP 500, so DNS code can only be tried in production.

### DNSSEC

```rust
let info = client.dnssec().get("example.com").await?;      // status, ds_records, allowed_algorithms, ...

let record: DsData = "12345 13 2 49FD46E6C4B45C55D4AC69CBD3CD34AC1AFE51DE".parse()?;
client.dnssec().add("example.com", [record.clone()]).await?;
client.dnssec().remove("example.com", record).await?;
client.dnssec().disable("example.com").await?;            // removes every DS record
```

### SSL

A domain can have several certificates, so an existing certificate is addressed by its **certificate ID** (a GUID),
never by its domain name. `initialize` and `renew` return the ID: store it. `list` finds the IDs of the certificates
you already have.

```rust
// 1. Buy (charges the balance). Package codes come from client.products().ssls(...).
let approvers = client.ssl().approvers(SslApproversRequest::for_purchase("example.com", "PKG-CODE")).await?;
let order = client.ssl().initialize("example.com", "PKG-CODE").await?;
let certificate_id = order.certificate_id;                  // store it: every later call takes it

// 2. A CSR and private key, if you do not have one
let csr = client.ssl().generate_csr(
    SslGenerateCsrRequest::new("example.com")
        .country("TR").state("Istanbul").locality("Istanbul").organization("Example Ltd").organization_unit("IT"),
).await?;

// 3. Configure within 15 days: CSR, validation method and contact
client.ssl().configure(
    SslConfigureRequest::new(&certificate_id, &csr.csr, certificate_contact.clone())
        .authentication_type(AuthenticationType::Email)
        .approver_mail(&approvers[0])
        .web_server_type(WebServerType::Apache),
).await?;

// 4. Wait for issuance, then download
let status = client.ssl().status(&certificate_id).await?;
if status.is_issued {
    let content = client.ssl().content(&certificate_id).await?;
    std::fs::write("fullchain.pem", content.full_chain())?;
}

let details = client.ssl().get(&certificate_id).await?;     // subscription, configuration, contacts
client.ssl().reissue(SslReissueRequest::new(&certificate_id).csr(new_csr).private_key(new_key)).await?;
client.ssl().revoke(&certificate_id).await?;                // cannot be undone
```

**List certificates and find an ID**

```rust
let page = client.ssl().list(SslListRequest::new().domain_name("example.com")).await?;   // newest first

let mut all = client.ssl().list_all(SslListRequest::new());
while let Some(c) = all.try_next().await? {
    println!("{} {} {} {:?}", c.certificate_id, c.domain_name, c.status, c.subscription_expires_at);
}

// The approver addresses of a certificate you already have
let mails = client.ssl().approvers(SslApproversRequest::for_certificate(&certificate_id)).await?;
```

**Renew** (this charges your balance)

```rust
// Allowed from 30 days before details.subscription.expires_at. The renewal is a new certificate with its own ID.
let renewal = client.ssl().renew(&certificate_id).await?;
client.ssl().configure(
    SslConfigureRequest::new(&renewal.certificate_id, &new_csr, certificate_contact)
        .authentication_type(AuthenticationType::Email)
        .approver_mail(&approvers[0]),
).await?;
```

A purchased or renewed certificate must be configured within 15 days; after that it is cancelled. To renew, call
`renew` and then `configure` with the new ID. Do not buy again with `initialize`: that is a separate purchase at the
purchase price, and the old certificate is never marked as renewed. A certificate ID that is empty or not a GUID (for
example a domain name passed by older code) is refused before anything is sent.

### Hosting

```rust
let packages = client.hosting().packages().await?;
let created = client.hosting().create("example.com", "starter-linux").await?;          // charges the balance
let status = client.hosting().status(HostingStatusRequest::by_service_id(&created.service_id)).await?;
if status.is_active() {
    println!("{} {}", status.panel_address, status.username);
}
let hosting = client.hosting().get(HostingGetRequest::by_domain("example.com")).await?;
let plans = client.hosting().plans(None).await?;
```

### Orders and products

```rust
let order = client.orders().get("ORDER-CODE").await?;

if let Some(com) = client.products().tld("com").await? {
    println!("{:?}, DNSSEC: {}", com.register_price(1), com.dnssec_available);
}

let mut tlds = client.products().all_tlds(ProductTldsRequest::new().currency("USD"));
while let Some(tld) = tlds.try_next().await? {
    println!("{}", tld.name);
}

let price = client.products().info("com", OrderType::Renew, 1).await?;
let wildcard = client.products().ssls(ProductSslsRequest::new().is_wildcard(true)).await?;
```

## Requests and the `Request` trait

Every operation is a request type named after its CLI command: `dna domain renew` is `DomainRenewRequest`,
`dna domain renew-check` is `DomainRenewCheckRequest`, `dna dns add` is `DnsAddRequest`. The area methods build them
for you, and you can also build and send them yourself:

```rust
let request = DomainRenewRequest::new("example.com", 2);
let endpoint = request.endpoint();
println!("{endpoint} is {}", endpoint.safety);   // "POST /api/v1/domains/renew is billable"

let result = client.send(&request).await?;

// The same, with the HTTP details: correlation ID, soft-quota state, attempts, duration
let response = client.send_with_response(&request).await?;
println!("{} quota {:?}", response.correlation_id, response.quota);
```

Rust has no inheritance; the base of every request is a trait:

| Trait | For |
|---|---|
| `Request` | Every operation. `type Response` is the answer type; `Empty` for operations that return nothing (lock, unlock, add a DNS record, ...). |
| `PagedRequest<T>` | List operations (`Response = Page<T>`). Gives access to the `Paging` (skip, limit, sort), and works with `Client::list_all`. |

A request says what it is with `endpoint()`: method, path, safety class and whether it is idempotent. The other three
methods have defaults: `query()` returns the query string (`Query` leaves out empty values), `body()` the JSON body
(`JsonBody` builds one and leaves out empty values, or use `serde_json::json!`), and `validate()` collects problems
before sending. To call an endpoint the SDK does not cover yet, write your own:

```rust
use domainnameapi::{Endpoint, Query, Request, Validator};
use serde::Deserialize;

struct DomainNotesRequest {
    domain_name: String,
}

#[derive(Deserialize)]
struct DomainNotes {
    notes: Vec<String>,
}

impl Request for DomainNotesRequest {
    type Response = DomainNotes;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/domains/notes")          // GET is a read
    }

    fn query(&self) -> Query {
        Query::new().set_str("DomainName", &self.domain_name)
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
    }
}

let notes = client.send(&DomainNotesRequest { domain_name: "example.com".into() }).await?;
```

Your request gets everything the built-in ones get: credentials, retries by safety class, pacing, read-only mode and
error handling. For a change, use `Endpoint::post(path, OperationSafety::Write)` (or `Billable`, `Destructive`), and
add `.idempotent()` only when sending it twice is harmless.

## Errors

Every failed request returns an `Error`, also when the request failed its checks and nothing was sent.

```rust
use domainnameapi::ErrorKind;

match client.domains().renew("example.com", 1).await {
    Ok(result) => println!("{} {}", result.success, result.message),
    Err(e) if e.kind() == ErrorKind::Validation => {
        println!("{}: {}", e.code(), e.message());   // Dna.DomainService:Domain:10007: ...
        for field in e.validation_errors() {
            println!("  {field}");
        }
    }
    Err(e) if e.is_outcome_unknown() => {
        // A billable or changing request failed on the network: it may have reached the API.
        // Check the state before sending it again.
        let info = client.domains().info("example.com").await?;
    }
    Err(e) => return Err(e.into()),
}
```

| Method | Meaning |
|---|---|
| `kind()` | A stable classification (table below). Branch on it, not on the message. |
| `code()` | The API's error code (such as `Dna.DomainService:Domain:10007`), or one of the SDK's codes below. The common ones are constants in `domainnameapi::codes`. |
| `message()` | What happened. Boilerplate messages are replaced with the API's details. |
| `status()` | The HTTP status, or `None` when no answer arrived. |
| `details()`, `validation_errors()` | Additional detail and field-level problems (from the API, or from the checks before sending). |
| `correlation_id()` | The gateway's `X-Correlation-Id`. Quote it when contacting support. |
| `hint()` | A short suggestion on what to do next. |
| `is_retryable()` | Whether repeating the same request later can succeed. |
| `is_outcome_unknown()` | The request may have run: a change failed on the network, or the API said the registry may already have done it. |
| `is_not_found()`, `is_insufficient_balance()` | Shortcuts for common cases. |
| `operation()`, `safety()` | The failed request (`POST /api/v1/domains/renew`) and its safety class. |
| `retry_after()`, `response_body()` | The gateway's `Retry-After` and the raw answer. |

`Error` implements `std::error::Error` and `Display` (`[CODE] message (HTTP 400); field: problem`), so it works
with `?`, `anyhow` and `thiserror`.

| `kind()` | When |
|---|---|
| `Auth` | Wrong or missing credentials, or an unknown or suspended reseller in this environment (401, redirect). |
| `Forbidden` | Authenticated but not allowed (403). |
| `NotFound` | The domain, contact, certificate, order or other resource does not exist (404). |
| `Validation` | The API refused the input or a business rule failed: bad field, domain locked, insufficient balance (400, 422). |
| `Conflict` | The resource's state does not allow the operation (409). |
| `RateLimited` | Still rate limited after the automatic retries (429). |
| `Server` | A server-side failure (5xx). Often temporary. |
| `Network`, `Timeout` | No answer: DNS, TLS, proxy, connection reset, or no answer within the timeout. |
| `InvalidInput` | The request failed its checks, or a client setting is invalid; nothing was sent. |
| `ReadOnly` | Blocked by read-only mode; nothing was sent. |
| `Unknown` | Anything else, including an answer the SDK could not read. |

`ErrorKind::as_str()` gives the CLI's names (`auth`, `not_found`, `rate_limited`, ...). Codes the SDK sets itself:

| Code | Meaning |
|---|---|
| `INVALID_RESELLER` | The gateway redirected to its suspend page: the reseller ID is unknown, inactive or suspended **in this environment**. |
| `INVALID_API_KEY` | The key does not match the reseller. Production and OTE keys differ. |
| `MISSING_API_KEY`, `MISSING_RESELLER_ID`, `NO_CREDENTIALS` | A credential is empty. |
| `SUBRESELLER_NOT_ALLOWED` | Sub-reseller accounts cannot use the API. Use the main reseller's credentials. |
| `UNAUTHORIZED` | Any other 401. |
| `RATE_LIMITED` | 429 after the retries. |
| `NETWORK`, `TIMEOUT` | No answer arrived. |
| `READ_ONLY` | Blocked by read-only mode. |
| `INVALID_INPUT`, `INVALID_CONFIGURATION` | A request failed its checks, or a client setting is invalid. |
| `EMPTY_RESPONSE`, `INVALID_RESPONSE` | A successful answer was empty or could not be read. |
| `HTTP_<status>` | The API sent no error code. |

## Safety, retries and rate limits

Every request has a safety class, the same one the `dna` CLI uses:

| `OperationSafety` | Means | Examples |
|---|---|---|
| `Read` | No side effects, even when the API uses POST or PUT | search, details, lists, renewal and transfer checks, CSR generation, certificate status |
| `Write` | A free change that can be undone | DNS records, lock, privacy, nameservers, contacts |
| `Billable` | Charges your balance | register, renew, transfer, restore, SSL purchase and renewal, hosting order |
| `Destructive` | Deletes something or is hard to undo | DNS record deletion, DNSSEC removal, certificate revocation |

- **Read-only mode** (`read_only(true)` or `DNA_READ_ONLY=true`) blocks everything that is not a `Read` before it is
  sent. Use it while exploring, in reports and in dashboards.
- **Retries.** 429 answers are always retried, honouring `Retry-After`. This is safe even for billable requests,
  because the gateway refuses them before anything runs. Network errors, timeouts and 502/503/504 answers are retried
  only for reads and for the writes the CLI marks idempotent (lock, unlock, privacy, nameservers, DNS update and
  apply). A registration or renewal is never sent twice.
- **Pacing.** The gateway allows 5 requests per second per key and 2 per second for domain searches. The client
  spaces requests to stay under these limits, and keeps DNS zone calls 1 second apart. Domain info and the TLD list
  are not rate limited.
- **Soft quota.** Searches and info lookups cost 1 point, everything else 10. Past 200 points the gateway slows each
  request down by up to 5 seconds; points decay by 20% per minute. `send_with_response` shows the state in `quota`.

## Async and blocking code

The SDK is async and needs a Tokio runtime (the one `#[tokio::main]` starts, or your server's). The futures are
`Send`, so they work in `tokio::spawn`, axum and actix handlers. Lists can be read page by page (`list`) or as a
`Stream` of items (`list_all`), which fetches the next page only when it is needed:

```rust
use futures_util::TryStreamExt;

let expiring: Vec<Domain> = client
    .domains()
    .list_all(DomainListRequest::new())
    .try_filter(|d| std::future::ready(d.remaining_days < 30))
    .try_collect()
    .await?;
```

There is no blocking API: it would double the surface for little gain. In synchronous code, run the calls on a
runtime:

```rust
let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
let account = runtime.block_on(client.account().get())?;
```

## Testing your code

The SDK only needs an HTTP endpoint. To test code that uses it without calling the API, point `base_url` at a local
server (for example one built with `wiremock`, `httpmock` or `axum`) and turn pacing off:

```rust
let client = Client::builder()
    .credentials("test", "test")
    .base_url(format!("http://{}", server_address))
    .pace_requests(false)
    .build()?;
```

The models implement `Default` and `Deserialize`, so test data can come from JSON; errors for your mocks can be built
with `Error::new(kind, code, message)`. For tests against the real API, use OTE with its own key.

## Project layout

```
src/
├── lib.rs           Crate documentation and re-exports
├── client.rs        Client, ClientBuilder, Environment: sending, retries, pacing, error parsing, list_all
├── request.rs       Request, PagedRequest, Endpoint, OperationSafety, Query, JsonBody, Validator, Page, Empty
├── error.rs         Error, ErrorKind, codes
├── de.rs            Forgiving decoding and the macros that declare models and enums (internal)
├── account.rs       AccountClient     Balances, profile and transactions
├── contacts.rs      ContactsClient    Contact handles
├── domains/         DomainsClient     Domains, plus forwarding(), child_name_servers() and documents()
├── dns.rs           DnsClient         Zone records
├── dnssec.rs        DnssecClient      DS records
├── ssl.rs           SslClient         Certificates
├── hosting.rs       HostingClient     Hosting packages and services
├── orders.rs        OrdersClient      Orders
└── products.rs      ProductsClient    TLDs, SSL products and prices
tests/               Fake-API tests, the spec coverage test and testdata/openapi.json
examples/            Runnable programs
```

Every area module has the same shape: the models and enums of its answers, one request type per operation
(`XxxRequest`) and an `XxxClient`.

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                    # unit, fake-API, spec and doc tests; none calls the real API
cargo build --examples
```

Without a local toolchain, use Docker:

```bash
docker run --rm -v "$PWD":/src -w /src rust:1.83 sh -c 'rustup component add rustfmt clippy; cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo test'
```

The tests never call the real API: they use a small HTTP server on 127.0.0.1 (`tests/common`). CI runs them on the
minimum supported Rust version (1.83) and on the latest stable.

`tests/testdata/openapi.json` is the API document the `dna` CLI embeds. `tests/spec.rs` checks that every operation
in it has exactly one request type with the CLI's method, path, safety class and idempotency, that every request
sends only query parameters and body properties the document describes (nested objects included, required ones
present), and that the answer types use the document's names. When the API gains an operation, update the document
and the test fails until the request type exists.

## Upgrading from 1.x

2.0 is a new SDK. 1.x was a port of the PHP library: a blocking `DomainNameApi` client returning untyped
`serde_json::Value` envelopes. 2.0 is async, with typed requests and models, `Result<T, Error>` and a builder.

| 1.x | 2.0 |
|---|---|
| `DomainNameApi::new(id, key, true)` | `Client::builder().credentials(id, key).environment(Environment::Ote).build()?` |
| `dna.get_reseller_details()`, `dna.get_current_balance(..)` | `client.account().get()` / `profile()` |
| `dna.check_availability(..)` | `client.domains().search(..)` / `bulk_search(..)` |
| `dna.get_list(..)` | `client.domains().list(..)` / `list_all(..)` |
| `dna.get_details(domain)` | `client.domains().info(domain)` |
| `dna.register_with_contact_info(..)` | `client.domains().register_with_contacts(..)` |
| `dna.renew(domain, period)` | `client.domains().renew(domain, years)` |
| `dna.transfer(domain, epp_code, period)` | `client.domains().transfer(DomainTransferRequest::new(domain, auth_code).years(years))` |
| `dna.check_transfer(domain, auth_code)` | `client.domains().transfer_check(domain, auth_code)` |
| `dna.modify_name_server(..)` | `client.domains().set_name_servers(..)` |
| `dna.enable_theft_protection_lock(domain)` | `client.domains().lock(domain)` |
| `dna.modify_privacy_protection_status(..)` | `client.domains().set_privacy(..)` |
| `dna.save_contacts(..)` | `client.domains().update_contacts(..)` |
| `dna.add_child_name_server(..)` | `client.domains().child_name_servers().add(..)` |
| `dna.get_resource_records(domain)` | `client.dns().list(domain)` |
| `dna.add_resource_record(..)` | `client.dns().add(..)` |
| `dna.set_resource_record_set(..)` | `client.dns().update(..)` |
| `dna.apply_resource_records(domain)` | `client.dns().apply(domain)` |
| `dna.set_forwarding(..)` | `client.domains().forwarding().set(..)` |
| `dna.get_tld_list(..)` | `client.products().all_tlds(..)` |
| `result["result"] == "ERROR"` | `Err(e)` with `e.kind()` and `e.code()` |

The transfer approve/cancel/reject and `sync_from_registry` calls of 1.x are not part of the REST API's document and
have no 2.0 counterpart. The MSRV stays 1.83; `ureq` is replaced by `reqwest`, so 2.0 needs a Tokio runtime. 1.x sent
error reports to Sentry; 2.0 sends nothing anywhere except to the API.

## Links

- [`dna` CLI and MCP server](https://github.com/domainreseller/cli): the same API from the terminal and from AI agents.
- [API documentation](https://www.domainnameapi.com/docs/)
- [Changelog](CHANGELOG.md)

## License

Apache License 2.0. See [LICENSE](LICENSE).
