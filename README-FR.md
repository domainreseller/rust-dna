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

Le SDK Rust officiel de la plateforme revendeur [Domain Name API](https://www.domainnameapi.com). Il couvre les
67 opérations de l'API REST avec des requêtes et des modèles typés : domaines, contacts, DNS, DNSSEC, certificats SSL,
hébergement, commandes, produits et votre compte.

```rust
let client = Client::builder().credentials(reseller_id, api_key).environment(Environment::Ote).build()?;

let search = client.domains().search("example.com").await?;
if search.info.is_available() {
    let request = DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact), ["tr.apiname.com", "eu.apiname.com"]);
    client.domains().register_with_contacts(request).await?;
}
```

- **Tout est typé.** Chaque opération est un type de requête et chaque réponse une struct documentée. Pas de maps, pas de parsing JSON.
- **Vos propres requêtes.** Chaque requête implémente le trait `Request`. Implémentez-le pour un endpoint que le SDK ne couvre pas encore.
- **Sûr avec l'argent.** Chaque requête déclare si elle lit, écrit, débite votre solde ou supprime quelque chose. Les requêtes facturables ne sont jamais rejouées, et un mode lecture seule bloque toute modification.
- **Résilience intégrée.** Les limites de débit sont respectées automatiquement, les réponses 429 sont rejouées, et les erreurs réseau sont rejouées pour les lectures.
- **Des erreurs claires.** Un seul type `Error` avec une nature stable, le code d'erreur de l'API, les erreurs de validation par champ, l'ID de corrélation et une piste de résolution.
- **Rust moderne.** Async sur Tokio, `reqwest` avec rustls (sans OpenSSL), modèles serde, pagination paresseuse sous forme de `Stream`, un client clonable `Send + Sync`, `#![deny(missing_docs)]`.
- **Confidentiel.** Aucune télémétrie. Rien ne quitte votre processus, à part les requêtes que vous envoyez à l'API.

C'est la version Rust de [`dna`, la CLI Domain Name API](https://github.com/domainreseller/cli) : le même document
d'API, les mêmes opérations et groupes, les mêmes classes de sécurité, règles de nouvelle tentative et codes d'erreur.

## Sommaire

- [Installation](#installation)
- [Identifiants](#identifiants)
- [Démarrage rapide](#démarrage-rapide)
- [Configuration](#configuration)
- [Utilisation](#utilisation) : [compte](#compte) · [domaines](#domaines) · [contacts](#contacts) · [DNS](#dns) · [DNSSEC](#dnssec) · [SSL](#ssl) · [hébergement](#hébergement) · [commandes et produits](#commandes-et-produits)
- [Requêtes et le trait `Request`](#requêtes-et-le-trait-request)
- [Erreurs](#erreurs)
- [Sécurité, nouvelles tentatives et limites de débit](#sécurité-nouvelles-tentatives-et-limites-de-débit)
- [Code asynchrone et code bloquant](#code-asynchrone-et-code-bloquant)
- [Tester votre code](#tester-votre-code)
- [Structure du projet](#structure-du-projet)
- [Développement](#développement)
- [Migrer depuis la 1.x](#migrer-depuis-la-1x)

## Installation

> [!IMPORTANT]
> **Utilisez toujours la [dernière version publiée](https://github.com/domainreseller/rust-dna/releases/latest).** Les paquets publiés sont versionnés et testés.
> Le bouton vert **Code → Download ZIP** télécharge à la place la branche de développement.

```toml
[dependencies]
domainnameapi = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"   # pour lire les streams list_all
```

Tant que la crate n'est pas publiée sur crates.io, récupérez-la depuis le tag de la version publiée :

```toml
domainnameapi = { git = "https://github.com/domainreseller/rust-dna", tag = "v2.1.0" }
```

**Prérequis :** Rust 1.83 ou version ultérieure et un runtime Tokio. Les dépendances sont `reqwest` (avec rustls, sans OpenSSL),
`serde`/`serde_json`, `tokio` (uniquement les timers), `futures-util`, `chrono`, `base64` et `httpdate`.

## Identifiants

L'API authentifie chaque requête avec deux valeurs, que le SDK envoie pour vous :

| Valeur | En-tête | Où la trouver |
|---|---|---|
| Reseller ID (un UUID) | `__reseller` | Panneau revendeur → **API settings** |
| Clé API | `X-API-KEY` | Panneau revendeur → **API settings** |

> [!NOTE]
> Il s'agit d'**identifiants API** : vos identifiants de connexion au panneau ne fonctionnent pas ici. **La production et l'OTE (l'environnement de test)
> utilisent des clés API différentes.** Utiliser l'une à la place de l'autre échoue avec `INVALID_API_KEY` ou `INVALID_RESELLER`.

## Démarrage rapide

```rust
use domainnameapi::contacts::Contact;
use domainnameapi::domains::{DomainContacts, DomainListRequest, DomainRegisterWithContactsRequest};
use domainnameapi::{Client, Environment};
use futures_util::TryStreamExt;

#[tokio::main]
async fn main() -> Result<(), domainnameapi::Error> {
    // L'OTE est l'environnement de test : la vraie API, ses propres données, pas d'argent réel.
    let client = Client::builder()
        .credentials("your-reseller-id", "your-api-key")
        .environment(Environment::Ote)
        .build()?;

    // 1. Vérifier les identifiants et le solde.
    let account = client.account().get().await?;
    println!("{}: {} USD", account.reseller_name, account.usd_balance);

    // 2. Le domaine est-il disponible, et à quel prix ?
    let search = client.domains().search("my-new-domain.com").await?;
    println!("{} {} {}", search.info.status, search.info.price, search.info.currency);

    // 3. L'enregistrer avec un seul contact pour les quatre rôles. Cela débite le solde.
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

    // 4. Lister tous les domaines ; les pages sont récupérées au fil de la lecture du stream.
    let mut domains = client.domains().list_all(DomainListRequest::new());
    while let Some(domain) = domains.try_next().await? {
        println!("{} expires in {} days", domain.domain_name, domain.remaining_days);
    }
    Ok(())
}
```

Le dossier [examples](examples) contient des programmes exécutables pour ces scénarios (`quickstart`, `register`, `dns`, `ssl`,
`custom_request`, `error_handling`). Ils lisent les variables d'environnement `DNA_*` ; faites-les donc pointer vers l'OTE :

```bash
export DNA_RESELLER_ID=... DNA_API_KEY=... DNA_ENV=ote
cargo run --example quickstart -- example.com
```

## Configuration

Il y a trois façons de créer un client :

```rust
// Production, paramètres par défaut
let client = Client::new(reseller_id, api_key)?;

// Tous les paramètres
let client = Client::builder()
    .credentials(reseller_id, api_key)
    .environment(Environment::Ote)
    .timeout(Duration::from_secs(30))
    .read_only(true)
    .build()?;

// Les mêmes variables d'environnement que celles lues par la CLI dna ...
let client = Client::from_env()?;

// ... plus vos propres paramètres
let client = ClientBuilder::from_env()?.http_client(my_reqwest_client).build()?;
```

| Méthode de `ClientBuilder` | Par défaut | Signification |
|---|---|---|
| `credentials(reseller_id, api_key)` | (obligatoire) | Les identifiants. `build()` échoue avec `NO_CREDENTIALS` si l'un d'eux est vide. |
| `environment(...)` | `Production` | `Production` (`https://api.domainresellerapi.com`) ou `Ote` (`https://ote.domainresellerapi.com`). |
| `base_url(...)` | aucune | Remplace l'adresse de l'environnement, par exemple pour un proxy. Un préfixe de chemin est conservé. |
| `timeout(...)` | 60 s | Délai maximal d'une tentative. |
| `max_retries(...)` | 2 | Combien de fois une requête échouée est rejouée (voir [nouvelles tentatives](#sécurité-nouvelles-tentatives-et-limites-de-débit)). 0 désactive les nouvelles tentatives. |
| `max_retry_delay(...)` | 30 s | L'attente la plus longue avant une nouvelle tentative. |
| `pace_requests(...)` | `true` | Espace les requêtes pour rester sous les limites de débit de la passerelle. |
| `read_only(...)` | `false` | Bloque, avant envoi, toute requête qui n'est pas une lecture. |
| `user_agent(...)` | `DomainNameAPI-Rust/<version>` | L'en-tête `User-Agent`. |
| `app(name, version)` | `rust`, la version du SDK | Les en-têtes `X-App` et `X-Version` : l'application qui utilise le SDK (voir plus bas). |
| `http_client(...)` | celui du SDK | Votre propre `reqwest::Client` (proxy, paramètres TLS). |

`from_env()` lit :

| Variable | Signification |
|---|---|
| `DNA_RESELLER_ID`, `DNA_API_KEY` | Les identifiants. |
| `DNA_ENV` | `production` ou `ote` (aussi `prod`, `live`, `test`, `sandbox`). |
| `DNA_TEST_MODE` | `true`, `1`, `on` ou `yes` sélectionne l'OTE. Utilisée quand `DNA_ENV` n'est pas définie. |
| `DNA_BASE_URL` | Remplacement facultatif de l'adresse de base. |
| `DNA_READ_ONLY` | `true` active le mode lecture seule. |
| `DNA_APP`, `DNA_APP_VERSION` | L'application qui utilise le SDK et sa version, envoyées dans `X-App` et `X-Version`. |

`Client` est `Clone + Send + Sync` et peu coûteux à cloner (il contient un `Arc`). Créez-en un par jeu d'identifiants et
partagez-le pendant toute la durée de vie de l'application : l'espacement des requêtes est propre à chaque client.

### Identifier votre application

Chaque requête porte les en-têtes `X-App` et `X-Version`, qui indiquent à l'équipe de l'API quelle intégration l'a
envoyée. Par défaut, ils valent `rust` et la version du SDK. Si vous construisez un module de panel ou un autre produit
sur le SDK, indiquez vos propres valeurs :

```rust
let client = Client::builder()
    .credentials(reseller_id, api_key)
    .app("WiseCP5", "V2.0.20-beta") // envoyé comme X-App: wisecp5, X-Version: 2.0.20
    .build()?;
```

Le nom est mis en minuscules et ne garde que `a-z`, `0-9`, `.`, `_` et `-` ; la version ne garde que les chiffres et les
points. Un nom vide revient à `rust`, une version vide à la version du SDK. `from_env()` lit les mêmes valeurs dans
`DNA_APP` et `DNA_APP_VERSION`. L'en-tête `User-Agent` ne change pas.

## Utilisation

Chaque domaine fonctionnel de l'API a son propre client, nommé comme les groupes de commandes de la CLI : `client.account()`,
`client.contacts()`, `client.domains()` (avec `forwarding()`, `child_name_servers()` et `documents()`),
`client.dns()`, `client.dnssec()`, `client.ssl()`, `client.hosting()`, `client.orders()` et `client.products()`.
Les entrées obligatoires sont des arguments ; les opérations avec des entrées facultatives prennent une requête construite avec `new(...)` et des
setters chaînés. Les méthodes qui débitent votre solde l'indiquent dans leur documentation.

### Compte

```rust
let account = client.account().get().await?;          // reseller_id, reseller_name, try_balance, usd_balance
let profile = client.account().profile().await?;      // name, company_name, account_currency, status, ...

// Les débits de ce mois, sur toutes les pages
let request = AccountTransactionsRequest::new()
    .charge_type(ChargeType::Subtract)
    .start_date("2026-11-01T00:00:00Z".parse()?)
    .transaction_types([TransactionType::Register, TransactionType::Renew]);
let mut transactions = client.account().all_transactions(request);
while let Some(t) = transactions.try_next().await? {
    println!("{:?} {} {} {}", t.creation_time, t.transaction_type, t.amount, t.amount_currency);
}
```

### Domaines

**Recherche**

```rust
let one = client.domains().search("example.com").await?;
let available = one.success && one.info.is_available();

let many = client.domains().bulk_search(["example.com", "example.net", "example.com.tr"]).await?;
for domain in many.available() {
    println!("{}: {} {}", domain.domain_name, domain.price, domain.currency);
}

// Avec une devise
client.domains().search_with(DomainSearchRequest::new("example.com").currency("USD")).await?;
```

**Liste et détails**

```rust
let page = client.domains().list(DomainListRequest::new().limit(50).statuses([DomainStatus::Active])).await?;

let all: Vec<Domain> = client.domains().list_all(DomainListRequest::new().domain_name("example")).try_collect().await?;

let info = client.domains().info("example.com").await?;
println!("{} {:?}", info.auth_code, info.expiration_date);
if let Some(registrant) = info.contact(ContactType::Registrant) {
    println!("{}", registrant.email);
}
```

**Enregistrer, renouveler, transférer, restaurer** (ces opérations débitent votre solde)

```rust
// Un seul contact pour les quatre rôles, deux ans
let request = DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact.clone()), ["tr.apiname.com", "eu.apiname.com"])
    .years(2);
client.domains().register_with_contacts(request).await?;

// Des contacts différents par rôle, des attributs de registre, le service trustee
let request = DomainRegisterWithContactsRequest::new(
    "example.com.tr",
    DomainContacts { registrant: Some(owner), admin: Some(staff.clone()), tech: Some(staff.clone()), billing: Some(staff) },
    ["tr.apiname.com", "eu.apiname.com"],
)
.tld_attribute("<key>", "<value>"); // les clés : client.products().tld("com.tr").await?.unwrap().attributes

// Avec des handles de contact existants
let request = DomainRegisterRequest::new("example.com", ContactHandles::same("DNA-12345"), ["tr.apiname.com", "eu.apiname.com"]);
client.domains().register(request).await?;

let renewal = client.domains().renew_check("example.com", Some(1)).await?;   // gratuit
if renewal.renewable {
    client.domains().renew("example.com", 1).await?;
}

let check = client.domains().transfer_check("example.com", "EPP-CODE").await?;  // gratuit
if check.transferable {
    client.domains().transfer(DomainTransferRequest::new("example.com", "EPP-CODE")).await?;
}

client.domains().restore("example.com").await?;
```

Les requêtes sont vérifiées avant d'être envoyées. Un enregistrement sans les quatre rôles de contact, un contact sans
adresse e-mail ou une durée de 0 an renvoie une erreur `ErrorKind::InvalidInput` qui liste chaque problème, et rien
n'atteint l'API.

> [!NOTE]
> L'API peut répondre à un enregistrement, un renouvellement ou un transfert par un HTTP 200 avec `"success": false` (par exemple quand le
> registre refuse le domaine). Le SDK renvoie cette réponse telle quelle, comme le SDK Java : vérifiez
> `DomainOperationResult::success` et `message`. Il en va de même pour `SslOperationResult` et `ContactCreateResult`.

**Paramètres**

```rust
client.domains().lock("example.com").await?;
client.domains().unlock("example.com").await?;
client.domains().set_privacy("example.com", true).await?;
client.domains()
    .set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net", "ns2.example.net"]))
    .await?;
client.domains()   // un domaine signé avec DNSSEC
    .set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net"]).dnssec_mode(DnssecMode::DisableDnssec))
    .await?;
let auth_code = client.domains().update_auth_code("example.com").await?;

client.domains().update_contacts("example.com", DomainContacts::same(contact)).await?;
client.domains().set_contact_handles("example.com", ContactHandles { tech: "DNA-12345".into(), ..Default::default() }).await?;
```

**Redirection d'URL, serveurs de noms enfants et documents**

```rust
client.domains().forwarding().set("example.com", "https://www.example.net", ForwardType::Standard).await?;
let forwarding = client.domains().forwarding().get("example.com").await?;   // None si non définie
client.domains().forwarding().delete("example.com").await?;

// La version IP de chaque adresse est détectée
client.domains().child_name_servers().add("example.com", "ns1.example.com", ["192.0.2.10", "2001:db8::10"]).await?;
client.domains().child_name_servers()
    .update(DomainHostUpdateRequest::new("example.com", "ns1.example.com", ["192.0.2.20"]))
    .await?;
client.domains().child_name_servers().delete("example.com", "ns1.example.com").await?;

// Documents du registre (surtout .tr)
let document = DocumentUpload::from_file(DocumentType::IdentityCard, "id.pdf")?;
client.domains().documents().upload("example.com.tr", [document]).await?;
let documents: Vec<Document> = client.domains().documents().list_all("example.com.tr").try_collect().await?;
```

### Contacts

Les handles de contact sont des contacts réutilisables. Créez-en un une fois et utilisez-le pour plusieurs domaines.

```rust
let created = client.contacts().create(contact.clone()).await?;
let handle = created.handle;                                         // par exemple DNA-12345

let details = client.contacts().get(&handle).await?;                 // le contact et son état de vérification
let result = client.contacts().update(&handle, contact).await?;
println!("{} of {} domains updated", result.stats.success_count, result.stats.total_domain_count);

let verifications = client.contacts().verification(&handle).await?;
client.contacts().resend_verification(&handle).await?;
```

### DNS

Enregistrements de zone des domaines qui utilisent les serveurs de noms de la plateforme. Les enregistrements sont résolus publiquement dès que le domaine utilise ces serveurs de noms,
ce dont `apply` se charge.

```rust
for set in client.dns().list("example.com").await? {
    println!("{} {} {} {:?}", set.name, set.record_type, set.ttl, set.values());
}

let dns = client.dns();
dns.add(DnsAddRequest::new("example.com", "www", RecordType::A, ["192.0.2.10"])).await?;
dns.add(DnsAddRequest::new("example.com", "@", "txt", ["v=spf1 include:_spf.example.net ~all"])).await?;   // "txt" devient TXT
dns.add(DnsAddRequest::new("example.com", "@", RecordType::Mx, ["10 mx1.example.net", "20 mx2.example.net"]).ttl(3600)).await?;
dns.update(DnsUpdateRequest::new("example.com", "www", RecordType::A, ["192.0.2.20"]).ttl(300)).await?;
dns.delete(DnsDeleteRequest::new("example.com", "www", RecordType::A, "192.0.2.20")).await?;

let applied = dns.apply("example.com").await?;
```

Le SDK vérifie ce que l'API refuserait : un CNAME à l'apex, un MX sans sa priorité, une valeur CAA sans ses
trois parties, un enregistrement A qui n'est pas en IPv4, un TTL hors de la plage 1–86400. Les appels de zone sont espacés de 1 seconde, la
limite de la passerelle. L'environnement de test OTE répond aux appels de zone par un HTTP 500 ; le code DNS ne peut donc être essayé qu'en production.

### DNSSEC

```rust
let info = client.dnssec().get("example.com").await?;      // status, ds_records, allowed_algorithms, ...

let record: DsData = "12345 13 2 49FD46E6C4B45C55D4AC69CBD3CD34AC1AFE51DE".parse()?;
client.dnssec().add("example.com", [record.clone()]).await?;
client.dnssec().remove("example.com", record).await?;
client.dnssec().disable("example.com").await?;            // supprime tous les enregistrements DS
```

### SSL

Un domaine peut avoir plusieurs certificats ; un certificat existant est donc désigné par son **identifiant de
certificat** (un GUID), jamais par son nom de domaine. `initialize` et `renew` renvoient cet identifiant : conservez-le.
`list` retrouve les identifiants des certificats que vous possédez déjà.

```rust
// 1. Acheter (débite le solde). Les codes de package proviennent de client.products().ssls(...).
let approvers = client.ssl().approvers(SslApproversRequest::for_purchase("example.com", "PKG-CODE")).await?;
let order = client.ssl().initialize("example.com", "PKG-CODE").await?;
let certificate_id = order.certificate_id;                  // conservez-le : chaque appel ultérieur l'utilise

// 2. Un CSR et une clé privée, si vous n'en avez pas
let csr = client.ssl().generate_csr(
    SslGenerateCsrRequest::new("example.com")
        .country("TR").state("Istanbul").locality("Istanbul").organization("Example Ltd").organization_unit("IT"),
).await?;

// 3. Configurer dans les 15 jours : CSR, méthode de validation et contact
client.ssl().configure(
    SslConfigureRequest::new(&certificate_id, &csr.csr, certificate_contact.clone())
        .authentication_type(AuthenticationType::Email)
        .approver_mail(&approvers[0])
        .web_server_type(WebServerType::Apache),
).await?;

// 4. Attendre l'émission, puis télécharger
let status = client.ssl().status(&certificate_id).await?;
if status.is_issued {
    let content = client.ssl().content(&certificate_id).await?;
    std::fs::write("fullchain.pem", content.full_chain())?;
}

let details = client.ssl().get(&certificate_id).await?;     // abonnement, configuration, contacts
client.ssl().reissue(SslReissueRequest::new(&certificate_id).csr(new_csr).private_key(new_key)).await?;
client.ssl().revoke(&certificate_id).await?;                // irréversible
```

**Lister les certificats et trouver un identifiant**

```rust
let page = client.ssl().list(SslListRequest::new().domain_name("example.com")).await?;   // les plus récents d'abord

let mut all = client.ssl().list_all(SslListRequest::new());
while let Some(c) = all.try_next().await? {
    println!("{} {} {} {:?}", c.certificate_id, c.domain_name, c.status, c.subscription_expires_at);
}

// Les adresses d'approbation d'un certificat que vous possédez déjà
let mails = client.ssl().approvers(SslApproversRequest::for_certificate(&certificate_id)).await?;
```

**Renouveler** (cette opération débite votre solde)

```rust
// Autorisé à partir de 30 jours avant details.subscription.expires_at. Le renouvellement est un nouveau certificat avec son propre identifiant.
let renewal = client.ssl().renew(&certificate_id).await?;
client.ssl().configure(
    SslConfigureRequest::new(&renewal.certificate_id, &new_csr, certificate_contact)
        .authentication_type(AuthenticationType::Email)
        .approver_mail(&approvers[0]),
).await?;
```

Un certificat acheté ou renouvelé doit être configuré dans les 15 jours ; passé ce délai, il est annulé. Pour renouveler,
appelez `renew`, puis `configure` avec le nouvel identifiant. N'achetez pas de nouveau avec `initialize` : il s'agirait d'un
achat distinct au prix d'achat, et l'ancien certificat ne serait jamais marqué comme renouvelé. Un identifiant de certificat
vide ou qui n'est pas un GUID (par exemple un nom de domaine passé par du code plus ancien) est refusé avant tout envoi.

### Hébergement

```rust
let packages = client.hosting().packages().await?;
let created = client.hosting().create("example.com", "starter-linux").await?;          // débite le solde
let status = client.hosting().status(HostingStatusRequest::by_service_id(&created.service_id)).await?;
if status.is_active() {
    println!("{} {}", status.panel_address, status.username);
}
let hosting = client.hosting().get(HostingGetRequest::by_domain("example.com")).await?;
let plans = client.hosting().plans(None).await?;
```

### Commandes et produits

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

## Requêtes et le trait `Request`

Chaque opération est un type de requête nommé d'après sa commande CLI : `dna domain renew` correspond à `DomainRenewRequest`,
`dna domain renew-check` à `DomainRenewCheckRequest`, `dna dns add` à `DnsAddRequest`. Les méthodes des clients les construisent
pour vous, et vous pouvez aussi les créer et les envoyer vous-même :

```rust
let request = DomainRenewRequest::new("example.com", 2);
let endpoint = request.endpoint();
println!("{endpoint} is {}", endpoint.safety);   // "POST /api/v1/domains/renew is billable"

let result = client.send(&request).await?;

// La même chose, avec les détails HTTP : ID de corrélation, état du quota souple, tentatives, durée
let response = client.send_with_response(&request).await?;
println!("{} quota {:?}", response.correlation_id, response.quota);
```

Rust n'a pas d'héritage ; la base de chaque requête est un trait :

| Trait | Pour |
|---|---|
| `Request` | Toutes les opérations. `type Response` est le type de la réponse ; `Empty` pour les opérations qui ne renvoient rien (verrouiller, déverrouiller, ajouter un enregistrement DNS, ...). |
| `PagedRequest<T>` | Les opérations de liste (`Response = Page<T>`). Donne accès au `Paging` (skip, limit, sort), et fonctionne avec `Client::list_all`. |

Une requête se décrit avec `endpoint()` : méthode, chemin, classe de sécurité et caractère idempotent ou non. Les trois autres
méthodes ont des implémentations par défaut : `query()` renvoie la chaîne de requête (`Query` omet les valeurs vides), `body()` le corps JSON
(`JsonBody` en construit un en omettant les valeurs vides, ou utilisez `serde_json::json!`), et `validate()` collecte les problèmes
avant l'envoi. Pour appeler un endpoint que le SDK ne couvre pas encore, écrivez votre propre requête :

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
        Endpoint::get("/api/v1/domains/notes")          // GET est une lecture
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

Votre requête bénéficie de tout ce que reçoivent les requêtes intégrées : identifiants, nouvelles tentatives selon la classe de sécurité, espacement, mode lecture seule et
gestion des erreurs. Pour une modification, utilisez `Endpoint::post(path, OperationSafety::Write)` (ou `Billable`, `Destructive`), et
n'ajoutez `.idempotent()` que si l'envoyer deux fois est sans conséquence.

## Erreurs

Toute requête échouée renvoie une `Error`, y compris quand la requête n'a pas passé ses vérifications et que rien n'a été envoyé.

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
        // Une requête facturable ou modificatrice a échoué sur le réseau : elle a peut-être atteint l'API.
        // Vérifiez l'état avant de l'envoyer de nouveau.
        let info = client.domains().info("example.com").await?;
    }
    Err(e) => return Err(e.into()),
}
```

| Méthode | Signification |
|---|---|
| `kind()` | Une classification stable (tableau ci-dessous). Branchez votre logique dessus, pas sur le message. |
| `code()` | Le code d'erreur de l'API (comme `Dna.DomainService:Domain:10007`), ou l'un des codes du SDK ci-dessous. Les plus courants sont des constantes dans `domainnameapi::codes`. |
| `message()` | Ce qui s'est passé. Les messages génériques sont remplacés par les détails de l'API. |
| `status()` | Le statut HTTP, ou `None` si aucune réponse n'est arrivée. |
| `details()`, `validation_errors()` | Détails supplémentaires et problèmes par champ (renvoyés par l'API, ou issus des vérifications avant envoi). |
| `correlation_id()` | Le `X-Correlation-Id` de la passerelle. Indiquez-le en contactant le support. |
| `hint()` | Une courte suggestion sur la suite à donner. |
| `is_retryable()` | Indique si rejouer la même requête plus tard peut réussir. |
| `is_outcome_unknown()` | La requête a peut-être été exécutée : une modification a échoué sur le réseau, ou l'API a indiqué que le registre l'a peut-être déjà effectuée. |
| `is_not_found()`, `is_insufficient_balance()` | Raccourcis pour les cas courants. |
| `operation()`, `safety()` | La requête en échec (`POST /api/v1/domains/renew`) et sa classe de sécurité. |
| `retry_after()`, `response_body()` | Le `Retry-After` de la passerelle et la réponse brute. |

`Error` implémente `std::error::Error` et `Display` (`[CODE] message (HTTP 400); field: problem`), et fonctionne donc
avec `?`, `anyhow` et `thiserror`.

| `kind()` | Quand |
|---|---|
| `Auth` | Identifiants erronés ou absents, ou revendeur inconnu ou suspendu dans cet environnement (401, redirection). |
| `Forbidden` | Authentifié mais non autorisé (403). |
| `NotFound` | Le domaine, le contact, le certificat, la commande ou une autre ressource n'existe pas (404). |
| `Validation` | L'API a refusé l'entrée ou une règle métier a échoué : champ invalide, domaine verrouillé, solde insuffisant (400, 422). |
| `Conflict` | L'état de la ressource n'autorise pas l'opération (409). |
| `RateLimited` | Toujours limité en débit après les nouvelles tentatives automatiques (429). |
| `Server` | Une défaillance côté serveur (5xx). Souvent temporaire. |
| `Network`, `Timeout` | Pas de réponse : DNS, TLS, proxy, connexion réinitialisée, ou aucune réponse dans le délai imparti. |
| `InvalidInput` | La requête n'a pas passé ses vérifications, ou un paramètre du client est invalide ; rien n'a été envoyé. |
| `ReadOnly` | Bloquée par le mode lecture seule ; rien n'a été envoyé. |
| `Unknown` | Tout le reste, y compris une réponse que le SDK n'a pas pu lire. |

`ErrorKind::as_str()` donne les noms utilisés par la CLI (`auth`, `not_found`, `rate_limited`, ...). Codes définis par le SDK lui-même :

| Code | Signification |
|---|---|
| `INVALID_RESELLER` | La passerelle a redirigé vers sa page de suspension : le Reseller ID est inconnu, inactif ou suspendu **dans cet environnement**. |
| `INVALID_API_KEY` | La clé ne correspond pas au revendeur. Les clés de production et d'OTE diffèrent. |
| `MISSING_API_KEY`, `MISSING_RESELLER_ID`, `NO_CREDENTIALS` | Un identifiant est vide. |
| `SUBRESELLER_NOT_ALLOWED` | Les comptes de sous-revendeur ne peuvent pas utiliser l'API. Utilisez les identifiants du revendeur principal. |
| `UNAUTHORIZED` | Toute autre 401. |
| `RATE_LIMITED` | 429 après les nouvelles tentatives. |
| `NETWORK`, `TIMEOUT` | Aucune réponse n'est arrivée. |
| `READ_ONLY` | Bloquée par le mode lecture seule. |
| `INVALID_INPUT`, `INVALID_CONFIGURATION` | Une requête n'a pas passé ses vérifications, ou un paramètre du client est invalide. |
| `EMPTY_RESPONSE`, `INVALID_RESPONSE` | Une réponse réussie était vide ou illisible. |
| `HTTP_<status>` | L'API n'a envoyé aucun code d'erreur. |

## Sécurité, nouvelles tentatives et limites de débit

Chaque requête a une classe de sécurité, la même que celle de la CLI `dna` :

| `OperationSafety` | Signification | Exemples |
|---|---|---|
| `Read` | Aucun effet de bord, même quand l'API utilise POST ou PUT | recherche, détails, listes, vérifications de renouvellement et de transfert, génération de CSR, statut d'un certificat |
| `Write` | Une modification gratuite et réversible | enregistrements DNS, verrouillage, confidentialité, serveurs de noms, contacts |
| `Billable` | Débite votre solde | enregistrement, renouvellement, transfert, restauration, achat et renouvellement de SSL, commande d'hébergement |
| `Destructive` | Supprime quelque chose ou est difficile à annuler | suppression d'un enregistrement DNS, retrait de DNSSEC, révocation d'un certificat |

- **Le mode lecture seule** (`read_only(true)` ou `DNA_READ_ONLY=true`) bloque, avant envoi, tout ce qui n'est pas `Read`.
  Utilisez-le pour explorer, dans les rapports et dans les tableaux de bord.
- **Nouvelles tentatives.** Les réponses 429 sont toujours rejouées, en respectant `Retry-After`. C'est sans risque, même pour les requêtes facturables,
  car la passerelle les refuse avant que quoi que ce soit ne s'exécute. Les erreurs réseau, les timeouts et les réponses 502/503/504 ne sont rejoués
  que pour les lectures et pour les écritures que la CLI marque comme idempotentes (verrouillage, déverrouillage, confidentialité, serveurs de noms, mise à jour DNS et
  apply). Un enregistrement ou un renouvellement n'est jamais envoyé deux fois.
- **Espacement.** La passerelle autorise 5 requêtes par seconde et par clé, et 2 par seconde pour les recherches de domaines. Le client
  espace les requêtes pour rester sous ces limites, et espace les appels de zone DNS de 1 seconde. Les informations de domaine et la liste des TLD
  ne sont pas limitées en débit.
- **Quota souple.** Les recherches et les consultations d'informations coûtent 1 point, tout le reste 10. Au-delà de 200 points, la passerelle ralentit chaque
  requête jusqu'à 5 secondes ; les points décroissent de 20 % par minute. `send_with_response` expose cet état dans `quota`.

## Code asynchrone et code bloquant

Le SDK est asynchrone et a besoin d'un runtime Tokio (celui que démarre `#[tokio::main]`, ou celui de votre serveur). Les futures sont
`Send` ; elles fonctionnent donc dans `tokio::spawn` et dans les handlers axum et actix. Les listes peuvent être lues page par page (`list`) ou comme un
`Stream` d'éléments (`list_all`), qui ne récupère la page suivante que lorsqu'elle est nécessaire :

```rust
use futures_util::TryStreamExt;

let expiring: Vec<Domain> = client
    .domains()
    .list_all(DomainListRequest::new())
    .try_filter(|d| std::future::ready(d.remaining_days < 30))
    .try_collect()
    .await?;
```

Il n'y a pas d'API bloquante : elle doublerait la surface de l'API pour un faible gain. Dans du code synchrone, exécutez les appels sur un
runtime :

```rust
let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
let account = runtime.block_on(client.account().get())?;
```

## Tester votre code

Le SDK n'a besoin que d'un endpoint HTTP. Pour tester le code qui l'utilise sans appeler l'API, faites pointer `base_url` vers un serveur
local (par exemple un serveur construit avec `wiremock`, `httpmock` ou `axum`) et désactivez l'espacement :

```rust
let client = Client::builder()
    .credentials("test", "test")
    .base_url(format!("http://{}", server_address))
    .pace_requests(false)
    .build()?;
```

Les modèles implémentent `Default` et `Deserialize` ; les données de test peuvent donc provenir de JSON. Les erreurs pour vos mocks peuvent être construites
avec `Error::new(kind, code, message)`. Pour des tests contre la vraie API, utilisez l'OTE avec sa propre clé.

## Structure du projet

```
src/
├── lib.rs           Documentation de la crate et ré-exports
├── client.rs        Client, ClientBuilder, Environment : envoi, nouvelles tentatives, espacement, analyse des erreurs, list_all
├── request.rs       Request, PagedRequest, Endpoint, OperationSafety, Query, JsonBody, Validator, Page, Empty
├── error.rs         Error, ErrorKind, codes
├── de.rs            Décodage tolérant et macros qui déclarent les modèles et les enums (interne)
├── account.rs       AccountClient     Soldes, profil et transactions
├── contacts.rs      ContactsClient    Handles de contact
├── domains/         DomainsClient     Domaines, plus forwarding(), child_name_servers() et documents()
├── dns.rs           DnsClient         Enregistrements de zone
├── dnssec.rs        DnssecClient      Enregistrements DS
├── ssl.rs           SslClient         Certificats
├── hosting.rs       HostingClient     Packages et services d'hébergement
├── orders.rs        OrdersClient      Commandes
└── products.rs      ProductsClient    TLD, produits SSL et prix
tests/               Tests sur une fausse API, le test de couverture de la spécification et testdata/openapi.json
examples/            Programmes exécutables
```

Chaque module de domaine fonctionnel a la même forme : les modèles et enums de ses réponses, un type de requête par opération
(`XxxRequest`) et un `XxxClient`.

## Développement

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                    # tests unitaires, sur fausse API, de spécification et de documentation ; aucun n'appelle la vraie API
cargo build --examples
```

Sans toolchain locale, utilisez Docker :

```bash
docker run --rm -v "$PWD":/src -w /src rust:1.83 sh -c 'rustup component add rustfmt clippy; cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo test'
```

Les tests n'appellent jamais la vraie API : ils utilisent un petit serveur HTTP sur 127.0.0.1 (`tests/common`). La CI les exécute sur la
version minimale de Rust prise en charge (1.83) et sur la dernière version stable.

`tests/testdata/openapi.json` est le document d'API que la CLI `dna` embarque. `tests/spec.rs` vérifie que chaque opération
qu'il contient a exactement un type de requête avec la méthode, le chemin, la classe de sécurité et l'idempotence de la CLI, que chaque requête
n'envoie que des paramètres de requête et des propriétés de corps décrits par le document (objets imbriqués compris, avec les propriétés obligatoires
présentes), et que les types de réponse utilisent les noms du document. Quand l'API gagne une opération, mettez à jour le document :
le test échoue tant que le type de requête n'existe pas.

## Migrer depuis la 1.x

La 2.0 est un nouveau SDK. La 1.x était un portage de la bibliothèque PHP : un client bloquant `DomainNameApi` qui renvoyait des
enveloppes `serde_json::Value` non typées. La 2.0 est asynchrone, avec des requêtes et des modèles typés, `Result<T, Error>` et un builder.

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
| `result["result"] == "ERROR"` | `Err(e)` avec `e.kind()` et `e.code()` |

Les appels d'approbation, d'annulation et de rejet de transfert ainsi que `sync_from_registry` de la 1.x ne font pas partie du document de
l'API REST et n'ont pas d'équivalent en 2.0. La MSRV reste 1.83 ; `ureq` est remplacé par `reqwest`, la 2.0 a donc besoin d'un runtime Tokio.
La 1.x envoyait des rapports d'erreurs à Sentry ; la 2.0 n'envoie rien nulle part, sauf à l'API.

## Liens

- [CLI `dna` et serveur MCP](https://github.com/domainreseller/cli) : la même API depuis le terminal et depuis des agents IA.
- [Documentation de l'API](https://www.domainnameapi.com/docs/)
- [Changelog](CHANGELOG.md)

## Licence

Apache License 2.0. Voir [LICENSE](LICENSE).
