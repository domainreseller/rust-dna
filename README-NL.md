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

De officiële Rust SDK voor het resellerplatform van [Domain Name API](https://www.domainnameapi.com). Hij dekt alle
67 operaties van de REST API met getypeerde requests en modellen: domeinen, contacten, DNS, DNSSEC, SSL-certificaten,
hosting, bestellingen, producten en je account.

```rust
let client = Client::builder().credentials(reseller_id, api_key).environment(Environment::Ote).build()?;

let search = client.domains().search("example.com").await?;
if search.info.is_available() {
    let request = DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact), ["tr.apiname.com", "eu.apiname.com"]);
    client.domains().register_with_contacts(request).await?;
}
```

- **Alles getypeerd.** Elke operatie is een requesttype en elk antwoord een gedocumenteerde struct. Geen maps, geen JSON-parsing.
- **Je eigen requests.** Elk request implementeert de trait `Request`. Implementeer hem voor een endpoint dat de SDK nog niet dekt.
- **Veilig met geld.** Elk request geeft aan of het leest, schrijft, je saldo belast of iets verwijdert. Billable requests worden nooit herhaald en een read-only modus blokkeert elke wijziging.
- **Ingebouwde veerkracht.** Rate limits worden automatisch gerespecteerd, 429-antwoorden worden herhaald en netwerkfouten worden herhaald bij leesacties.
- **Duidelijke fouten.** Eén `Error`-type met een vaste soort, de foutcode van de API, validatiefouten per veld, de correlation ID en een hint.
- **Modern Rust.** Async op Tokio, `reqwest` met rustls (geen OpenSSL), serde-modellen, lazy paginering als `Stream`, een kloonbare `Send + Sync`-client, `#![deny(missing_docs)]`.
- **Privé.** Geen telemetrie. Niets verlaat je proces behalve de requests die je naar de API stuurt.

Het is de Rust-tegenhanger van [`dna`, de Domain Name API CLI](https://github.com/domainreseller/cli): hetzelfde API-document,
dezelfde operaties en groepen, dezelfde veiligheidsklassen, retryregels en foutcodes.

## Inhoud

- [Installatie](#installatie)
- [Inloggegevens](#inloggegevens)
- [Snel aan de slag](#snel-aan-de-slag)
- [Configuratie](#configuratie)
- [Gebruik](#gebruik): [account](#account) · [domeinen](#domeinen) · [contacten](#contacten) · [DNS](#dns) · [DNSSEC](#dnssec) · [SSL](#ssl) · [hosting](#hosting) · [bestellingen en producten](#bestellingen-en-producten)
- [Requests en de trait `Request`](#requests-en-de-trait-request)
- [Fouten](#fouten)
- [Veiligheid, retries en rate limits](#veiligheid-retries-en-rate-limits)
- [Async en blokkerende code](#async-en-blokkerende-code)
- [Je code testen](#je-code-testen)
- [Projectindeling](#projectindeling)
- [Ontwikkeling](#ontwikkeling)
- [Upgraden van 1.x](#upgraden-van-1x)

## Installatie

> [!IMPORTANT]
> **Gebruik altijd de [nieuwste release](https://github.com/domainreseller/rust-dna/releases/latest).** Release-pakketten hebben een versienummer en zijn getest.
> De groene knop **Code → Download ZIP** downloadt in plaats daarvan de ontwikkelbranch.

```toml
[dependencies]
domainnameapi = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"   # voor het lezen van de list_all-streams
```

Zolang de crate niet op crates.io is gepubliceerd, haal je hem op via de release-tag:

```toml
domainnameapi = { git = "https://github.com/domainreseller/rust-dna", tag = "v2.0.0" }
```

**Vereisten:** Rust 1.83 of nieuwer en een Tokio-runtime. De dependencies zijn `reqwest` (met rustls, geen OpenSSL),
`serde`/`serde_json`, `tokio` (alleen timers), `futures-util`, `chrono`, `base64` en `httpdate`.

## Inloggegevens

De API authenticeert elk request met twee waarden, die de SDK voor je meestuurt:

| Waarde | Header | Waar je het vindt |
|---|---|---|
| Reseller ID (een UUID) | `__reseller` | Resellerpaneel → **API settings** |
| API-sleutel | `X-API-KEY` | Resellerpaneel → **API settings** |

> [!NOTE]
> Dit zijn **API-inloggegevens**: je paneelaccount werkt hier niet. **Productie en OTE (de testomgeving) gebruiken
> verschillende API-sleutels.** Als je de ene bij de andere gebruikt, krijg je `INVALID_API_KEY` of `INVALID_RESELLER`.

## Snel aan de slag

```rust
use domainnameapi::contacts::Contact;
use domainnameapi::domains::{DomainContacts, DomainListRequest, DomainRegisterWithContactsRequest};
use domainnameapi::{Client, Environment};
use futures_util::TryStreamExt;

#[tokio::main]
async fn main() -> Result<(), domainnameapi::Error> {
    // OTE is de testomgeving: echte API, eigen gegevens, geen echt geld.
    let client = Client::builder()
        .credentials("your-reseller-id", "your-api-key")
        .environment(Environment::Ote)
        .build()?;

    // 1. Controleer de inloggegevens en het saldo.
    let account = client.account().get().await?;
    println!("{}: {} USD", account.reseller_name, account.usd_balance);

    // 2. Is het domein beschikbaar, en tegen welke prijs?
    let search = client.domains().search("my-new-domain.com").await?;
    println!("{} {} {}", search.info.status, search.info.price, search.info.currency);

    // 3. Registreer het met één contact voor alle vier de rollen. Dit belast het saldo.
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

    // 4. Toon alle domeinen; pagina's worden opgehaald terwijl de stream wordt gelezen.
    let mut domains = client.domains().list_all(DomainListRequest::new());
    while let Some(domain) = domains.try_next().await? {
        println!("{} expires in {} days", domain.domain_name, domain.remaining_days);
    }
    Ok(())
}
```

De map [examples](examples) bevat uitvoerbare programma's voor deze scenario's (`quickstart`, `register`, `dns`, `ssl`,
`custom_request`, `error_handling`). Ze lezen de omgevingsvariabelen `DNA_*`, dus laat ze naar OTE wijzen:

```bash
export DNA_RESELLER_ID=... DNA_API_KEY=... DNA_ENV=ote
cargo run --example quickstart -- example.com
```

## Configuratie

Er zijn drie manieren om een client te maken:

```rust
// Productie, standaardinstellingen
let client = Client::new(reseller_id, api_key)?;

// Alle instellingen
let client = Client::builder()
    .credentials(reseller_id, api_key)
    .environment(Environment::Ote)
    .timeout(Duration::from_secs(30))
    .read_only(true)
    .build()?;

// Dezelfde omgevingsvariabelen die de dna CLI leest ...
let client = Client::from_env()?;

// ... plus je eigen instellingen
let client = ClientBuilder::from_env()?.http_client(my_reqwest_client).build()?;
```

| `ClientBuilder`-methode | Standaard | Betekenis |
|---|---|---|
| `credentials(reseller_id, api_key)` | (verplicht) | De inloggegevens. `build()` faalt met `NO_CREDENTIALS` als een ervan leeg is. |
| `environment(...)` | `Production` | `Production` (`https://api.domainresellerapi.com`) of `Ote` (`https://ote.domainresellerapi.com`). |
| `base_url(...)` | geen | Overschrijft het adres van de omgeving, bijvoorbeeld voor een proxy. Een padprefix blijft behouden. |
| `timeout(...)` | 60 s | Tijdslimiet van één poging. |
| `max_retries(...)` | 2 | Hoe vaak een mislukt request wordt herhaald (zie [retries](#veiligheid-retries-en-rate-limits)). 0 zet retries uit. |
| `max_retry_delay(...)` | 30 s | De langste wachttijd voor een retry. |
| `pace_requests(...)` | `true` | Spreidt requests zodat je onder de rate limits van de gateway blijft. |
| `read_only(...)` | `false` | Blokkeert elk request dat geen leesactie is, voordat het wordt verstuurd. |
| `user_agent(...)` | `DomainNameAPI-Rust/<version>` | De `User-Agent`-header. |
| `http_client(...)` | die van de SDK zelf | Je eigen `reqwest::Client` (proxy, TLS-instellingen). |

`from_env()` leest:

| Variabele | Betekenis |
|---|---|
| `DNA_RESELLER_ID`, `DNA_API_KEY` | De inloggegevens. |
| `DNA_ENV` | `production` of `ote` (ook `prod`, `live`, `test`, `sandbox`). |
| `DNA_TEST_MODE` | `true`, `1`, `on` of `yes` kiest OTE. Wordt gebruikt als `DNA_ENV` niet is ingesteld. |
| `DNA_BASE_URL` | Optionele overschrijving van het basisadres. |
| `DNA_READ_ONLY` | `true` zet de read-only modus aan. |

`Client` is `Clone + Send + Sync` en goedkoop om te klonen (hij bevat een `Arc`). Maak één client per set inloggegevens en
deel die gedurende de hele levensduur van de applicatie: de pacing voor de rate limits geldt per client.

## Gebruik

Elk gebied van de API heeft een eigen client, genoemd naar de commandogroepen van de CLI: `client.account()`,
`client.contacts()`, `client.domains()` (met `forwarding()`, `child_name_servers()` en `documents()`),
`client.dns()`, `client.dnssec()`, `client.ssl()`, `client.hosting()`, `client.orders()` en `client.products()`.
Verplichte invoer zijn argumenten; operaties met optionele invoer nemen een request dat je bouwt met `new(...)` en
geketende setters. Methoden die je saldo belasten, vermelden dat in hun documentatie.

### Account

```rust
let account = client.account().get().await?;          // reseller_id, reseller_name, try_balance, usd_balance
let profile = client.account().profile().await?;      // name, company_name, account_currency, status, ...

// De afschrijvingen van deze maand, over alle pagina's
let request = AccountTransactionsRequest::new()
    .charge_type(ChargeType::Subtract)
    .start_date("2026-11-01T00:00:00Z".parse()?)
    .transaction_types([TransactionType::Register, TransactionType::Renew]);
let mut transactions = client.account().all_transactions(request);
while let Some(t) = transactions.try_next().await? {
    println!("{:?} {} {} {}", t.creation_time, t.transaction_type, t.amount, t.amount_currency);
}
```

### Domeinen

**Zoeken**

```rust
let one = client.domains().search("example.com").await?;
let available = one.success && one.info.is_available();

let many = client.domains().bulk_search(["example.com", "example.net", "example.com.tr"]).await?;
for domain in many.available() {
    println!("{}: {} {}", domain.domain_name, domain.price, domain.currency);
}

// Met een valuta
client.domains().search_with(DomainSearchRequest::new("example.com").currency("USD")).await?;
```

**Lijst en details**

```rust
let page = client.domains().list(DomainListRequest::new().limit(50).statuses([DomainStatus::Active])).await?;

let all: Vec<Domain> = client.domains().list_all(DomainListRequest::new().domain_name("example")).try_collect().await?;

let info = client.domains().info("example.com").await?;
println!("{} {:?}", info.auth_code, info.expiration_date);
if let Some(registrant) = info.contact(ContactType::Registrant) {
    println!("{}", registrant.email);
}
```

**Registreren, verlengen, verhuizen, herstellen** (deze belasten je saldo)

```rust
// Eén contact voor alle vier de rollen, twee jaar
let request = DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact.clone()), ["tr.apiname.com", "eu.apiname.com"])
    .years(2);
client.domains().register_with_contacts(request).await?;

// Verschillende contacten per rol, registry-attributen, de trusteedienst
let request = DomainRegisterWithContactsRequest::new(
    "example.com.tr",
    DomainContacts { registrant: Some(owner), admin: Some(staff.clone()), tech: Some(staff.clone()), billing: Some(staff) },
    ["tr.apiname.com", "eu.apiname.com"],
)
.tld_attribute("<key>", "<value>"); // de sleutels: client.products().tld("com.tr").await?.unwrap().attributes

// Met bestaande contacthandles
let request = DomainRegisterRequest::new("example.com", ContactHandles::same("DNA-12345"), ["tr.apiname.com", "eu.apiname.com"]);
client.domains().register(request).await?;

let renewal = client.domains().renew_check("example.com", Some(1)).await?;   // gratis
if renewal.renewable {
    client.domains().renew("example.com", 1).await?;
}

let check = client.domains().transfer_check("example.com", "EPP-CODE").await?;  // gratis
if check.transferable {
    client.domains().transfer(DomainTransferRequest::new("example.com", "EPP-CODE")).await?;
}

client.domains().restore("example.com").await?;
```

Requests worden gecontroleerd voordat ze worden verstuurd. Een registratie zonder alle vier de contactrollen, een contact
zonder e-mailadres of een periode van 0 jaar geeft een `ErrorKind::InvalidInput`-fout terug met een lijst van alle
problemen, en er bereikt niets de API.

> [!NOTE]
> De API kan een registratie, verlenging of verhuizing beantwoorden met HTTP 200 en `"success": false` (bijvoorbeeld als de
> registry het domein weigert). De SDK geeft dat antwoord ongewijzigd terug, net als de Java SDK: controleer
> `DomainOperationResult::success` en `message`. Hetzelfde geldt voor `SslOperationResult` en `ContactCreateResult`.

**Instellingen**

```rust
client.domains().lock("example.com").await?;
client.domains().unlock("example.com").await?;
client.domains().set_privacy("example.com", true).await?;
client.domains()
    .set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net", "ns2.example.net"]))
    .await?;
client.domains()   // een met DNSSEC ondertekend domein
    .set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net"]).dnssec_mode(DnssecMode::DisableDnssec))
    .await?;
let auth_code = client.domains().update_auth_code("example.com").await?;

client.domains().update_contacts("example.com", DomainContacts::same(contact)).await?;
client.domains().set_contact_handles("example.com", ContactHandles { tech: "DNA-12345".into(), ..Default::default() }).await?;
```

**URL-doorsturing, child nameservers en documenten**

```rust
client.domains().forwarding().set("example.com", "https://www.example.net", ForwardType::Standard).await?;
let forwarding = client.domains().forwarding().get("example.com").await?;   // None als er niets is ingesteld
client.domains().forwarding().delete("example.com").await?;

// De IP-versie van elk adres wordt herkend
client.domains().child_name_servers().add("example.com", "ns1.example.com", ["192.0.2.10", "2001:db8::10"]).await?;
client.domains().child_name_servers()
    .update(DomainHostUpdateRequest::new("example.com", "ns1.example.com", ["192.0.2.20"]))
    .await?;
client.domains().child_name_servers().delete("example.com", "ns1.example.com").await?;

// Registrydocumenten (meestal .tr)
let document = DocumentUpload::from_file(DocumentType::IdentityCard, "id.pdf")?;
client.domains().documents().upload("example.com.tr", [document]).await?;
let documents: Vec<Document> = client.domains().documents().list_all("example.com.tr").try_collect().await?;
```

### Contacten

Contacthandles zijn herbruikbare contacten. Maak er één keer een aan en gebruik hem voor veel domeinen.

```rust
let created = client.contacts().create(contact.clone()).await?;
let handle = created.handle;                                         // bijvoorbeeld DNA-12345

let details = client.contacts().get(&handle).await?;                 // het contact en zijn verificatiestatus
let result = client.contacts().update(&handle, contact).await?;
println!("{} of {} domains updated", result.stats.success_count, result.stats.total_domain_count);

let verifications = client.contacts().verification(&handle).await?;
client.contacts().resend_verification(&handle).await?;
```

### DNS

Zonerecords van domeinen op de nameservers van het platform. Records zijn publiek vindbaar zodra het domein die nameservers gebruikt,
waar `apply` voor zorgt.

```rust
for set in client.dns().list("example.com").await? {
    println!("{} {} {} {:?}", set.name, set.record_type, set.ttl, set.values());
}

let dns = client.dns();
dns.add(DnsAddRequest::new("example.com", "www", RecordType::A, ["192.0.2.10"])).await?;
dns.add(DnsAddRequest::new("example.com", "@", "txt", ["v=spf1 include:_spf.example.net ~all"])).await?;   // "txt" wordt TXT
dns.add(DnsAddRequest::new("example.com", "@", RecordType::Mx, ["10 mx1.example.net", "20 mx2.example.net"]).ttl(3600)).await?;
dns.update(DnsUpdateRequest::new("example.com", "www", RecordType::A, ["192.0.2.20"]).ttl(300)).await?;
dns.delete(DnsDeleteRequest::new("example.com", "www", RecordType::A, "192.0.2.20")).await?;

let applied = dns.apply("example.com").await?;
```

De SDK controleert wat de API zou weigeren: een CNAME op de apex, een MX zonder prioriteit, een CAA-waarde zonder de drie
onderdelen, een A-record dat geen IPv4-adres is, een TTL buiten 1–86400. Zoneaanroepen worden 1 seconde uit elkaar
gehouden, de limiet van de gateway. De OTE-testomgeving beantwoordt zoneaanroepen met HTTP 500, dus DNS-code kan alleen
in productie worden uitgeprobeerd.

### DNSSEC

```rust
let info = client.dnssec().get("example.com").await?;      // status, ds_records, allowed_algorithms, ...

let record: DsData = "12345 13 2 49FD46E6C4B45C55D4AC69CBD3CD34AC1AFE51DE".parse()?;
client.dnssec().add("example.com", [record.clone()]).await?;
client.dnssec().remove("example.com", record).await?;
client.dnssec().disable("example.com").await?;            // verwijdert alle DS-records
```

### SSL

Een domein kan meerdere certificaten hebben. Een bestaand certificaat wordt daarom aangeduid met zijn **certificaat-ID**
(een GUID), nooit met de domeinnaam. `initialize` en `renew` geven het ID terug: bewaar het. `list` vindt de ID's van de
certificaten die je al hebt.

```rust
// 1. Kopen (belast het saldo). Pakketcodes komen uit client.products().ssls(...).
let approvers = client.ssl().approvers(SslApproversRequest::for_purchase("example.com", "PKG-CODE")).await?;
let order = client.ssl().initialize("example.com", "PKG-CODE").await?;
let certificate_id = order.certificate_id;                  // bewaar het: elke latere aanroep heeft het nodig

// 2. Een CSR en private key, als je die nog niet hebt
let csr = client.ssl().generate_csr(
    SslGenerateCsrRequest::new("example.com")
        .country("TR").state("Istanbul").locality("Istanbul").organization("Example Ltd").organization_unit("IT"),
).await?;

// 3. Binnen 15 dagen configureren: CSR, validatiemethode en contact
client.ssl().configure(
    SslConfigureRequest::new(&certificate_id, &csr.csr, certificate_contact.clone())
        .authentication_type(AuthenticationType::Email)
        .approver_mail(&approvers[0])
        .web_server_type(WebServerType::Apache),
).await?;

// 4. Wachten op uitgifte, dan downloaden
let status = client.ssl().status(&certificate_id).await?;
if status.is_issued {
    let content = client.ssl().content(&certificate_id).await?;
    std::fs::write("fullchain.pem", content.full_chain())?;
}

let details = client.ssl().get(&certificate_id).await?;     // abonnement, configuratie, contacten
client.ssl().reissue(SslReissueRequest::new(&certificate_id).csr(new_csr).private_key(new_key)).await?;
client.ssl().revoke(&certificate_id).await?;                // kan niet ongedaan worden gemaakt
```

**Certificaten weergeven en een ID vinden**

```rust
let page = client.ssl().list(SslListRequest::new().domain_name("example.com")).await?;   // nieuwste eerst

let mut all = client.ssl().list_all(SslListRequest::new());
while let Some(c) = all.try_next().await? {
    println!("{} {} {} {:?}", c.certificate_id, c.domain_name, c.status, c.subscription_expires_at);
}

// De goedkeuringsadressen van een certificaat dat je al hebt
let mails = client.ssl().approvers(SslApproversRequest::for_certificate(&certificate_id)).await?;
```

**Verlengen** (dit belast je saldo)

```rust
// Toegestaan vanaf 30 dagen vóór details.subscription.expires_at. De verlenging is een nieuw certificaat met een eigen ID.
let renewal = client.ssl().renew(&certificate_id).await?;
client.ssl().configure(
    SslConfigureRequest::new(&renewal.certificate_id, &new_csr, certificate_contact)
        .authentication_type(AuthenticationType::Email)
        .approver_mail(&approvers[0]),
).await?;
```

Een gekocht of verlengd certificaat moet binnen 15 dagen worden geconfigureerd; daarna wordt het geannuleerd. Om te
verlengen roep je `renew` aan en daarna `configure` met het nieuwe ID. Koop niet opnieuw met `initialize`: dat is een
losse aankoop tegen de aankoopprijs, en het oude certificaat wordt nooit als verlengd gemarkeerd. Een certificaat-ID dat
leeg is of geen GUID is (bijvoorbeeld een domeinnaam die oudere code doorgeeft), wordt geweigerd voordat er iets wordt verstuurd.

### Hosting

```rust
let packages = client.hosting().packages().await?;
let created = client.hosting().create("example.com", "starter-linux").await?;          // belast het saldo
let status = client.hosting().status(HostingStatusRequest::by_service_id(&created.service_id)).await?;
if status.is_active() {
    println!("{} {}", status.panel_address, status.username);
}
let hosting = client.hosting().get(HostingGetRequest::by_domain("example.com")).await?;
let plans = client.hosting().plans(None).await?;
```

### Bestellingen en producten

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

## Requests en de trait `Request`

Elke operatie is een requesttype, genoemd naar het bijbehorende CLI-commando: `dna domain renew` is `DomainRenewRequest`,
`dna domain renew-check` is `DomainRenewCheckRequest`, `dna dns add` is `DnsAddRequest`. De gebiedsmethoden bouwen ze voor
je, en je kunt ze ook zelf maken en versturen:

```rust
let request = DomainRenewRequest::new("example.com", 2);
let endpoint = request.endpoint();
println!("{endpoint} is {}", endpoint.safety);   // "POST /api/v1/domains/renew is billable"

let result = client.send(&request).await?;

// Hetzelfde, met de HTTP-details: correlation ID, soft-quotastatus, pogingen, duur
let response = client.send_with_response(&request).await?;
println!("{} quota {:?}", response.correlation_id, response.quota);
```

Rust kent geen overerving; de basis van elk request is een trait:

| Trait | Voor |
|---|---|
| `Request` | Elke operatie. `type Response` is het antwoordtype; `Empty` voor operaties die niets teruggeven (vergrendelen, ontgrendelen, een DNS-record toevoegen, ...). |
| `PagedRequest<T>` | Lijstoperaties (`Response = Page<T>`). Geeft toegang tot de `Paging` (skip, limit, sort) en werkt met `Client::list_all`. |

Een request geeft met `endpoint()` aan wat het is: methode, pad, veiligheidsklasse en of het idempotent is. De andere drie
methoden hebben een standaardimplementatie: `query()` geeft de querystring terug (`Query` laat lege waarden weg), `body()`
de JSON-body (`JsonBody` bouwt er een en laat lege waarden weg, of gebruik `serde_json::json!`), en `validate()` verzamelt
problemen vóór het versturen. Om een endpoint aan te roepen dat de SDK nog niet dekt, schrijf je er zelf een:

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
        Endpoint::get("/api/v1/domains/notes")          // GET is een leesactie
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

Jouw request krijgt alles wat de ingebouwde requests krijgen: inloggegevens, retries per veiligheidsklasse, pacing,
read-only modus en foutafhandeling. Gebruik voor een wijziging `Endpoint::post(path, OperationSafety::Write)` (of
`Billable`, `Destructive`), en voeg `.idempotent()` alleen toe als twee keer versturen geen kwaad kan.

## Fouten

Elk mislukt request geeft een `Error` terug, ook als het request zijn controles niet doorstond en er niets is verstuurd.

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
        // Een billable of wijzigend request mislukte op het netwerk: het kan de API hebben bereikt.
        // Controleer de status voordat je het opnieuw verstuurt.
        let info = client.domains().info("example.com").await?;
    }
    Err(e) => return Err(e.into()),
}
```

| Methode | Betekenis |
|---|---|
| `kind()` | Een vaste classificatie (tabel hieronder). Vertak op deze waarde, niet op het bericht. |
| `code()` | De foutcode van de API (zoals `Dna.DomainService:Domain:10007`), of een van de SDK-codes hieronder. De gangbare zijn constanten in `domainnameapi::codes`. |
| `message()` | Wat er gebeurde. Standaardberichten worden vervangen door de details van de API. |
| `status()` | De HTTP-status, of `None` als er geen antwoord kwam. |
| `details()`, `validation_errors()` | Extra details en problemen per veld (van de API, of van de controles vóór het versturen). |
| `correlation_id()` | De `X-Correlation-Id` van de gateway. Noem hem als je contact opneemt met support. |
| `hint()` | Een korte suggestie voor wat je daarna kunt doen. |
| `is_retryable()` | Of het herhalen van hetzelfde request later kan slagen. |
| `is_outcome_unknown()` | Het request kan zijn uitgevoerd: een wijziging mislukte op het netwerk, of de API meldde dat de registry het mogelijk al heeft gedaan. |
| `is_not_found()`, `is_insufficient_balance()` | Snelkoppelingen voor veelvoorkomende gevallen. |
| `operation()`, `safety()` | Het mislukte request (`POST /api/v1/domains/renew`) en zijn veiligheidsklasse. |
| `retry_after()`, `response_body()` | De `Retry-After` van de gateway en het ruwe antwoord. |

`Error` implementeert `std::error::Error` en `Display` (`[CODE] message (HTTP 400); field: problem`), dus hij werkt met
`?`, `anyhow` en `thiserror`.

| `kind()` | Wanneer |
|---|---|
| `Auth` | Onjuiste of ontbrekende inloggegevens, of een onbekende of geschorste reseller in deze omgeving (401, redirect). |
| `Forbidden` | Geauthenticeerd, maar niet toegestaan (403). |
| `NotFound` | Het domein, contact, certificaat, de bestelling of een andere resource bestaat niet (404). |
| `Validation` | De API weigerde de invoer of een bedrijfsregel faalde: ongeldig veld, domein vergrendeld, onvoldoende saldo (400, 422). |
| `Conflict` | De status van de resource staat de operatie niet toe (409). |
| `RateLimited` | Na de automatische retries nog steeds rate limited (429). |
| `Server` | Een fout aan de serverzijde (5xx). Vaak tijdelijk. |
| `Network`, `Timeout` | Geen antwoord: DNS, TLS, proxy, verbroken verbinding, of geen antwoord binnen de timeout. |
| `InvalidInput` | Het request doorstond zijn controles niet, of een clientinstelling is ongeldig; er is niets verstuurd. |
| `ReadOnly` | Geblokkeerd door de read-only modus; er is niets verstuurd. |
| `Unknown` | Al het overige, inclusief een antwoord dat de SDK niet kon lezen. |

`ErrorKind::as_str()` geeft de namen van de CLI (`auth`, `not_found`, `rate_limited`, ...). Codes die de SDK zelf instelt:

| Code | Betekenis |
|---|---|
| `INVALID_RESELLER` | De gateway verwees door naar de suspendpagina: de reseller-ID is onbekend, inactief of geschorst **in deze omgeving**. |
| `INVALID_API_KEY` | De sleutel hoort niet bij de reseller. Productie- en OTE-sleutels verschillen. |
| `MISSING_API_KEY`, `MISSING_RESELLER_ID`, `NO_CREDENTIALS` | Een inloggegeven is leeg. |
| `SUBRESELLER_NOT_ALLOWED` | Subreseller-accounts kunnen de API niet gebruiken. Gebruik de inloggegevens van de hoofdreseller. |
| `UNAUTHORIZED` | Elke andere 401. |
| `RATE_LIMITED` | 429 na de retries. |
| `NETWORK`, `TIMEOUT` | Er kwam geen antwoord. |
| `READ_ONLY` | Geblokkeerd door de read-only modus. |
| `INVALID_INPUT`, `INVALID_CONFIGURATION` | Een request doorstond zijn controles niet, of een clientinstelling is ongeldig. |
| `EMPTY_RESPONSE`, `INVALID_RESPONSE` | Een geslaagd antwoord was leeg of kon niet worden gelezen. |
| `HTTP_<status>` | De API stuurde geen foutcode. |

## Veiligheid, retries en rate limits

Elk request heeft een veiligheidsklasse, dezelfde als die de `dna` CLI gebruikt:

| `OperationSafety` | Betekent | Voorbeelden |
|---|---|---|
| `Read` | Geen neveneffecten, ook niet als de API POST of PUT gebruikt | zoeken, details, lijsten, verlengings- en verhuiscontroles, CSR-generatie, certificaatstatus |
| `Write` | Een gratis wijziging die kan worden teruggedraaid | DNS-records, vergrendeling, privacy, nameservers, contacten |
| `Billable` | Belast je saldo | registreren, verlengen, verhuizen, herstellen, SSL-aankoop en -verlenging, hostingbestelling |
| `Destructive` | Verwijdert iets of is moeilijk terug te draaien | DNS-record verwijderen, DNSSEC verwijderen, certificaat intrekken |

- **Read-only modus** (`read_only(true)` of `DNA_READ_ONLY=true`) blokkeert alles wat geen `Read` is, voordat het wordt
  verstuurd. Gebruik hem tijdens het verkennen, in rapporten en in dashboards.
- **Retries.** 429-antwoorden worden altijd herhaald, met inachtneming van `Retry-After`. Dit is veilig, zelfs voor billable
  requests, omdat de gateway ze weigert voordat er iets wordt uitgevoerd. Netwerkfouten, timeouts en 502/503/504-antwoorden
  worden alleen herhaald bij leesacties en bij de schrijfacties die de CLI als idempotent markeert (vergrendelen,
  ontgrendelen, privacy, nameservers, DNS bijwerken en toepassen). Een registratie of verlenging wordt nooit twee keer verstuurd.
- **Pacing.** De gateway staat 5 requests per seconde per sleutel toe en 2 per seconde voor domeinzoekopdrachten. De client
  spreidt requests om onder deze limieten te blijven en houdt DNS-zoneaanroepen 1 seconde uit elkaar. Domeininfo en de
  TLD-lijst vallen niet onder een rate limit.
- **Soft quota.** Zoekopdrachten en infoaanvragen kosten 1 punt, al het andere 10. Boven 200 punten vertraagt de gateway elk
  request met maximaal 5 seconden; punten nemen met 20% per minuut af. `send_with_response` toont de status in `quota`.

## Async en blokkerende code

De SDK is async en heeft een Tokio-runtime nodig (de runtime die `#[tokio::main]` start, of die van je server). De futures
zijn `Send`, dus ze werken in `tokio::spawn` en in axum- en actix-handlers. Lijsten kun je per pagina lezen (`list`) of als
`Stream` van items (`list_all`), die de volgende pagina pas ophaalt wanneer die nodig is:

```rust
use futures_util::TryStreamExt;

let expiring: Vec<Domain> = client
    .domains()
    .list_all(DomainListRequest::new())
    .try_filter(|d| std::future::ready(d.remaining_days < 30))
    .try_collect()
    .await?;
```

Er is geen blokkerende API: die zou het oppervlak verdubbelen voor weinig winst. Voer in synchrone code de aanroepen uit
op een runtime:

```rust
let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
let account = runtime.block_on(client.account().get())?;
```

## Je code testen

De SDK heeft alleen een HTTP-endpoint nodig. Om code die hem gebruikt te testen zonder de API aan te roepen, laat je
`base_url` naar een lokale server wijzen (bijvoorbeeld een die is gebouwd met `wiremock`, `httpmock` of `axum`) en zet je
pacing uit:

```rust
let client = Client::builder()
    .credentials("test", "test")
    .base_url(format!("http://{}", server_address))
    .pace_requests(false)
    .build()?;
```

De modellen implementeren `Default` en `Deserialize`, dus testgegevens kunnen uit JSON komen; fouten voor je mocks maak je
met `Error::new(kind, code, message)`. Gebruik voor tests tegen de echte API OTE met een eigen sleutel.

## Projectindeling

```
src/
├── lib.rs           Crate-documentatie en re-exports
├── client.rs        Client, ClientBuilder, Environment: versturen, retries, pacing, fouten uitlezen, list_all
├── request.rs       Request, PagedRequest, Endpoint, OperationSafety, Query, JsonBody, Validator, Page, Empty
├── error.rs         Error, ErrorKind, codes
├── de.rs            Tolerant decoderen en de macro's die modellen en enums declareren (intern)
├── account.rs       AccountClient     Saldi, profiel en transacties
├── contacts.rs      ContactsClient    Contacthandles
├── domains/         DomainsClient     Domeinen, plus forwarding(), child_name_servers() en documents()
├── dns.rs           DnsClient         Zonerecords
├── dnssec.rs        DnssecClient      DS-records
├── ssl.rs           SslClient         Certificaten
├── hosting.rs       HostingClient     Hostingpakketten en -diensten
├── orders.rs        OrdersClient      Bestellingen
└── products.rs      ProductsClient    TLD's, SSL-producten en prijzen
tests/               Tests tegen een nep-API, de spec-dekkingstest en testdata/openapi.json
examples/            Uitvoerbare programma's
```

Elke gebiedsmodule heeft dezelfde opbouw: de modellen en enums van de antwoorden, één requesttype per operatie
(`XxxRequest`) en een `XxxClient`.

## Ontwikkeling

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                    # unit-, nep-API-, spec- en doctests; geen enkele roept de echte API aan
cargo build --examples
```

Zonder lokale toolchain gebruik je Docker:

```bash
docker run --rm -v "$PWD":/src -w /src rust:1.83 sh -c 'rustup component add rustfmt clippy; cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo test'
```

De tests roepen nooit de echte API aan: ze gebruiken een kleine HTTP-server op 127.0.0.1 (`tests/common`). CI draait ze op
de minimaal ondersteunde Rust-versie (1.83) en op de nieuwste stabiele versie.

`tests/testdata/openapi.json` is het API-document dat de `dna` CLI embedt. `tests/spec.rs` controleert of elke operatie
erin precies één requesttype heeft met de methode, het pad, de veiligheidsklasse en de idempotentie van de CLI, of elk
request alleen queryparameters en bodyvelden verstuurt die het document beschrijft (inclusief geneste objecten, waarbij
verplichte velden aanwezig moeten zijn), en of de antwoordtypen de namen uit het document gebruiken. Als de API een
operatie erbij krijgt, werk je het document bij en faalt de test totdat het requesttype bestaat.

## Upgraden van 1.x

2.0 is een nieuwe SDK. 1.x was een port van de PHP-bibliotheek: een blokkerende `DomainNameApi`-client die ongetypeerde
`serde_json::Value`-envelopes teruggaf. 2.0 is async, met getypeerde requests en modellen, `Result<T, Error>` en een builder.

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
| `result["result"] == "ERROR"` | `Err(e)` met `e.kind()` en `e.code()` |

De aanroepen voor het goedkeuren, annuleren en weigeren van verhuizingen en `sync_from_registry` uit 1.x maken geen deel
uit van het document van de REST API en hebben geen tegenhanger in 2.0. De MSRV blijft 1.83; `ureq` is vervangen door
`reqwest`, dus 2.0 heeft een Tokio-runtime nodig. 1.x stuurde foutrapporten naar Sentry; 2.0 stuurt nergens iets heen
behalve naar de API.

## Links

- [`dna` CLI en MCP-server](https://github.com/domainreseller/cli): dezelfde API vanuit de terminal en vanuit AI-agents.
- [API-documentatie](https://www.domainnameapi.com/docs/)
- [Changelog](CHANGELOG.md)

## Licentie

Apache License 2.0. Zie [LICENSE](LICENSE).
