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

Das offizielle Rust SDK für die Reseller-Plattform von [Domain Name API](https://www.domainnameapi.com). Es deckt alle
67 Operationen der REST-API mit typisierten Requests und Modellen ab: Domains, Kontakte, DNS, DNSSEC, SSL-Zertifikate,
Hosting, Bestellungen, Produkte und Ihr Konto.

```rust
let client = Client::builder().credentials(reseller_id, api_key).environment(Environment::Ote).build()?;

let search = client.domains().search("example.com").await?;
if search.info.is_available() {
    let request = DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact), ["tr.apiname.com", "eu.apiname.com"]);
    client.domains().register_with_contacts(request).await?;
}
```

- **Alles typisiert.** Jede Operation ist ein Request-Typ und jede Antwort ein dokumentiertes Struct. Keine Maps, kein JSON-Parsing.
- **Ihre eigenen Requests.** Jeder Request implementiert den Trait `Request`. Implementieren Sie ihn für einen Endpunkt, den das SDK noch nicht abdeckt.
- **Sicher im Umgang mit Geld.** Jeder Request gibt an, ob er liest, schreibt, Ihr Guthaben belastet oder etwas löscht. Kostenpflichtige Requests werden nie wiederholt, und ein Nur-Lese-Modus blockiert jede Änderung.
- **Eingebaute Ausfallsicherheit.** Rate-Limits werden automatisch eingehalten, 429-Antworten werden wiederholt, und Netzwerkfehler werden bei Lesezugriffen wiederholt.
- **Klare Fehler.** Ein einziger `Error`-Typ mit stabiler Art, dem Fehlercode der API, Validierungsfehlern auf Feldebene, der Correlation-ID und einem Hinweis.
- **Modernes Rust.** Async auf Tokio, `reqwest` mit rustls (kein OpenSSL), serde-Modelle, verzögerte Paginierung als `Stream`, ein klonbarer `Send + Sync`-Client, `#![deny(missing_docs)]`.
- **Privat.** Keine Telemetrie. Nichts verlässt Ihren Prozess außer den Requests, die Sie an die API senden.

Es ist das Rust-Pendant zu [`dna`, der Domain Name API CLI](https://github.com/domainreseller/cli): dasselbe API-Dokument,
dieselben Operationen und Gruppen, dieselben Sicherheitsklassen, Wiederholungsregeln und Fehlercodes.

## Inhalt

- [Installation](#installation)
- [Zugangsdaten](#zugangsdaten)
- [Schnellstart](#schnellstart)
- [Konfiguration](#konfiguration)
- [Verwendung](#verwendung): [Konto](#konto) · [Domains](#domains) · [Kontakte](#kontakte) · [DNS](#dns) · [DNSSEC](#dnssec) · [SSL](#ssl) · [Hosting](#hosting) · [Bestellungen und Produkte](#bestellungen-und-produkte)
- [Requests und der Trait `Request`](#requests-und-der-trait-request)
- [Fehler](#fehler)
- [Sicherheit, Wiederholungen und Rate-Limits](#sicherheit-wiederholungen-und-rate-limits)
- [Asynchroner und blockierender Code](#asynchroner-und-blockierender-code)
- [Ihren Code testen](#ihren-code-testen)
- [Projektstruktur](#projektstruktur)
- [Entwicklung](#entwicklung)
- [Umstieg von 1.x](#umstieg-von-1x)

## Installation

> [!IMPORTANT]
> **Verwenden Sie immer das [neueste Release](https://github.com/domainreseller/rust-dna/releases/latest).** Release-Pakete sind versioniert und getestet.
> Der grüne Button **Code → Download ZIP** lädt stattdessen den Entwicklungszweig herunter.

```toml
[dependencies]
domainnameapi = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"   # zum Lesen der list_all-Streams
```

Solange das Crate nicht auf crates.io veröffentlicht ist, beziehen Sie es über den Release-Tag:

```toml
domainnameapi = { git = "https://github.com/domainreseller/rust-dna", tag = "v2.1.0" }
```

**Voraussetzungen:** Rust 1.83 oder neuer und eine Tokio-Runtime. Die Abhängigkeiten sind `reqwest` (mit rustls, kein OpenSSL),
`serde`/`serde_json`, `tokio` (nur Timer), `futures-util`, `chrono`, `base64` und `httpdate`.

## Zugangsdaten

Die API authentifiziert jeden Request mit zwei Werten, die das SDK für Sie sendet:

| Wert | Header | Fundort |
|---|---|---|
| Reseller-ID (eine UUID) | `__reseller` | Reseller-Panel → **API-Einstellungen** |
| API-Key | `X-API-KEY` | Reseller-Panel → **API-Einstellungen** |

> [!NOTE]
> Dies sind **API-Zugangsdaten**: Ihre Panel-Anmeldung funktioniert hier nicht. **Produktion und OTE (die Testumgebung) verwenden
> unterschiedliche API-Keys.** Wird der eine mit der jeweils anderen Umgebung verwendet, schlägt der Aufruf mit `INVALID_API_KEY` oder `INVALID_RESELLER` fehl.

## Schnellstart

```rust
use domainnameapi::contacts::Contact;
use domainnameapi::domains::{DomainContacts, DomainListRequest, DomainRegisterWithContactsRequest};
use domainnameapi::{Client, Environment};
use futures_util::TryStreamExt;

#[tokio::main]
async fn main() -> Result<(), domainnameapi::Error> {
    // OTE ist die Testumgebung: echte API, eigene Daten, kein echtes Geld.
    let client = Client::builder()
        .credentials("your-reseller-id", "your-api-key")
        .environment(Environment::Ote)
        .build()?;

    // 1. Zugangsdaten und Guthaben prüfen.
    let account = client.account().get().await?;
    println!("{}: {} USD", account.reseller_name, account.usd_balance);

    // 2. Ist die Domain verfügbar, und zu welchem Preis?
    let search = client.domains().search("my-new-domain.com").await?;
    println!("{} {} {}", search.info.status, search.info.price, search.info.currency);

    // 3. Mit einem Kontakt für alle vier Rollen registrieren. Dies belastet das Guthaben.
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

    // 4. Alle Domains auflisten; Seiten werden abgerufen, während der Stream gelesen wird.
    let mut domains = client.domains().list_all(DomainListRequest::new());
    while let Some(domain) = domains.try_next().await? {
        println!("{} expires in {} days", domain.domain_name, domain.remaining_days);
    }
    Ok(())
}
```

Der Ordner [examples](examples) enthält lauffähige Programme für diese Szenarien (`quickstart`, `register`, `dns`, `ssl`,
`custom_request`, `error_handling`). Sie lesen die Umgebungsvariablen `DNA_*`, richten Sie sie also auf OTE aus:

```bash
export DNA_RESELLER_ID=... DNA_API_KEY=... DNA_ENV=ote
cargo run --example quickstart -- example.com
```

## Konfiguration

Es gibt drei Wege, einen Client zu erstellen:

```rust
// Produktion, Standardeinstellungen
let client = Client::new(reseller_id, api_key)?;

// Alle Einstellungen
let client = Client::builder()
    .credentials(reseller_id, api_key)
    .environment(Environment::Ote)
    .timeout(Duration::from_secs(30))
    .read_only(true)
    .build()?;

// Dieselben Umgebungsvariablen, die die dna CLI liest ...
let client = Client::from_env()?;

// ... plus Ihre eigenen Einstellungen
let client = ClientBuilder::from_env()?.http_client(my_reqwest_client).build()?;
```

| `ClientBuilder`-Methode | Standard | Bedeutung |
|---|---|---|
| `credentials(reseller_id, api_key)` | (erforderlich) | Die Zugangsdaten. `build()` schlägt mit `NO_CREDENTIALS` fehl, wenn einer der Werte leer ist. |
| `environment(...)` | `Production` | `Production` (`https://api.domainresellerapi.com`) oder `Ote` (`https://ote.domainresellerapi.com`). |
| `base_url(...)` | keine | Überschreibt die Adresse der Umgebung, zum Beispiel für einen Proxy. Ein Pfadpräfix bleibt erhalten. |
| `timeout(...)` | 60 s | Zeitlimit eines einzelnen Versuchs. |
| `max_retries(...)` | 2 | Wie oft ein fehlgeschlagener Request wiederholt wird (siehe [Wiederholungen](#sicherheit-wiederholungen-und-rate-limits)). 0 schaltet Wiederholungen aus. |
| `max_retry_delay(...)` | 30 s | Die längste Wartezeit vor einer Wiederholung. |
| `pace_requests(...)` | `true` | Verteilt Requests zeitlich, um unter den Rate-Limits des Gateways zu bleiben. |
| `read_only(...)` | `false` | Blockiert vor dem Senden jeden Request, der kein Lesezugriff ist. |
| `user_agent(...)` | `DomainNameAPI-Rust/<version>` | Der `User-Agent`-Header. |
| `app(name, version)` | `rust`, die SDK-Version | Die Header `X-App` und `X-Version`: die Anwendung, die das SDK nutzt (siehe unten). |
| `http_client(...)` | der eigene des SDK | Ihr eigener `reqwest::Client` (Proxy, TLS-Einstellungen). |

`from_env()` liest:

| Variable | Bedeutung |
|---|---|
| `DNA_RESELLER_ID`, `DNA_API_KEY` | Die Zugangsdaten. |
| `DNA_ENV` | `production` oder `ote` (auch `prod`, `live`, `test`, `sandbox`). |
| `DNA_TEST_MODE` | `true`, `1`, `on` oder `yes` wählt OTE. Wird verwendet, wenn `DNA_ENV` nicht gesetzt ist. |
| `DNA_BASE_URL` | Optionale Überschreibung der Basisadresse. |
| `DNA_READ_ONLY` | `true` schaltet den Nur-Lese-Modus ein. |
| `DNA_APP`, `DNA_APP_VERSION` | Die Anwendung, die das SDK nutzt, und ihre Version, gesendet als `X-App` und `X-Version`. |

`Client` ist `Clone + Send + Sync` und günstig zu klonen (er hält einen `Arc`). Erstellen Sie pro Satz Zugangsdaten einen
Client und teilen Sie ihn für die Lebensdauer der Anwendung: Das Pacing für die Rate-Limits gilt pro Client.

### Ihre Anwendung kennzeichnen

Jede Anfrage trägt die Header `X-App` und `X-Version`, an denen das API-Team erkennt, welche Integration sie gesendet
hat. Standardmäßig sind das `rust` und die SDK-Version. Wenn Sie ein Panel-Modul oder ein anderes Produkt auf dem SDK
aufbauen, setzen Sie Ihre eigenen Werte:

```rust
let client = Client::builder()
    .credentials(reseller_id, api_key)
    .app("WiseCP5", "V2.0.20-beta") // gesendet als X-App: wisecp5, X-Version: 2.0.20
    .build()?;
```

Der Name wird kleingeschrieben und behält nur `a-z`, `0-9`, `.`, `_` und `-`; die Version behält nur Ziffern und Punkte.
Ein leerer Name fällt auf `rust` zurück, eine leere Version auf die SDK-Version. `from_env()` liest dieselben Werte aus
`DNA_APP` und `DNA_APP_VERSION`. Der `User-Agent`-Header bleibt unverändert.

## Verwendung

Jeder Bereich der API hat einen eigenen Client, benannt wie die Befehlsgruppen der CLI: `client.account()`,
`client.contacts()`, `client.domains()` (mit `forwarding()`, `child_name_servers()` und `documents()`),
`client.dns()`, `client.dnssec()`, `client.ssl()`, `client.hosting()`, `client.orders()` und `client.products()`.
Pflichtangaben sind Argumente; Operationen mit optionalen Angaben nehmen einen Request entgegen, der mit `new(...)` und
verketteten Settern erstellt wird. Methoden, die Ihr Guthaben belasten, weisen in ihrer Dokumentation darauf hin.

### Konto

```rust
let account = client.account().get().await?;          // reseller_id, reseller_name, try_balance, usd_balance
let profile = client.account().profile().await?;      // name, company_name, account_currency, status, ...

// Die Belastungen dieses Monats, über alle Seiten
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

**Suche**

```rust
let one = client.domains().search("example.com").await?;
let available = one.success && one.info.is_available();

let many = client.domains().bulk_search(["example.com", "example.net", "example.com.tr"]).await?;
for domain in many.available() {
    println!("{}: {} {}", domain.domain_name, domain.price, domain.currency);
}

// Mit einer Währung
client.domains().search_with(DomainSearchRequest::new("example.com").currency("USD")).await?;
```

**Liste und Details**

```rust
let page = client.domains().list(DomainListRequest::new().limit(50).statuses([DomainStatus::Active])).await?;

let all: Vec<Domain> = client.domains().list_all(DomainListRequest::new().domain_name("example")).try_collect().await?;

let info = client.domains().info("example.com").await?;
println!("{} {:?}", info.auth_code, info.expiration_date);
if let Some(registrant) = info.contact(ContactType::Registrant) {
    println!("{}", registrant.email);
}
```

**Registrieren, verlängern, transferieren, wiederherstellen** (diese belasten Ihr Guthaben)

```rust
// Ein Kontakt für alle vier Rollen, zwei Jahre
let request = DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact.clone()), ["tr.apiname.com", "eu.apiname.com"])
    .years(2);
client.domains().register_with_contacts(request).await?;

// Unterschiedliche Kontakte pro Rolle, Registry-Attribute, der Treuhanddienst
let request = DomainRegisterWithContactsRequest::new(
    "example.com.tr",
    DomainContacts { registrant: Some(owner), admin: Some(staff.clone()), tech: Some(staff.clone()), billing: Some(staff) },
    ["tr.apiname.com", "eu.apiname.com"],
)
.tld_attribute("<key>", "<value>"); // die Schlüssel: client.products().tld("com.tr").await?.unwrap().attributes

// Mit bestehenden Kontakt-Handles
let request = DomainRegisterRequest::new("example.com", ContactHandles::same("DNA-12345"), ["tr.apiname.com", "eu.apiname.com"]);
client.domains().register(request).await?;

let renewal = client.domains().renew_check("example.com", Some(1)).await?;   // kostenlos
if renewal.renewable {
    client.domains().renew("example.com", 1).await?;
}

let check = client.domains().transfer_check("example.com", "EPP-CODE").await?;  // kostenlos
if check.transferable {
    client.domains().transfer(DomainTransferRequest::new("example.com", "EPP-CODE")).await?;
}

client.domains().restore("example.com").await?;
```

Requests werden vor dem Senden geprüft. Eine Registrierung ohne alle vier Kontaktrollen, ein Kontakt ohne E-Mail-Adresse
oder eine Laufzeit von 0 Jahren gibt einen `ErrorKind::InvalidInput`-Fehler zurück, der jedes Problem auflistet, und nichts
erreicht die API.

> [!NOTE]
> Die API kann eine Registrierung, Verlängerung oder einen Transfer mit HTTP 200 und `"success": false` beantworten (zum Beispiel, wenn die
> Registry die Domain ablehnt). Das SDK gibt diese Antwort unverändert zurück, wie das Java SDK: Prüfen Sie
> `DomainOperationResult::success` und `message`. Dasselbe gilt für `SslOperationResult` und `ContactCreateResult`.

**Einstellungen**

```rust
client.domains().lock("example.com").await?;
client.domains().unlock("example.com").await?;
client.domains().set_privacy("example.com", true).await?;
client.domains()
    .set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net", "ns2.example.net"]))
    .await?;
client.domains()   // eine mit DNSSEC signierte Domain
    .set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net"]).dnssec_mode(DnssecMode::DisableDnssec))
    .await?;
let auth_code = client.domains().update_auth_code("example.com").await?;

client.domains().update_contacts("example.com", DomainContacts::same(contact)).await?;
client.domains().set_contact_handles("example.com", ContactHandles { tech: "DNA-12345".into(), ..Default::default() }).await?;
```

**URL-Weiterleitung, Child-Nameserver und Dokumente**

```rust
client.domains().forwarding().set("example.com", "https://www.example.net", ForwardType::Standard).await?;
let forwarding = client.domains().forwarding().get("example.com").await?;   // None, wenn nicht gesetzt
client.domains().forwarding().delete("example.com").await?;

// Die IP-Version jeder Adresse wird erkannt
client.domains().child_name_servers().add("example.com", "ns1.example.com", ["192.0.2.10", "2001:db8::10"]).await?;
client.domains().child_name_servers()
    .update(DomainHostUpdateRequest::new("example.com", "ns1.example.com", ["192.0.2.20"]))
    .await?;
client.domains().child_name_servers().delete("example.com", "ns1.example.com").await?;

// Registry-Dokumente (meist .tr)
let document = DocumentUpload::from_file(DocumentType::IdentityCard, "id.pdf")?;
client.domains().documents().upload("example.com.tr", [document]).await?;
let documents: Vec<Document> = client.domains().documents().list_all("example.com.tr").try_collect().await?;
```

### Kontakte

Kontakt-Handles sind wiederverwendbare Kontakte. Legen Sie einen einmal an und verwenden Sie ihn für viele Domains.

```rust
let created = client.contacts().create(contact.clone()).await?;
let handle = created.handle;                                         // zum Beispiel DNA-12345

let details = client.contacts().get(&handle).await?;                 // der Kontakt und sein Verifizierungsstatus
let result = client.contacts().update(&handle, contact).await?;
println!("{} of {} domains updated", result.stats.success_count, result.stats.total_domain_count);

let verifications = client.contacts().verification(&handle).await?;
client.contacts().resend_verification(&handle).await?;
```

### DNS

Zoneneinträge von Domains auf den Nameservern der Plattform. Die Einträge sind öffentlich auflösbar, sobald die Domain diese Nameserver verwendet,
worum sich `apply` kümmert.

```rust
for set in client.dns().list("example.com").await? {
    println!("{} {} {} {:?}", set.name, set.record_type, set.ttl, set.values());
}

let dns = client.dns();
dns.add(DnsAddRequest::new("example.com", "www", RecordType::A, ["192.0.2.10"])).await?;
dns.add(DnsAddRequest::new("example.com", "@", "txt", ["v=spf1 include:_spf.example.net ~all"])).await?;   // "txt" wird zu TXT
dns.add(DnsAddRequest::new("example.com", "@", RecordType::Mx, ["10 mx1.example.net", "20 mx2.example.net"]).ttl(3600)).await?;
dns.update(DnsUpdateRequest::new("example.com", "www", RecordType::A, ["192.0.2.20"]).ttl(300)).await?;
dns.delete(DnsDeleteRequest::new("example.com", "www", RecordType::A, "192.0.2.20")).await?;

let applied = dns.apply("example.com").await?;
```

Das SDK prüft, was die API ablehnen würde: einen CNAME am Apex, einen MX ohne seine Priorität, einen CAA-Wert ohne seine
drei Bestandteile, einen A-Eintrag, der keine IPv4-Adresse ist, eine TTL außerhalb von 1–86400. Zonenaufrufe werden im
Abstand von 1 Sekunde gesendet, dem Limit des Gateways. Die OTE-Testumgebung beantwortet Zonenaufrufe mit HTTP 500, daher
lässt sich DNS-Code nur in der Produktion ausprobieren.

### DNSSEC

```rust
let info = client.dnssec().get("example.com").await?;      // status, ds_records, allowed_algorithms, ...

let record: DsData = "12345 13 2 49FD46E6C4B45C55D4AC69CBD3CD34AC1AFE51DE".parse()?;
client.dnssec().add("example.com", [record.clone()]).await?;
client.dnssec().remove("example.com", record).await?;
client.dnssec().disable("example.com").await?;            // entfernt alle DS-Einträge
```

### SSL

Eine Domain kann mehrere Zertifikate haben. Ein bestehendes Zertifikat wird deshalb über seine **Zertifikats-ID** (eine GUID)
angesprochen, nie über den Domainnamen. `initialize` und `renew` geben die ID zurück: Speichern Sie sie. `list` findet die IDs
der Zertifikate, die Sie bereits haben.

```rust
// 1. Kaufen (belastet das Guthaben). Paketcodes kommen aus client.products().ssls(...).
let approvers = client.ssl().approvers(SslApproversRequest::for_purchase("example.com", "PKG-CODE")).await?;
let order = client.ssl().initialize("example.com", "PKG-CODE").await?;
let certificate_id = order.certificate_id;                  // speichern: Jeder spätere Aufruf benötigt sie

// 2. Ein CSR und ein privater Schlüssel, falls Sie keine haben
let csr = client.ssl().generate_csr(
    SslGenerateCsrRequest::new("example.com")
        .country("TR").state("Istanbul").locality("Istanbul").organization("Example Ltd").organization_unit("IT"),
).await?;

// 3. Innerhalb von 15 Tagen konfigurieren: CSR, Validierungsmethode und Kontakt
client.ssl().configure(
    SslConfigureRequest::new(&certificate_id, &csr.csr, certificate_contact.clone())
        .authentication_type(AuthenticationType::Email)
        .approver_mail(&approvers[0])
        .web_server_type(WebServerType::Apache),
).await?;

// 4. Auf die Ausstellung warten, dann herunterladen
let status = client.ssl().status(&certificate_id).await?;
if status.is_issued {
    let content = client.ssl().content(&certificate_id).await?;
    std::fs::write("fullchain.pem", content.full_chain())?;
}

let details = client.ssl().get(&certificate_id).await?;     // Abonnement, Konfiguration, Kontakte
client.ssl().reissue(SslReissueRequest::new(&certificate_id).csr(new_csr).private_key(new_key)).await?;
client.ssl().revoke(&certificate_id).await?;                // kann nicht rückgängig gemacht werden
```

**Zertifikate auflisten und eine ID finden**

```rust
let page = client.ssl().list(SslListRequest::new().domain_name("example.com")).await?;   // neueste zuerst

let mut all = client.ssl().list_all(SslListRequest::new());
while let Some(c) = all.try_next().await? {
    println!("{} {} {} {:?}", c.certificate_id, c.domain_name, c.status, c.subscription_expires_at);
}

// Die Genehmiger-Adressen eines Zertifikats, das Sie bereits haben
let mails = client.ssl().approvers(SslApproversRequest::for_certificate(&certificate_id)).await?;
```

**Verlängern** (dies belastet Ihr Guthaben)

```rust
// Erlaubt ab 30 Tagen vor details.subscription.expires_at. Die Verlängerung ist ein neues Zertifikat mit eigener ID.
let renewal = client.ssl().renew(&certificate_id).await?;
client.ssl().configure(
    SslConfigureRequest::new(&renewal.certificate_id, &new_csr, certificate_contact)
        .authentication_type(AuthenticationType::Email)
        .approver_mail(&approvers[0]),
).await?;
```

Ein gekauftes oder verlängertes Zertifikat muss innerhalb von 15 Tagen konfiguriert werden; danach wird es storniert.
Zum Verlängern rufen Sie `renew` und anschließend `configure` mit der neuen ID auf. Kaufen Sie nicht erneut mit
`initialize`: Das ist ein eigenständiger Kauf zum Kaufpreis, und das alte Zertifikat wird nie als verlängert markiert.
Eine Zertifikats-ID, die leer oder keine GUID ist (zum Beispiel ein Domainname, den älterer Code übergibt), wird
abgelehnt, bevor etwas gesendet wird.

### Hosting

```rust
let packages = client.hosting().packages().await?;
let created = client.hosting().create("example.com", "starter-linux").await?;          // belastet das Guthaben
let status = client.hosting().status(HostingStatusRequest::by_service_id(&created.service_id)).await?;
if status.is_active() {
    println!("{} {}", status.panel_address, status.username);
}
let hosting = client.hosting().get(HostingGetRequest::by_domain("example.com")).await?;
let plans = client.hosting().plans(None).await?;
```

### Bestellungen und Produkte

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

## Requests und der Trait `Request`

Jede Operation ist ein Request-Typ, benannt nach ihrem CLI-Befehl: `dna domain renew` ist `DomainRenewRequest`,
`dna domain renew-check` ist `DomainRenewCheckRequest`, `dna dns add` ist `DnsAddRequest`. Die Bereichsmethoden erstellen
sie für Sie, und Sie können sie auch selbst erstellen und senden:

```rust
let request = DomainRenewRequest::new("example.com", 2);
let endpoint = request.endpoint();
println!("{endpoint} is {}", endpoint.safety);   // "POST /api/v1/domains/renew is billable"

let result = client.send(&request).await?;

// Dasselbe, mit den HTTP-Details: Correlation-ID, Soft-Quota-Zustand, Versuche, Dauer
let response = client.send_with_response(&request).await?;
println!("{} quota {:?}", response.correlation_id, response.quota);
```

Rust kennt keine Vererbung; die Grundlage jedes Requests ist ein Trait:

| Trait | Wofür |
|---|---|
| `Request` | Jede Operation. `type Response` ist der Antworttyp; `Empty` für Operationen, die nichts zurückgeben (sperren, entsperren, einen DNS-Eintrag hinzufügen, ...). |
| `PagedRequest<T>` | Listenoperationen (`Response = Page<T>`). Gibt Zugriff auf das `Paging` (skip, limit, sort) und funktioniert mit `Client::list_all`. |

Ein Request beschreibt sich selbst über `endpoint()`: Methode, Pfad, Sicherheitsklasse und ob er idempotent ist. Die anderen
drei Methoden haben Standardimplementierungen: `query()` gibt den Query-String zurück (`Query` lässt leere Werte weg), `body()`
den JSON-Body (`JsonBody` erstellt einen und lässt leere Werte weg, oder verwenden Sie `serde_json::json!`), und `validate()`
sammelt Probleme vor dem Senden. Um einen Endpunkt aufzurufen, den das SDK noch nicht abdeckt, schreiben Sie einen eigenen Request:

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
        Endpoint::get("/api/v1/domains/notes")          // GET ist ein Lesezugriff
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

Ihr Request erhält alles, was die eingebauten erhalten: Zugangsdaten, Wiederholungen nach Sicherheitsklasse, Pacing,
Nur-Lese-Modus und Fehlerbehandlung. Für eine Änderung verwenden Sie `Endpoint::post(path, OperationSafety::Write)` (oder
`Billable`, `Destructive`) und ergänzen `.idempotent()` nur, wenn zweimaliges Senden unschädlich ist.

## Fehler

Jeder fehlgeschlagene Request gibt einen `Error` zurück, auch wenn der Request seine Prüfungen nicht bestanden hat und nichts gesendet wurde.

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
        // Ein kostenpflichtiger oder ändernder Request ist im Netzwerk fehlgeschlagen: Er hat die API möglicherweise erreicht.
        // Prüfen Sie den Zustand, bevor Sie ihn erneut senden.
        let info = client.domains().info("example.com").await?;
    }
    Err(e) => return Err(e.into()),
}
```

| Methode | Bedeutung |
|---|---|
| `kind()` | Eine stabile Klassifizierung (Tabelle unten). Verzweigen Sie danach, nicht nach der Meldung. |
| `code()` | Der Fehlercode der API (etwa `Dna.DomainService:Domain:10007`) oder einer der unten genannten Codes des SDK. Die gängigen sind Konstanten in `domainnameapi::codes`. |
| `message()` | Was passiert ist. Standardmeldungen werden durch die Details der API ersetzt. |
| `status()` | Der HTTP-Status, oder `None`, wenn keine Antwort eingetroffen ist. |
| `details()`, `validation_errors()` | Zusätzliche Details und Probleme auf Feldebene (von der API oder aus den Prüfungen vor dem Senden). |
| `correlation_id()` | Die `X-Correlation-Id` des Gateways. Nennen Sie sie, wenn Sie den Support kontaktieren. |
| `hint()` | Ein kurzer Vorschlag, was als Nächstes zu tun ist. |
| `is_retryable()` | Ob die spätere Wiederholung desselben Requests erfolgreich sein kann. |
| `is_outcome_unknown()` | Der Request wurde möglicherweise ausgeführt: Eine Änderung ist im Netzwerk fehlgeschlagen, oder die API hat gemeldet, dass die Registry sie möglicherweise bereits durchgeführt hat. |
| `is_not_found()`, `is_insufficient_balance()` | Abkürzungen für häufige Fälle. |
| `operation()`, `safety()` | Der fehlgeschlagene Request (`POST /api/v1/domains/renew`) und seine Sicherheitsklasse. |
| `retry_after()`, `response_body()` | Das `Retry-After` des Gateways und die rohe Antwort. |

`Error` implementiert `std::error::Error` und `Display` (`[CODE] message (HTTP 400); field: problem`) und funktioniert
daher mit `?`, `anyhow` und `thiserror`.

| `kind()` | Wann |
|---|---|
| `Auth` | Falsche oder fehlende Zugangsdaten oder ein unbekannter oder gesperrter Reseller in dieser Umgebung (401, Weiterleitung). |
| `Forbidden` | Authentifiziert, aber nicht berechtigt (403). |
| `NotFound` | Die Domain, der Kontakt, das Zertifikat, die Bestellung oder eine andere Ressource existiert nicht (404). |
| `Validation` | Die API hat die Eingabe abgelehnt oder eine Geschäftsregel ist fehlgeschlagen: ungültiges Feld, Domain gesperrt, unzureichendes Guthaben (400, 422). |
| `Conflict` | Der Zustand der Ressource lässt die Operation nicht zu (409). |
| `RateLimited` | Auch nach den automatischen Wiederholungen weiterhin durch das Rate-Limit begrenzt (429). |
| `Server` | Ein serverseitiger Fehler (5xx). Oft vorübergehend. |
| `Network`, `Timeout` | Keine Antwort: DNS, TLS, Proxy, zurückgesetzte Verbindung oder keine Antwort innerhalb des Timeouts. |
| `InvalidInput` | Der Request hat seine Prüfungen nicht bestanden, oder eine Client-Einstellung ist ungültig; nichts wurde gesendet. |
| `ReadOnly` | Vom Nur-Lese-Modus blockiert; nichts wurde gesendet. |
| `Unknown` | Alles andere, einschließlich einer Antwort, die das SDK nicht lesen konnte. |

`ErrorKind::as_str()` liefert die Namen der CLI (`auth`, `not_found`, `rate_limited`, ...). Codes, die das SDK selbst setzt:

| Code | Bedeutung |
|---|---|
| `INVALID_RESELLER` | Das Gateway hat auf seine Sperrseite weitergeleitet: Die Reseller-ID ist **in dieser Umgebung** unbekannt, inaktiv oder gesperrt. |
| `INVALID_API_KEY` | Der Key passt nicht zum Reseller. Keys für Produktion und OTE unterscheiden sich. |
| `MISSING_API_KEY`, `MISSING_RESELLER_ID`, `NO_CREDENTIALS` | Ein Zugangswert ist leer. |
| `SUBRESELLER_NOT_ALLOWED` | Sub-Reseller-Konten können die API nicht verwenden. Verwenden Sie die Zugangsdaten des Haupt-Resellers. |
| `UNAUTHORIZED` | Jedes andere 401. |
| `RATE_LIMITED` | 429 nach den Wiederholungen. |
| `NETWORK`, `TIMEOUT` | Es ist keine Antwort eingetroffen. |
| `READ_ONLY` | Vom Nur-Lese-Modus blockiert. |
| `INVALID_INPUT`, `INVALID_CONFIGURATION` | Ein Request hat seine Prüfungen nicht bestanden, oder eine Client-Einstellung ist ungültig. |
| `EMPTY_RESPONSE`, `INVALID_RESPONSE` | Eine erfolgreiche Antwort war leer oder konnte nicht gelesen werden. |
| `HTTP_<status>` | Die API hat keinen Fehlercode gesendet. |

## Sicherheit, Wiederholungen und Rate-Limits

Jeder Request hat eine Sicherheitsklasse, dieselbe, die die `dna` CLI verwendet:

| `OperationSafety` | Bedeutet | Beispiele |
|---|---|---|
| `Read` | Keine Seiteneffekte, auch wenn die API POST oder PUT verwendet | Suche, Details, Listen, Verlängerungs- und Transferprüfungen, CSR-Erzeugung, Zertifikatsstatus |
| `Write` | Eine kostenlose Änderung, die sich rückgängig machen lässt | DNS-Einträge, Sperre, Datenschutz, Nameserver, Kontakte |
| `Billable` | Belastet Ihr Guthaben | Registrierung, Verlängerung, Transfer, Wiederherstellung, SSL-Kauf und -Verlängerung, Hosting-Bestellung |
| `Destructive` | Löscht etwas oder lässt sich kaum rückgängig machen | Löschen von DNS-Einträgen, Entfernen von DNSSEC, Widerruf von Zertifikaten |

- **Nur-Lese-Modus** (`read_only(true)` oder `DNA_READ_ONLY=true`) blockiert vor dem Senden alles, was kein `Read` ist.
  Verwenden Sie ihn beim Erkunden, in Berichten und in Dashboards.
- **Wiederholungen.** 429-Antworten werden immer wiederholt, wobei `Retry-After` beachtet wird. Das ist auch bei kostenpflichtigen
  Requests sicher, weil das Gateway sie ablehnt, bevor etwas ausgeführt wird. Netzwerkfehler, Timeouts und 502/503/504-Antworten
  werden nur bei Lesezugriffen wiederholt und bei den Schreibzugriffen, die die CLI als idempotent kennzeichnet (Sperren,
  Entsperren, Datenschutz, Nameserver, DNS-Aktualisierung und -Anwendung). Eine Registrierung oder Verlängerung wird nie zweimal gesendet.
- **Pacing.** Das Gateway erlaubt 5 Requests pro Sekunde und Key und 2 pro Sekunde für Domainsuchen. Der Client verteilt
  Requests zeitlich, um unter diesen Limits zu bleiben, und sendet DNS-Zonenaufrufe im Abstand von 1 Sekunde. Domain-Infos
  und die TLD-Liste unterliegen keinem Rate-Limit.
- **Soft-Quota.** Suchen und Info-Abfragen kosten 1 Punkt, alles andere 10. Ab 200 Punkten verlangsamt das Gateway jeden
  Request um bis zu 5 Sekunden; die Punkte verfallen um 20 % pro Minute. `send_with_response` zeigt den Zustand in `quota`.

## Asynchroner und blockierender Code

Das SDK ist asynchron und benötigt eine Tokio-Runtime (die von `#[tokio::main]` gestartete oder die Ihres Servers). Die
Futures sind `Send` und funktionieren daher in `tokio::spawn` sowie in axum- und actix-Handlern. Listen lassen sich Seite für
Seite (`list`) oder als `Stream` von Elementen (`list_all`) lesen, der die nächste Seite erst abruft, wenn sie benötigt wird:

```rust
use futures_util::TryStreamExt;

let expiring: Vec<Domain> = client
    .domains()
    .list_all(DomainListRequest::new())
    .try_filter(|d| std::future::ready(d.remaining_days < 30))
    .try_collect()
    .await?;
```

Es gibt keine blockierende API: Sie würde die Schnittstelle für wenig Nutzen verdoppeln. In synchronem Code führen Sie die
Aufrufe auf einer Runtime aus:

```rust
let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
let account = runtime.block_on(client.account().get())?;
```

## Ihren Code testen

Das SDK benötigt nur einen HTTP-Endpunkt. Um Code, der es verwendet, zu testen, ohne die API aufzurufen, richten Sie `base_url`
auf einen lokalen Server (zum Beispiel einen mit `wiremock`, `httpmock` oder `axum` erstellten) und schalten das Pacing aus:

```rust
let client = Client::builder()
    .credentials("test", "test")
    .base_url(format!("http://{}", server_address))
    .pace_requests(false)
    .build()?;
```

Die Modelle implementieren `Default` und `Deserialize`, Testdaten können also aus JSON stammen; Fehler für Ihre Mocks lassen
sich mit `Error::new(kind, code, message)` erstellen. Für Tests gegen die echte API verwenden Sie OTE mit dem zugehörigen eigenen Key.

## Projektstruktur

```
src/
├── lib.rs           Crate-Dokumentation und Re-Exports
├── client.rs        Client, ClientBuilder, Environment: Senden, Wiederholungen, Pacing, Fehlerauswertung, list_all
├── request.rs       Request, PagedRequest, Endpoint, OperationSafety, Query, JsonBody, Validator, Page, Empty
├── error.rs         Error, ErrorKind, codes
├── de.rs            Tolerantes Dekodieren und die Makros, die Modelle und Enums deklarieren (intern)
├── account.rs       AccountClient     Guthaben, Profil und Transaktionen
├── contacts.rs      ContactsClient    Kontakt-Handles
├── domains/         DomainsClient     Domains, plus forwarding(), child_name_servers() und documents()
├── dns.rs           DnsClient         Zoneneinträge
├── dnssec.rs        DnssecClient      DS-Einträge
├── ssl.rs           SslClient         Zertifikate
├── hosting.rs       HostingClient     Hosting-Pakete und -Dienste
├── orders.rs        OrdersClient      Bestellungen
└── products.rs      ProductsClient    TLDs, SSL-Produkte und Preise
tests/               Tests gegen eine gefälschte API, der Spec-Abdeckungstest und testdata/openapi.json
examples/            Lauffähige Programme
```

Jedes Bereichsmodul hat denselben Aufbau: die Modelle und Enums seiner Antworten, einen Request-Typ pro Operation
(`XxxRequest`) und einen `XxxClient`.

## Entwicklung

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                    # Unit-, Fake-API-, Spec- und Doc-Tests; keiner ruft die echte API auf
cargo build --examples
```

Ohne lokale Toolchain verwenden Sie Docker:

```bash
docker run --rm -v "$PWD":/src -w /src rust:1.83 sh -c 'rustup component add rustfmt clippy; cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo test'
```

Die Tests rufen nie die echte API auf: Sie verwenden einen kleinen HTTP-Server auf 127.0.0.1 (`tests/common`). Die CI führt
sie auf der minimal unterstützten Rust-Version (1.83) und auf der neuesten stabilen Version aus.

`tests/testdata/openapi.json` ist das API-Dokument, das die `dna` CLI einbettet. `tests/spec.rs` prüft, dass jede darin
enthaltene Operation genau einen Request-Typ mit der Methode, dem Pfad, der Sicherheitsklasse und der Idempotenz der CLI hat,
dass jeder Request nur Query-Parameter und Body-Eigenschaften sendet, die das Dokument beschreibt (einschließlich verschachtelter
Objekte, wobei erforderliche vorhanden sein müssen), und dass die Antworttypen die Namen des Dokuments verwenden. Wenn die API
eine Operation erhält, aktualisieren Sie das Dokument, und der Test schlägt so lange fehl, bis der Request-Typ existiert.

## Umstieg von 1.x

2.0 ist ein neues SDK. 1.x war eine Portierung der PHP-Bibliothek: ein blockierender `DomainNameApi`-Client, der untypisierte
`serde_json::Value`-Umschläge zurückgab. 2.0 ist asynchron, mit typisierten Requests und Modellen, `Result<T, Error>` und einem Builder.

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
| `result["result"] == "ERROR"` | `Err(e)` mit `e.kind()` und `e.code()` |

Die Aufrufe für Transfer-Genehmigung, -Abbruch und -Ablehnung sowie `sync_from_registry` aus 1.x sind nicht Teil des
REST-API-Dokuments und haben in 2.0 keine Entsprechung. Die MSRV bleibt 1.83; `ureq` wird durch `reqwest` ersetzt, daher
benötigt 2.0 eine Tokio-Runtime. 1.x hat Fehlerberichte an Sentry gesendet; 2.0 sendet nirgendwohin etwas, außer an die API.

## Links

- [`dna` CLI und MCP-Server](https://github.com/domainreseller/cli): dieselbe API vom Terminal und von KI-Agenten aus.
- [API-Dokumentation](https://www.domainnameapi.com/docs/)
- [Changelog](CHANGELOG.md)

## Lizenz

Apache License 2.0. Siehe [LICENSE](LICENSE).
