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

L'SDK Rust ufficiale per la piattaforma reseller di [Domain Name API](https://www.domainnameapi.com). Copre tutte le
67 operazioni dell'API REST con richieste e modelli tipizzati: domini, contatti, DNS, DNSSEC, certificati SSL,
hosting, ordini, prodotti e il tuo account.

```rust
let client = Client::builder().credentials(reseller_id, api_key).environment(Environment::Ote).build()?;

let search = client.domains().search("example.com").await?;
if search.info.is_available() {
    let request = DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact), ["tr.apiname.com", "eu.apiname.com"]);
    client.domains().register_with_contacts(request).await?;
}
```

- **Tutto tipizzato.** Ogni operazione è un tipo di richiesta e ogni risposta una struct documentata. Niente mappe, niente parsing JSON.
- **Le tue richieste.** Ogni richiesta implementa il trait `Request`. Implementalo per un endpoint che l'SDK non copre ancora.
- **Sicuro con il denaro.** Ogni richiesta dichiara se legge, scrive, addebita il tuo saldo o elimina qualcosa. Le richieste a pagamento non vengono mai ripetute e la modalità di sola lettura blocca qualsiasi modifica.
- **Resilienza integrata.** I limiti di richieste vengono rispettati automaticamente, le risposte 429 vengono ritentate e gli errori di rete vengono ritentati per le letture.
- **Errori chiari.** Un solo tipo `Error` con un tipo stabile, il codice di errore dell'API, gli errori di validazione per campo, il correlation ID e un suggerimento.
- **Rust moderno.** Async su Tokio, `reqwest` con rustls (senza OpenSSL), modelli serde, paginazione lazy come `Stream`, un client clonabile `Send + Sync`, `#![deny(missing_docs)]`.
- **Riservato.** Nessuna telemetria. Nulla lascia il tuo processo, tranne le richieste che invii all'API.

È la controparte Rust di [`dna`, la CLI di Domain Name API](https://github.com/domainreseller/cli): stesso documento
API, stesse operazioni e gruppi, stesse classi di sicurezza, regole di retry e codici di errore.

## Indice

- [Installazione](#installazione)
- [Credenziali](#credenziali)
- [Avvio rapido](#avvio-rapido)
- [Configurazione](#configurazione)
- [Utilizzo](#utilizzo): [account](#account) · [domini](#domini) · [contatti](#contatti) · [DNS](#dns) · [DNSSEC](#dnssec) · [SSL](#ssl) · [hosting](#hosting) · [ordini e prodotti](#ordini-e-prodotti)
- [Richieste e il trait `Request`](#richieste-e-il-trait-request)
- [Errori](#errori)
- [Sicurezza, retry e limiti di richieste](#sicurezza-retry-e-limiti-di-richieste)
- [Codice asincrono e bloccante](#codice-asincrono-e-bloccante)
- [Testare il tuo codice](#testare-il-tuo-codice)
- [Struttura del progetto](#struttura-del-progetto)
- [Sviluppo](#sviluppo)
- [Passare dalla 1.x](#passare-dalla-1x)

## Installazione

> [!IMPORTANT]
> **Usa sempre l'[ultima release](https://github.com/domainreseller/rust-dna/releases/latest).** I pacchetti delle release sono versionati e testati.
> Il pulsante verde **Code → Download ZIP** scarica invece il branch di sviluppo.

```toml
[dependencies]
domainnameapi = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"   # per leggere gli stream di list_all
```

Finché il crate non è pubblicato su crates.io, prendilo dal tag della release:

```toml
domainnameapi = { git = "https://github.com/domainreseller/rust-dna", tag = "v2.0.0" }
```

**Requisiti:** Rust 1.83 o successivo e un runtime Tokio. Le dipendenze sono `reqwest` (con rustls, senza OpenSSL),
`serde`/`serde_json`, `tokio` (solo i timer), `futures-util`, `chrono`, `base64` e `httpdate`.

## Credenziali

L'API autentica ogni richiesta con due valori, che l'SDK invia per te:

| Valore | Header | Dove trovarlo |
|---|---|---|
| Reseller ID (un UUID) | `__reseller` | Pannello reseller → **API settings** |
| API key | `X-API-KEY` | Pannello reseller → **API settings** |

> [!NOTE]
> Queste sono **credenziali API**: il login del pannello qui non funziona. **Produzione e OTE (l'ambiente di test) usano
> chiavi API diverse.** Usare l'una con l'altro fallisce con `INVALID_API_KEY` o `INVALID_RESELLER`.

## Avvio rapido

```rust
use domainnameapi::contacts::Contact;
use domainnameapi::domains::{DomainContacts, DomainListRequest, DomainRegisterWithContactsRequest};
use domainnameapi::{Client, Environment};
use futures_util::TryStreamExt;

#[tokio::main]
async fn main() -> Result<(), domainnameapi::Error> {
    // OTE è l'ambiente di test: API reale, dati propri, nessun denaro reale.
    let client = Client::builder()
        .credentials("your-reseller-id", "your-api-key")
        .environment(Environment::Ote)
        .build()?;

    // 1. Verifica le credenziali e il saldo.
    let account = client.account().get().await?;
    println!("{}: {} USD", account.reseller_name, account.usd_balance);

    // 2. Il dominio è disponibile, e a quale prezzo?
    let search = client.domains().search("my-new-domain.com").await?;
    println!("{} {} {}", search.info.status, search.info.price, search.info.currency);

    // 3. Registralo con un solo contatto per tutti e quattro i ruoli. Questo addebita il saldo.
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

    // 4. Elenca tutti i domini; le pagine vengono recuperate man mano che lo stream viene letto.
    let mut domains = client.domains().list_all(DomainListRequest::new());
    while let Some(domain) = domains.try_next().await? {
        println!("{} expires in {} days", domain.domain_name, domain.remaining_days);
    }
    Ok(())
}
```

La cartella [examples](examples) contiene programmi eseguibili per questi scenari (`quickstart`, `register`, `dns`, `ssl`,
`custom_request`, `error_handling`). Leggono le variabili d'ambiente `DNA_*`, quindi puntale su OTE:

```bash
export DNA_RESELLER_ID=... DNA_API_KEY=... DNA_ENV=ote
cargo run --example quickstart -- example.com
```

## Configurazione

Ci sono tre modi per creare un client:

```rust
// Produzione, impostazioni predefinite
let client = Client::new(reseller_id, api_key)?;

// Tutte le impostazioni
let client = Client::builder()
    .credentials(reseller_id, api_key)
    .environment(Environment::Ote)
    .timeout(Duration::from_secs(30))
    .read_only(true)
    .build()?;

// Le stesse variabili d'ambiente lette dalla CLI dna ...
let client = Client::from_env()?;

// ... più le tue impostazioni
let client = ClientBuilder::from_env()?.http_client(my_reqwest_client).build()?;
```

| Metodo di `ClientBuilder` | Predefinito | Significato |
|---|---|---|
| `credentials(reseller_id, api_key)` | (obbligatorio) | Le credenziali. `build()` fallisce con `NO_CREDENTIALS` se una delle due è vuota. |
| `environment(...)` | `Production` | `Production` (`https://api.domainresellerapi.com`) oppure `Ote` (`https://ote.domainresellerapi.com`). |
| `base_url(...)` | nessuno | Sostituisce l'indirizzo dell'ambiente, ad esempio per un proxy. Un prefisso di percorso viene mantenuto. |
| `timeout(...)` | 60 s | Tempo massimo di un singolo tentativo. |
| `max_retries(...)` | 2 | Quante volte una richiesta fallita viene ripetuta (vedi [retry](#sicurezza-retry-e-limiti-di-richieste)). 0 disattiva i retry. |
| `max_retry_delay(...)` | 30 s | L'attesa massima prima di un retry. |
| `pace_requests(...)` | `true` | Distanzia le richieste per restare entro i limiti di richieste del gateway. |
| `read_only(...)` | `false` | Blocca, prima dell'invio, ogni richiesta che non sia una lettura. |
| `user_agent(...)` | `DomainNameAPI-Rust/<version>` | L'header `User-Agent`. |
| `http_client(...)` | quello dell'SDK | Il tuo `reqwest::Client` (proxy, impostazioni TLS). |

`from_env()` legge:

| Variabile | Significato |
|---|---|
| `DNA_RESELLER_ID`, `DNA_API_KEY` | Le credenziali. |
| `DNA_ENV` | `production` oppure `ote` (anche `prod`, `live`, `test`, `sandbox`). |
| `DNA_TEST_MODE` | `true`, `1`, `on` o `yes` seleziona OTE. Usata quando `DNA_ENV` non è impostata. |
| `DNA_BASE_URL` | Sostituzione facoltativa dell'indirizzo base. |
| `DNA_READ_ONLY` | `true` attiva la modalità di sola lettura. |

`Client` è `Clone + Send + Sync` ed economico da clonare (contiene un `Arc`). Creane uno per ogni set di credenziali e
condividilo per tutta la vita dell'applicazione: il distanziamento delle richieste è per singolo client.

## Utilizzo

Ogni area dell'API ha un proprio client, con lo stesso nome dei gruppi di comandi della CLI: `client.account()`,
`client.contacts()`, `client.domains()` (con `forwarding()`, `child_name_servers()` e `documents()`),
`client.dns()`, `client.dnssec()`, `client.ssl()`, `client.hosting()`, `client.orders()` e `client.products()`.
Gli input obbligatori sono argomenti; le operazioni con input facoltativi accettano una richiesta costruita con `new(...)` e
setter concatenati. I metodi che addebitano il tuo saldo lo indicano nella loro documentazione.

### Account

```rust
let account = client.account().get().await?;          // reseller_id, reseller_name, try_balance, usd_balance
let profile = client.account().profile().await?;      // name, company_name, account_currency, status, ...

// Gli addebiti di questo mese, su tutte le pagine
let request = AccountTransactionsRequest::new()
    .charge_type(ChargeType::Subtract)
    .start_date("2026-11-01T00:00:00Z".parse()?)
    .transaction_types([TransactionType::Register, TransactionType::Renew]);
let mut transactions = client.account().all_transactions(request);
while let Some(t) = transactions.try_next().await? {
    println!("{:?} {} {} {}", t.creation_time, t.transaction_type, t.amount, t.amount_currency);
}
```

### Domini

**Ricerca**

```rust
let one = client.domains().search("example.com").await?;
let available = one.success && one.info.is_available();

let many = client.domains().bulk_search(["example.com", "example.net", "example.com.tr"]).await?;
for domain in many.available() {
    println!("{}: {} {}", domain.domain_name, domain.price, domain.currency);
}

// Con una valuta
client.domains().search_with(DomainSearchRequest::new("example.com").currency("USD")).await?;
```

**Elenco e dettagli**

```rust
let page = client.domains().list(DomainListRequest::new().limit(50).statuses([DomainStatus::Active])).await?;

let all: Vec<Domain> = client.domains().list_all(DomainListRequest::new().domain_name("example")).try_collect().await?;

let info = client.domains().info("example.com").await?;
println!("{} {:?}", info.auth_code, info.expiration_date);
if let Some(registrant) = info.contact(ContactType::Registrant) {
    println!("{}", registrant.email);
}
```

**Registrazione, rinnovo, trasferimento, ripristino** (addebitano il tuo saldo)

```rust
// Un solo contatto per tutti e quattro i ruoli, due anni
let request = DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact.clone()), ["tr.apiname.com", "eu.apiname.com"])
    .years(2);
client.domains().register_with_contacts(request).await?;

// Contatti diversi per ruolo, attributi del registry, il servizio trustee
let request = DomainRegisterWithContactsRequest::new(
    "example.com.tr",
    DomainContacts { registrant: Some(owner), admin: Some(staff.clone()), tech: Some(staff.clone()), billing: Some(staff) },
    ["tr.apiname.com", "eu.apiname.com"],
)
.tld_attribute("<key>", "<value>"); // le chiavi: client.products().tld("com.tr").await?.unwrap().attributes

// Con handle di contatto esistenti
let request = DomainRegisterRequest::new("example.com", ContactHandles::same("DNA-12345"), ["tr.apiname.com", "eu.apiname.com"]);
client.domains().register(request).await?;

let renewal = client.domains().renew_check("example.com", Some(1)).await?;   // gratuito
if renewal.renewable {
    client.domains().renew("example.com", 1).await?;
}

let check = client.domains().transfer_check("example.com", "EPP-CODE").await?;  // gratuito
if check.transferable {
    client.domains().transfer(DomainTransferRequest::new("example.com", "EPP-CODE")).await?;
}

client.domains().restore("example.com").await?;
```

Le richieste vengono controllate prima dell'invio. Una registrazione senza tutti e quattro i ruoli di contatto, un contatto
senza indirizzo e-mail o un periodo di 0 anni restituiscono un errore `ErrorKind::InvalidInput` che elenca ogni problema, e nulla
raggiunge l'API.

> [!NOTE]
> L'API può rispondere a una registrazione, a un rinnovo o a un trasferimento con HTTP 200 e `"success": false` (ad esempio quando il
> registry rifiuta il dominio). L'SDK restituisce quella risposta così com'è, come l'SDK Java: controlla
> `DomainOperationResult::success` e `message`. Lo stesso vale per `SslOperationResult` e `ContactCreateResult`.

**Impostazioni**

```rust
client.domains().lock("example.com").await?;
client.domains().unlock("example.com").await?;
client.domains().set_privacy("example.com", true).await?;
client.domains()
    .set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net", "ns2.example.net"]))
    .await?;
client.domains()   // un dominio firmato con DNSSEC
    .set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net"]).dnssec_mode(DnssecMode::DisableDnssec))
    .await?;
let auth_code = client.domains().update_auth_code("example.com").await?;

client.domains().update_contacts("example.com", DomainContacts::same(contact)).await?;
client.domains().set_contact_handles("example.com", ContactHandles { tech: "DNA-12345".into(), ..Default::default() }).await?;
```

**Inoltro URL, nameserver figli e documenti**

```rust
client.domains().forwarding().set("example.com", "https://www.example.net", ForwardType::Standard).await?;
let forwarding = client.domains().forwarding().get("example.com").await?;   // None se non impostato
client.domains().forwarding().delete("example.com").await?;

// La versione IP di ogni indirizzo viene rilevata
client.domains().child_name_servers().add("example.com", "ns1.example.com", ["192.0.2.10", "2001:db8::10"]).await?;
client.domains().child_name_servers()
    .update(DomainHostUpdateRequest::new("example.com", "ns1.example.com", ["192.0.2.20"]))
    .await?;
client.domains().child_name_servers().delete("example.com", "ns1.example.com").await?;

// Documenti del registry (soprattutto .tr)
let document = DocumentUpload::from_file(DocumentType::IdentityCard, "id.pdf")?;
client.domains().documents().upload("example.com.tr", [document]).await?;
let documents: Vec<Document> = client.domains().documents().list_all("example.com.tr").try_collect().await?;
```

### Contatti

Gli handle di contatto sono contatti riutilizzabili. Creane uno una volta e usalo per molti domini.

```rust
let created = client.contacts().create(contact.clone()).await?;
let handle = created.handle;                                         // ad esempio DNA-12345

let details = client.contacts().get(&handle).await?;                 // il contatto e il suo stato di verifica
let result = client.contacts().update(&handle, contact).await?;
println!("{} of {} domains updated", result.stats.success_count, result.stats.total_domain_count);

let verifications = client.contacts().verification(&handle).await?;
client.contacts().resend_verification(&handle).await?;
```

### DNS

Record di zona dei domini sui nameserver della piattaforma. I record si risolvono pubblicamente quando il dominio usa quei nameserver,
cosa di cui si occupa `apply`.

```rust
for set in client.dns().list("example.com").await? {
    println!("{} {} {} {:?}", set.name, set.record_type, set.ttl, set.values());
}

let dns = client.dns();
dns.add(DnsAddRequest::new("example.com", "www", RecordType::A, ["192.0.2.10"])).await?;
dns.add(DnsAddRequest::new("example.com", "@", "txt", ["v=spf1 include:_spf.example.net ~all"])).await?;   // "txt" diventa TXT
dns.add(DnsAddRequest::new("example.com", "@", RecordType::Mx, ["10 mx1.example.net", "20 mx2.example.net"]).ttl(3600)).await?;
dns.update(DnsUpdateRequest::new("example.com", "www", RecordType::A, ["192.0.2.20"]).ttl(300)).await?;
dns.delete(DnsDeleteRequest::new("example.com", "www", RecordType::A, "192.0.2.20")).await?;

let applied = dns.apply("example.com").await?;
```

L'SDK controlla ciò che l'API rifiuterebbe: un CNAME all'apex, un MX senza la sua priorità, un valore CAA senza le sue
tre parti, un record A che non è IPv4, un TTL fuori dall'intervallo 1–86400. Le chiamate alla zona sono distanziate di 1 secondo, il
limite del gateway. L'ambiente di test OTE risponde alle chiamate alla zona con HTTP 500, quindi il codice DNS si può provare solo in produzione.

### DNSSEC

```rust
let info = client.dnssec().get("example.com").await?;      // status, ds_records, allowed_algorithms, ...

let record: DsData = "12345 13 2 49FD46E6C4B45C55D4AC69CBD3CD34AC1AFE51DE".parse()?;
client.dnssec().add("example.com", [record.clone()]).await?;
client.dnssec().remove("example.com", record).await?;
client.dnssec().disable("example.com").await?;            // rimuove tutti i record DS
```

### SSL

Un dominio può avere più certificati, quindi un certificato esistente si indica con il suo **ID del certificato** (un GUID),
mai con il nome di dominio. `initialize` e `renew` restituiscono l'ID: conservalo. `list` trova gli ID dei certificati
che hai già.

```rust
// 1. Acquisto (addebita il saldo). I codici dei pacchetti provengono da client.products().ssls(...).
let approvers = client.ssl().approvers(SslApproversRequest::for_purchase("example.com", "PKG-CODE")).await?;
let order = client.ssl().initialize("example.com", "PKG-CODE").await?;
let certificate_id = order.certificate_id;                  // conservalo: ogni chiamata successiva lo richiede

// 2. Una CSR e una chiave privata, se non ne hai una
let csr = client.ssl().generate_csr(
    SslGenerateCsrRequest::new("example.com")
        .country("TR").state("Istanbul").locality("Istanbul").organization("Example Ltd").organization_unit("IT"),
).await?;

// 3. Configura entro 15 giorni: CSR, metodo di validazione e contatto
client.ssl().configure(
    SslConfigureRequest::new(&certificate_id, &csr.csr, certificate_contact.clone())
        .authentication_type(AuthenticationType::Email)
        .approver_mail(&approvers[0])
        .web_server_type(WebServerType::Apache),
).await?;

// 4. Attendi l'emissione, poi scarica
let status = client.ssl().status(&certificate_id).await?;
if status.is_issued {
    let content = client.ssl().content(&certificate_id).await?;
    std::fs::write("fullchain.pem", content.full_chain())?;
}

let details = client.ssl().get(&certificate_id).await?;     // abbonamento, configurazione, contatti
client.ssl().reissue(SslReissueRequest::new(&certificate_id).csr(new_csr).private_key(new_key)).await?;
client.ssl().revoke(&certificate_id).await?;                // non può essere annullato
```

**Elencare i certificati e trovare un ID**

```rust
let page = client.ssl().list(SslListRequest::new().domain_name("example.com")).await?;   // prima i più recenti

let mut all = client.ssl().list_all(SslListRequest::new());
while let Some(c) = all.try_next().await? {
    println!("{} {} {} {:?}", c.certificate_id, c.domain_name, c.status, c.subscription_expires_at);
}

// Gli indirizzi di approvazione di un certificato che hai già
let mails = client.ssl().approvers(SslApproversRequest::for_certificate(&certificate_id)).await?;
```

**Rinnovo** (addebita il tuo saldo)

```rust
// Consentito a partire da 30 giorni prima di details.subscription.expires_at. Il rinnovo è un nuovo certificato con un proprio ID.
let renewal = client.ssl().renew(&certificate_id).await?;
client.ssl().configure(
    SslConfigureRequest::new(&renewal.certificate_id, &new_csr, certificate_contact)
        .authentication_type(AuthenticationType::Email)
        .approver_mail(&approvers[0]),
).await?;
```

Un certificato acquistato o rinnovato va configurato entro 15 giorni; dopo viene annullato. Per rinnovare, chiama
`renew` e poi `configure` con il nuovo ID. Non acquistare di nuovo con `initialize`: sarebbe un acquisto separato al
prezzo di acquisto e il vecchio certificato non verrebbe mai contrassegnato come rinnovato. Un ID del certificato vuoto o
che non è un GUID (ad esempio un nome di dominio passato da codice più vecchio) viene rifiutato prima che venga inviato qualcosa.

### Hosting

```rust
let packages = client.hosting().packages().await?;
let created = client.hosting().create("example.com", "starter-linux").await?;          // addebita il saldo
let status = client.hosting().status(HostingStatusRequest::by_service_id(&created.service_id)).await?;
if status.is_active() {
    println!("{} {}", status.panel_address, status.username);
}
let hosting = client.hosting().get(HostingGetRequest::by_domain("example.com")).await?;
let plans = client.hosting().plans(None).await?;
```

### Ordini e prodotti

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

## Richieste e il trait `Request`

Ogni operazione è un tipo di richiesta che prende il nome dal suo comando CLI: `dna domain renew` è `DomainRenewRequest`,
`dna domain renew-check` è `DomainRenewCheckRequest`, `dna dns add` è `DnsAddRequest`. I metodi delle aree le costruiscono
per te, ma puoi anche crearle e inviarle tu stesso:

```rust
let request = DomainRenewRequest::new("example.com", 2);
let endpoint = request.endpoint();
println!("{endpoint} is {}", endpoint.safety);   // "POST /api/v1/domains/renew is billable"

let result = client.send(&request).await?;

// Lo stesso, con i dettagli HTTP: correlation ID, stato della soft quota, tentativi, durata
let response = client.send_with_response(&request).await?;
println!("{} quota {:?}", response.correlation_id, response.quota);
```

Rust non ha l'ereditarietà; la base di ogni richiesta è un trait:

| Trait | Per |
|---|---|
| `Request` | Ogni operazione. `type Response` è il tipo della risposta; `Empty` per le operazioni che non restituiscono nulla (blocco, sblocco, aggiunta di un record DNS, ...). |
| `PagedRequest<T>` | Operazioni di elenco (`Response = Page<T>`). Dà accesso al `Paging` (skip, limit, sort) e funziona con `Client::list_all`. |

Una richiesta dichiara cosa è con `endpoint()`: metodo, percorso, classe di sicurezza e se è idempotente. Gli altri tre
metodi hanno implementazioni predefinite: `query()` restituisce la query string (`Query` omette i valori vuoti), `body()` il corpo JSON
(`JsonBody` ne costruisce uno e omette i valori vuoti, oppure usa `serde_json::json!`) e `validate()` raccoglie i problemi
prima dell'invio. Per chiamare un endpoint che l'SDK non copre ancora, scrivi la tua richiesta:

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
        Endpoint::get("/api/v1/domains/notes")          // GET è una lettura
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

La tua richiesta ottiene tutto ciò che ottengono quelle integrate: credenziali, retry per classe di sicurezza, distanziamento, modalità di sola lettura e
gestione degli errori. Per una modifica, usa `Endpoint::post(path, OperationSafety::Write)` (oppure `Billable`, `Destructive`) e
aggiungi `.idempotent()` solo quando inviarla due volte non ha conseguenze.

## Errori

Ogni richiesta fallita restituisce un `Error`, anche quando la richiesta non ha superato i controlli e non è stato inviato nulla.

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
        // Una richiesta a pagamento o che modifica qualcosa è fallita in rete: potrebbe aver raggiunto l'API.
        // Controlla lo stato prima di inviarla di nuovo.
        let info = client.domains().info("example.com").await?;
    }
    Err(e) => return Err(e.into()),
}
```

| Metodo | Significato |
|---|---|
| `kind()` | Una classificazione stabile (tabella sotto). Ramifica su questa, non sul messaggio. |
| `code()` | Il codice di errore dell'API (come `Dna.DomainService:Domain:10007`) o uno dei codici dell'SDK qui sotto. Quelli più comuni sono costanti in `domainnameapi::codes`. |
| `message()` | Cosa è successo. I messaggi generici vengono sostituiti con i dettagli dell'API. |
| `status()` | Lo stato HTTP, oppure `None` se non è arrivata alcuna risposta. |
| `details()`, `validation_errors()` | Dettagli aggiuntivi e problemi a livello di campo (dall'API o dai controlli prima dell'invio). |
| `correlation_id()` | L'`X-Correlation-Id` del gateway. Citalo quando contatti il supporto. |
| `hint()` | Un breve suggerimento su cosa fare dopo. |
| `is_retryable()` | Indica se ripetere la stessa richiesta più tardi può riuscire. |
| `is_outcome_unknown()` | La richiesta potrebbe essere stata eseguita: una modifica è fallita in rete, oppure l'API ha indicato che il registry potrebbe averla già eseguita. |
| `is_not_found()`, `is_insufficient_balance()` | Scorciatoie per i casi comuni. |
| `operation()`, `safety()` | La richiesta fallita (`POST /api/v1/domains/renew`) e la sua classe di sicurezza. |
| `retry_after()`, `response_body()` | Il `Retry-After` del gateway e la risposta grezza. |

`Error` implementa `std::error::Error` e `Display` (`[CODE] message (HTTP 400); field: problem`), quindi funziona
con `?`, `anyhow` e `thiserror`.

| `kind()` | Quando |
|---|---|
| `Auth` | Credenziali errate o mancanti, oppure reseller sconosciuto o sospeso in questo ambiente (401, redirect). |
| `Forbidden` | Autenticato ma non autorizzato (403). |
| `NotFound` | Il dominio, il contatto, il certificato, l'ordine o un'altra risorsa non esiste (404). |
| `Validation` | L'API ha rifiutato l'input o una regola di business è fallita: campo errato, dominio bloccato, saldo insufficiente (400, 422). |
| `Conflict` | Lo stato della risorsa non consente l'operazione (409). |
| `RateLimited` | Limite di richieste ancora superato dopo i retry automatici (429). |
| `Server` | Un errore lato server (5xx). Spesso temporaneo. |
| `Network`, `Timeout` | Nessuna risposta: DNS, TLS, proxy, connessione interrotta, o nessuna risposta entro il timeout. |
| `InvalidInput` | La richiesta non ha superato i controlli, oppure un'impostazione del client non è valida; non è stato inviato nulla. |
| `ReadOnly` | Bloccata dalla modalità di sola lettura; non è stato inviato nulla. |
| `Unknown` | Qualsiasi altro caso, inclusa una risposta che l'SDK non ha saputo leggere. |

`ErrorKind::as_str()` restituisce i nomi usati dalla CLI (`auth`, `not_found`, `rate_limited`, ...). Codici impostati dall'SDK stesso:

| Codice | Significato |
|---|---|
| `INVALID_RESELLER` | Il gateway ha reindirizzato alla sua pagina di sospensione: il reseller ID è sconosciuto, inattivo o sospeso **in questo ambiente**. |
| `INVALID_API_KEY` | La chiave non corrisponde al reseller. Le chiavi di produzione e OTE sono diverse. |
| `MISSING_API_KEY`, `MISSING_RESELLER_ID`, `NO_CREDENTIALS` | Una credenziale è vuota. |
| `SUBRESELLER_NOT_ALLOWED` | Gli account sub-reseller non possono usare l'API. Usa le credenziali del reseller principale. |
| `UNAUTHORIZED` | Qualsiasi altro 401. |
| `RATE_LIMITED` | 429 dopo i retry. |
| `NETWORK`, `TIMEOUT` | Non è arrivata alcuna risposta. |
| `READ_ONLY` | Bloccata dalla modalità di sola lettura. |
| `INVALID_INPUT`, `INVALID_CONFIGURATION` | Una richiesta non ha superato i controlli, oppure un'impostazione del client non è valida. |
| `EMPTY_RESPONSE`, `INVALID_RESPONSE` | Una risposta riuscita era vuota o non è stato possibile leggerla. |
| `HTTP_<status>` | L'API non ha inviato alcun codice di errore. |

## Sicurezza, retry e limiti di richieste

Ogni richiesta ha una classe di sicurezza, la stessa usata dalla CLI `dna`:

| `OperationSafety` | Significato | Esempi |
|---|---|---|
| `Read` | Nessun effetto collaterale, anche quando l'API usa POST o PUT | ricerca, dettagli, elenchi, verifiche di rinnovo e trasferimento, generazione della CSR, stato del certificato |
| `Write` | Una modifica gratuita e reversibile | record DNS, blocco, privacy, nameserver, contatti |
| `Billable` | Addebita il tuo saldo | registrazione, rinnovo, trasferimento, ripristino, acquisto e rinnovo SSL, ordine di hosting |
| `Destructive` | Elimina qualcosa o è difficile da annullare | eliminazione di record DNS, rimozione DNSSEC, revoca del certificato |

- **Modalità di sola lettura** (`read_only(true)` o `DNA_READ_ONLY=true`) blocca, prima dell'invio, tutto ciò che non è `Read`.
  Usala mentre esplori, nei report e nelle dashboard.
- **Retry.** Le risposte 429 vengono sempre ritentate, rispettando `Retry-After`. È sicuro anche per le richieste a pagamento,
  perché il gateway le rifiuta prima che venga eseguito qualsiasi cosa. Errori di rete, timeout e risposte 502/503/504 vengono ritentati
  solo per le letture e per le scritture che la CLI contrassegna come idempotenti (blocco, sblocco, privacy, nameserver, aggiornamento DNS e
  apply). Una registrazione o un rinnovo non viene mai inviato due volte.
- **Distanziamento.** Il gateway consente 5 richieste al secondo per chiave e 2 al secondo per le ricerche di domini. Il client
  distanzia le richieste per restare entro questi limiti e mantiene le chiamate alla zona DNS a 1 secondo di distanza. Le informazioni sul dominio e l'elenco dei TLD
  non hanno limiti di richieste.
- **Soft quota.** Le ricerche e le consultazioni di informazioni costano 1 punto, tutto il resto 10. Oltre i 200 punti il gateway rallenta ogni
  richiesta fino a 5 secondi; i punti decadono del 20% al minuto. `send_with_response` mostra lo stato in `quota`.

## Codice asincrono e bloccante

L'SDK è asincrono e richiede un runtime Tokio (quello avviato da `#[tokio::main]`, o quello del tuo server). I future sono
`Send`, quindi funzionano in `tokio::spawn` e negli handler di axum e actix. Gli elenchi si possono leggere pagina per pagina (`list`) o come
`Stream` di elementi (`list_all`), che recupera la pagina successiva solo quando serve:

```rust
use futures_util::TryStreamExt;

let expiring: Vec<Domain> = client
    .domains()
    .list_all(DomainListRequest::new())
    .try_filter(|d| std::future::ready(d.remaining_days < 30))
    .try_collect()
    .await?;
```

Non esiste un'API bloccante: raddoppierebbe la superficie dell'API con poco vantaggio. Nel codice sincrono, esegui le chiamate su un
runtime:

```rust
let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
let account = runtime.block_on(client.account().get())?;
```

## Testare il tuo codice

All'SDK serve solo un endpoint HTTP. Per testare il codice che lo usa senza chiamare l'API, punta `base_url` su un server
locale (ad esempio uno costruito con `wiremock`, `httpmock` o `axum`) e disattiva il distanziamento:

```rust
let client = Client::builder()
    .credentials("test", "test")
    .base_url(format!("http://{}", server_address))
    .pace_requests(false)
    .build()?;
```

I modelli implementano `Default` e `Deserialize`, quindi i dati di test possono provenire da JSON; gli errori per i tuoi mock si possono costruire
con `Error::new(kind, code, message)`. Per i test sull'API reale, usa OTE con la sua chiave.

## Struttura del progetto

```
src/
├── lib.rs           Documentazione del crate e re-export
├── client.rs        Client, ClientBuilder, Environment: invio, retry, distanziamento, parsing degli errori, list_all
├── request.rs       Request, PagedRequest, Endpoint, OperationSafety, Query, JsonBody, Validator, Page, Empty
├── error.rs         Error, ErrorKind, codes
├── de.rs            Decodifica tollerante e macro che dichiarano modelli ed enum (interno)
├── account.rs       AccountClient     Saldi, profilo e transazioni
├── contacts.rs      ContactsClient    Handle di contatto
├── domains/         DomainsClient     Domini, più forwarding(), child_name_servers() e documents()
├── dns.rs           DnsClient         Record di zona
├── dnssec.rs        DnssecClient      Record DS
├── ssl.rs           SslClient         Certificati
├── hosting.rs       HostingClient     Pacchetti e servizi di hosting
├── orders.rs        OrdersClient      Ordini
└── products.rs      ProductsClient    TLD, prodotti SSL e prezzi
tests/               Test su API finta, il test di copertura della specifica e testdata/openapi.json
examples/            Programmi eseguibili
```

Ogni modulo di area ha la stessa forma: i modelli e gli enum delle sue risposte, un tipo di richiesta per operazione
(`XxxRequest`) e un `XxxClient`.

## Sviluppo

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                    # test unitari, su API finta, della specifica e della documentazione; nessuno chiama l'API reale
cargo build --examples
```

Senza una toolchain locale, usa Docker:

```bash
docker run --rm -v "$PWD":/src -w /src rust:1.83 sh -c 'rustup component add rustfmt clippy; cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo test'
```

I test non chiamano mai l'API reale: usano un piccolo server HTTP su 127.0.0.1 (`tests/common`). La CI li esegue sulla
versione minima di Rust supportata (1.83) e sull'ultima stabile.

`tests/testdata/openapi.json` è il documento API che la CLI `dna` incorpora. `tests/spec.rs` verifica che ogni operazione
al suo interno abbia esattamente un tipo di richiesta con il metodo, il percorso, la classe di sicurezza e l'idempotenza della CLI, che ogni richiesta
invii solo parametri di query e proprietà del corpo descritti nel documento (oggetti annidati inclusi, con quelli obbligatori
presenti) e che i tipi di risposta usino i nomi del documento. Quando l'API aggiunge un'operazione, aggiorna il documento e il
test fallisce finché il tipo di richiesta non esiste.

## Passare dalla 1.x

La 2.0 è un nuovo SDK. La 1.x era un port della libreria PHP: un client bloccante `DomainNameApi` che restituiva envelope
`serde_json::Value` non tipizzati. La 2.0 è asincrona, con richieste e modelli tipizzati, `Result<T, Error>` e un builder.

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
| `result["result"] == "ERROR"` | `Err(e)` con `e.kind()` e `e.code()` |

Le chiamate di approvazione, annullamento e rifiuto del trasferimento e `sync_from_registry` della 1.x non fanno parte del documento
dell'API REST e non hanno un equivalente nella 2.0. La MSRV resta 1.83; `ureq` è sostituito da `reqwest`, quindi la 2.0 richiede un runtime Tokio.
La 1.x inviava report di errore a Sentry; la 2.0 non invia nulla a nessuno, tranne che all'API.

## Link

- [CLI `dna` e server MCP](https://github.com/domainreseller/cli): la stessa API dal terminale e dagli agenti AI.
- [Documentazione dell'API](https://www.domainnameapi.com/docs/)
- [Changelog](CHANGELOG.md)

## Licenza

Apache License 2.0. Vedi [LICENSE](LICENSE).
