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

[Domain Name API](https://www.domainnameapi.com) reseller platforması üçün rəsmi Rust SDK. REST API-nin bütün
67 əməliyyatını tipli sorğular və modellərlə əhatə edir: domenlər, əlaqələr, DNS, DNSSEC, SSL sertifikatları,
hostinq, sifarişlər, məhsullar və hesabınız.

```rust
let client = Client::builder().credentials(reseller_id, api_key).environment(Environment::Ote).build()?;

let search = client.domains().search("example.com").await?;
if search.info.is_available() {
    let request = DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact), ["tr.apiname.com", "eu.apiname.com"]);
    client.domains().register_with_contacts(request).await?;
}
```

- **Hər şey tiplidir.** Hər əməliyyat bir sorğu tipidir, hər cavab isə sənədləşməsi olan struct-dır. Map-lər və JSON parsinq yoxdur.
- **Öz sorğularınız.** Hər sorğu `Request` trait-ini reallaşdırır. SDK-nın hələ əhatə etmədiyi endpoint üçün onu özünüz reallaşdırın.
- **Pulla təhlükəsiz işləyir.** Hər sorğu bildirir ki, oxuyur, yazır, balansınızdan ödəniş alır, yoxsa nəyisə silir. Ödənişli sorğular heç vaxt təkrarlanmır, yalnız oxuma rejimi isə hər dəyişikliyi bloklayır.
- **Daxili davamlılıq.** Sorğu limitlərinə avtomatik əməl edilir, 429 cavabları təkrarlanır, oxuma sorğularında isə şəbəkə xətaları təkrarlanır.
- **Aydın xətalar.** Sabit növü, API-nin xəta kodu, sahə səviyyəsində yoxlama xətaları, korrelyasiya ID-si və məsləhəti olan tək bir `Error` tipi.
- **Müasir Rust.** Tokio üzərində async, rustls ilə `reqwest` (OpenSSL yoxdur), serde modelləri, `Stream` kimi tənbəl (lazy) səhifələmə, klonlana bilən `Send + Sync` klient, `#![deny(missing_docs)]`.
- **Məxfi.** Telemetriya yoxdur. API-yə göndərdiyiniz sorğulardan başqa heç nə prosesinizdən kənara çıxmır.

O, [`dna`, Domain Name API CLI-nin](https://github.com/domainreseller/cli) Rust qarşılığıdır: eyni API
sənədi, eyni əməliyyatlar və qruplar, eyni təhlükəsizlik sinifləri, təkrar qaydaları və xəta kodları.

## Məzmun

- [Quraşdırma](#quraşdırma)
- [Giriş məlumatları](#giriş-məlumatları)
- [Sürətli başlanğıc](#sürətli-başlanğıc)
- [Konfiqurasiya](#konfiqurasiya)
- [SDK-dan istifadə](#sdk-dan-istifadə): [hesab](#hesab) · [domenlər](#domenlər) · [əlaqələr](#əlaqələr) · [DNS](#dns) · [DNSSEC](#dnssec) · [SSL](#ssl) · [hostinq](#hostinq) · [sifarişlər və məhsullar](#sifarişlər-və-məhsullar)
- [Sorğular və `Request` trait-i](#sorğular-və-request-trait-i)
- [Xətalar](#xətalar)
- [Təhlükəsizlik, təkrarlar və sorğu limitləri](#təhlükəsizlik-təkrarlar-və-sorğu-limitləri)
- [Asinxron və bloklayan kod](#asinxron-və-bloklayan-kod)
- [Kodunuzu test etmək](#kodunuzu-test-etmək)
- [Layihənin strukturu](#layihənin-strukturu)
- [SDK-nın inkişafı](#sdk-nın-inkişafı)
- [1.x versiyasından keçid](#1x-versiyasından-keçid)

## Quraşdırma

> [!IMPORTANT]
> **Həmişə [son buraxılışdan](https://github.com/domainreseller/rust-dna/releases/latest) istifadə edin.** Buraxılış paketləri versiyalaşdırılır və test edilir.
> Yaşıl **Code → Download ZIP** düyməsi isə inkişaf branch-ini yükləyir.

```toml
[dependencies]
domainnameapi = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"   # list_all axınlarını oxumaq üçün
```

Crate crates.io-da dərc olunana qədər onu buraxılış tag-ından götürün:

```toml
domainnameapi = { git = "https://github.com/domainreseller/rust-dna", tag = "v2.0.0" }
```

**Tələblər:** Rust 1.83 və ya daha yeni versiya və Tokio runtime. Asılılıqlar bunlardır: `reqwest` (rustls ilə, OpenSSL
yoxdur), `serde`/`serde_json`, `tokio` (yalnız taymerlər), `futures-util`, `chrono`, `base64` və `httpdate`.

## Giriş məlumatları

API hər sorğunu iki dəyərlə autentifikasiya edir, SDK onları sizin yerinizə göndərir:

| Dəyər | Header | Harada tapılır |
|---|---|---|
| Reseller ID (UUID) | `__reseller` | Reseller paneli → **API settings** |
| API açarı | `X-API-KEY` | Reseller paneli → **API settings** |

> [!NOTE]
> Bunlar **API giriş məlumatlarıdır**: panelə giriş məlumatlarınız burada işləmir. **Canlı mühit və OTE (test mühiti)
> fərqli API açarlarından istifadə edir.** Birini digərində işlətmək `INVALID_API_KEY` və ya `INVALID_RESELLER` xətası ilə nəticələnir.

## Sürətli başlanğıc

```rust
use domainnameapi::contacts::Contact;
use domainnameapi::domains::{DomainContacts, DomainListRequest, DomainRegisterWithContactsRequest};
use domainnameapi::{Client, Environment};
use futures_util::TryStreamExt;

#[tokio::main]
async fn main() -> Result<(), domainnameapi::Error> {
    // OTE test mühitidir: real API, öz məlumatları, real pul yoxdur.
    let client = Client::builder()
        .credentials("your-reseller-id", "your-api-key")
        .environment(Environment::Ote)
        .build()?;

    // 1. Giriş məlumatlarını və balansı yoxlayın.
    let account = client.account().get().await?;
    println!("{}: {} USD", account.reseller_name, account.usd_balance);

    // 2. Domen boşdurmu və qiyməti nədir?
    let search = client.domains().search("my-new-domain.com").await?;
    println!("{} {} {}", search.info.status, search.info.price, search.info.currency);

    // 3. Onu dörd rolun hamısı üçün bir əlaqə ilə qeydiyyatdan keçirin. Bu, balansdan ödəniş alır.
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

    // 4. Bütün domenləri siyahılayın; səhifələr axın oxunduqca yüklənir.
    let mut domains = client.domains().list_all(DomainListRequest::new());
    while let Some(domain) = domains.try_next().await? {
        println!("{} expires in {} days", domain.domain_name, domain.remaining_days);
    }
    Ok(())
}
```

[examples](examples) qovluğunda bu ssenarilər üçün işə salına bilən proqramlar var (`quickstart`, `register`, `dns`, `ssl`,
`custom_request`, `error_handling`). Onlar `DNA_*` mühit dəyişənlərini oxuyur, ona görə onları OTE-yə yönəldin:

```bash
export DNA_RESELLER_ID=... DNA_API_KEY=... DNA_ENV=ote
cargo run --example quickstart -- example.com
```

## Konfiqurasiya

Klient yaratmağın üç yolu var:

```rust
// Canlı mühit, defolt parametrlər
let client = Client::new(reseller_id, api_key)?;

// Bütün parametrlər
let client = Client::builder()
    .credentials(reseller_id, api_key)
    .environment(Environment::Ote)
    .timeout(Duration::from_secs(30))
    .read_only(true)
    .build()?;

// dna CLI-nin oxuduğu eyni mühit dəyişənləri ...
let client = Client::from_env()?;

// ... və üstəlik öz parametrləriniz
let client = ClientBuilder::from_env()?.http_client(my_reqwest_client).build()?;
```

| `ClientBuilder` metodu | Defolt | Mənası |
|---|---|---|
| `credentials(reseller_id, api_key)` | (məcburi) | Giriş məlumatları. Biri boş olduqda `build()` `NO_CREDENTIALS` xətası ilə uğursuz olur. |
| `environment(...)` | `Production` | `Production` (`https://api.domainresellerapi.com`) və ya `Ote` (`https://ote.domainresellerapi.com`). |
| `base_url(...)` | yoxdur | Mühitin ünvanını əvəz edir, məsələn proxy üçün. Yol prefiksi saxlanılır. |
| `timeout(...)` | 60 s | Bir cəhdin vaxt limiti. |
| `max_retries(...)` | 2 | Uğursuz sorğunun neçə dəfə təkrarlanacağı ([təkrarlara](#təhlükəsizlik-təkrarlar-və-sorğu-limitləri) baxın). 0 təkrarları söndürür. |
| `max_retry_delay(...)` | 30 s | Təkrardan əvvəl ən uzun gözləmə. |
| `pace_requests(...)` | `true` | Gateway-in sorğu limitlərinin altında qalmaq üçün sorğular arasında fasilə qoyur. |
| `read_only(...)` | `false` | Oxuma olmayan hər sorğunu göndərilməzdən əvvəl bloklayır. |
| `user_agent(...)` | `DomainNameAPI-Rust/<version>` | `User-Agent` header-i. |
| `app(name, version)` | `rust`, SDK versiyası | `X-App` və `X-Version` header-ləri: SDK-dan istifadə edən tətbiq (aşağıya baxın). |
| `http_client(...)` | SDK-nın özününkü | Öz `reqwest::Client` obyektiniz (proxy, TLS parametrləri). |

`from_env()` bunları oxuyur:

| Dəyişən | Mənası |
|---|---|
| `DNA_RESELLER_ID`, `DNA_API_KEY` | Giriş məlumatları. |
| `DNA_ENV` | `production` və ya `ote` (həmçinin `prod`, `live`, `test`, `sandbox`). |
| `DNA_TEST_MODE` | `true`, `1`, `on` və ya `yes` OTE-ni seçir. `DNA_ENV` təyin edilməyibsə istifadə olunur. |
| `DNA_BASE_URL` | İstəyə bağlı baza ünvanı əvəzləməsi. |
| `DNA_READ_ONLY` | `true` yalnız oxuma rejimini aktiv edir. |
| `DNA_APP`, `DNA_APP_VERSION` | SDK-dan istifadə edən tətbiq və onun versiyası; `X-App` və `X-Version` kimi göndərilir. |

`Client` `Clone + Send + Sync`-dir və onu klonlamaq ucuzdur (içində `Arc` saxlayır). Hər giriş məlumatı dəsti üçün bir
dənə yaradın və onu tətbiqin bütün işləmə müddətində paylaşın: sorğu tempi hər klient üçün ayrıca tənzimlənir.

### Tətbiqinizi tanıtmaq

Hər sorğu API komandasına onu hansı inteqrasiyanın göndərdiyini bildirən `X-App` və `X-Version` header-lərini daşıyır.
Standart olaraq onlar `rust` və SDK-nın versiyasıdır. SDK üzərində panel modulu və ya başqa bir məhsul qurursunuzsa,
öz dəyərlərinizi təyin edin:

```rust
let client = Client::builder()
    .credentials(reseller_id, api_key)
    .app("WiseCP5", "V2.0.20-beta") // X-App: wisecp5, X-Version: 2.0.20 kimi göndərilir
    .build()?;
```

Ad kiçik hərflərə çevrilir və yalnız `a-z`, `0-9`, `.`, `_` və `-` simvolları saxlanılır; versiyada yalnız rəqəmlər və
nöqtələr qalır. Boş ad `rust`-a, boş versiya isə SDK-nın versiyasına qayıdır. `from_env()` eyni dəyərləri `DNA_APP` və
`DNA_APP_VERSION` dəyişənlərindən oxuyur. `User-Agent` header-i dəyişmir.

## SDK-dan istifadə

API-nin hər sahəsinin CLI-nin komanda qrupları kimi adlandırılmış öz klienti var: `client.account()`,
`client.contacts()`, `client.domains()` (`forwarding()`, `child_name_servers()` və `documents()` ilə birlikdə),
`client.dns()`, `client.dnssec()`, `client.ssl()`, `client.hosting()`, `client.orders()` və `client.products()`.
Məcburi girişlər arqument kimi verilir; istəyə bağlı girişləri olan əməliyyatlar `new(...)` ilə qurulan və zəncirvari
setter-lərlə tənzimlənən sorğu qəbul edir. Balansınızdan ödəniş alan metodlar bunu sənədləşmələrində bildirir.

### Hesab

```rust
let account = client.account().get().await?;          // reseller_id, reseller_name, try_balance, usd_balance
let profile = client.account().profile().await?;      // name, company_name, account_currency, status, ...

// Bu ayın ödənişləri, bütün səhifələr üzrə
let request = AccountTransactionsRequest::new()
    .charge_type(ChargeType::Subtract)
    .start_date("2026-11-01T00:00:00Z".parse()?)
    .transaction_types([TransactionType::Register, TransactionType::Renew]);
let mut transactions = client.account().all_transactions(request);
while let Some(t) = transactions.try_next().await? {
    println!("{:?} {} {} {}", t.creation_time, t.transaction_type, t.amount, t.amount_currency);
}
```

### Domenlər

**Axtarış**

```rust
let one = client.domains().search("example.com").await?;
let available = one.success && one.info.is_available();

let many = client.domains().bulk_search(["example.com", "example.net", "example.com.tr"]).await?;
for domain in many.available() {
    println!("{}: {} {}", domain.domain_name, domain.price, domain.currency);
}

// Valyuta ilə
client.domains().search_with(DomainSearchRequest::new("example.com").currency("USD")).await?;
```

**Siyahı və məlumat**

```rust
let page = client.domains().list(DomainListRequest::new().limit(50).statuses([DomainStatus::Active])).await?;

let all: Vec<Domain> = client.domains().list_all(DomainListRequest::new().domain_name("example")).try_collect().await?;

let info = client.domains().info("example.com").await?;
println!("{} {:?}", info.auth_code, info.expiration_date);
if let Some(registrant) = info.contact(ContactType::Registrant) {
    println!("{}", registrant.email);
}
```

**Qeydiyyat, yenilənmə, transfer, bərpa** (bunlar balansınızdan ödəniş alır)

```rust
// Dörd rolun hamısı üçün bir əlaqə, iki il
let request = DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact.clone()), ["tr.apiname.com", "eu.apiname.com"])
    .years(2);
client.domains().register_with_contacts(request).await?;

// Hər rol üçün fərqli əlaqələr, reyestr atributları, trustee xidməti
let request = DomainRegisterWithContactsRequest::new(
    "example.com.tr",
    DomainContacts { registrant: Some(owner), admin: Some(staff.clone()), tech: Some(staff.clone()), billing: Some(staff) },
    ["tr.apiname.com", "eu.apiname.com"],
)
.tld_attribute("<key>", "<value>"); // açarlar: client.products().tld("com.tr").await?.unwrap().attributes

// Mövcud əlaqə handle-ları ilə
let request = DomainRegisterRequest::new("example.com", ContactHandles::same("DNA-12345"), ["tr.apiname.com", "eu.apiname.com"]);
client.domains().register(request).await?;

let renewal = client.domains().renew_check("example.com", Some(1)).await?;   // pulsuz
if renewal.renewable {
    client.domains().renew("example.com", 1).await?;
}

let check = client.domains().transfer_check("example.com", "EPP-CODE").await?;  // pulsuz
if check.transferable {
    client.domains().transfer(DomainTransferRequest::new("example.com", "EPP-CODE")).await?;
}

client.domains().restore("example.com").await?;
```

Sorğular göndərilməzdən əvvəl yoxlanılır. Dörd əlaqə rolunun hamısı olmayan qeydiyyat, e-poçt ünvanı olmayan əlaqə
və ya 0 illik müddət bütün problemləri sadalayan `ErrorKind::InvalidInput` xətası qaytarır və API-yə heç nə çatmır.

> [!NOTE]
> API qeydiyyat, yenilənmə və ya transfer sorğusuna HTTP 200 və `"success": false` ilə cavab verə bilər (məsələn,
> reyestr domeni rədd etdikdə). SDK, Java SDK kimi, bu cavabı olduğu kimi qaytarır: `DomainOperationResult::success`
> və `message` dəyərlərini yoxlayın. Eyni qayda `SslOperationResult` və `ContactCreateResult` üçün də keçərlidir.

**Parametrlər**

```rust
client.domains().lock("example.com").await?;
client.domains().unlock("example.com").await?;
client.domains().set_privacy("example.com", true).await?;
client.domains()
    .set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net", "ns2.example.net"]))
    .await?;
client.domains()   // DNSSEC ilə imzalanmış domen
    .set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net"]).dnssec_mode(DnssecMode::DisableDnssec))
    .await?;
let auth_code = client.domains().update_auth_code("example.com").await?;

client.domains().update_contacts("example.com", DomainContacts::same(contact)).await?;
client.domains().set_contact_handles("example.com", ContactHandles { tech: "DNA-12345".into(), ..Default::default() }).await?;
```

**URL yönləndirmə, child nameserver-lər və sənədlər**

```rust
client.domains().forwarding().set("example.com", "https://www.example.net", ForwardType::Standard).await?;
let forwarding = client.domains().forwarding().get("example.com").await?;   // təyin edilməyibsə None
client.domains().forwarding().delete("example.com").await?;

// Hər ünvanın IP versiyası avtomatik müəyyən edilir
client.domains().child_name_servers().add("example.com", "ns1.example.com", ["192.0.2.10", "2001:db8::10"]).await?;
client.domains().child_name_servers()
    .update(DomainHostUpdateRequest::new("example.com", "ns1.example.com", ["192.0.2.20"]))
    .await?;
client.domains().child_name_servers().delete("example.com", "ns1.example.com").await?;

// Reyestr sənədləri (əsasən .tr)
let document = DocumentUpload::from_file(DocumentType::IdentityCard, "id.pdf")?;
client.domains().documents().upload("example.com.tr", [document]).await?;
let documents: Vec<Document> = client.domains().documents().list_all("example.com.tr").try_collect().await?;
```

### Əlaqələr

Əlaqə handle-ları təkrar istifadə edilə bilən əlaqələrdir. Birini bir dəfə yaradın və çox domen üçün istifadə edin.

```rust
let created = client.contacts().create(contact.clone()).await?;
let handle = created.handle;                                         // məsələn DNA-12345

let details = client.contacts().get(&handle).await?;                 // əlaqə və onun təsdiq vəziyyəti
let result = client.contacts().update(&handle, contact).await?;
println!("{} of {} domains updated", result.stats.success_count, result.stats.total_domain_count);

let verifications = client.contacts().verification(&handle).await?;
client.contacts().resend_verification(&handle).await?;
```

### DNS

Platforma nameserver-lərindəki domenlərin zona qeydləri. Domen həmin nameserver-lərdən istifadə etməyə başladıqdan sonra
qeydlər hamıya açıq şəkildə həll olunur; bunu `apply` həyata keçirir.

```rust
for set in client.dns().list("example.com").await? {
    println!("{} {} {} {:?}", set.name, set.record_type, set.ttl, set.values());
}

let dns = client.dns();
dns.add(DnsAddRequest::new("example.com", "www", RecordType::A, ["192.0.2.10"])).await?;
dns.add(DnsAddRequest::new("example.com", "@", "txt", ["v=spf1 include:_spf.example.net ~all"])).await?;   // "txt" TXT olur
dns.add(DnsAddRequest::new("example.com", "@", RecordType::Mx, ["10 mx1.example.net", "20 mx2.example.net"]).ttl(3600)).await?;
dns.update(DnsUpdateRequest::new("example.com", "www", RecordType::A, ["192.0.2.20"]).ttl(300)).await?;
dns.delete(DnsDeleteRequest::new("example.com", "www", RecordType::A, "192.0.2.20")).await?;

let applied = dns.apply("example.com").await?;
```

SDK API-nin rədd edəcəyi halları əvvəlcədən yoxlayır: apex-də CNAME, prioriteti olmayan MX, üç hissəsi olmayan CAA
dəyəri, IPv4 olmayan A qeydi, 1–86400 aralığından kənar TTL. Zona çağırışları bir-birindən 1 saniyə aralı göndərilir,
bu gateway-in limitidir. OTE test mühiti zona çağırışlarına HTTP 500 ilə cavab verir, ona görə DNS kodunu yalnız
canlı mühitdə sınamaq olar.

### DNSSEC

```rust
let info = client.dnssec().get("example.com").await?;      // status, ds_records, allowed_algorithms, ...

let record: DsData = "12345 13 2 49FD46E6C4B45C55D4AC69CBD3CD34AC1AFE51DE".parse()?;
client.dnssec().add("example.com", [record.clone()]).await?;
client.dnssec().remove("example.com", record).await?;
client.dnssec().disable("example.com").await?;            // bütün DS qeydlərini silir
```

### SSL

Bir domenin bir neçə sertifikatı ola bilər, buna görə mövcud sertifikat heç vaxt domen adı ilə deyil, **sertifikat
ID-si** (GUID) ilə göstərilir. `initialize` və `renew` bu ID-ni qaytarır: onu saxlayın. `list` artıq sahib olduğunuz
sertifikatların ID-lərini tapır.

```rust
// 1. Alış (balansdan ödəniş alır). Paket kodları client.products().ssls(...) vasitəsilə əldə edilir.
let approvers = client.ssl().approvers(SslApproversRequest::for_purchase("example.com", "PKG-CODE")).await?;
let order = client.ssl().initialize("example.com", "PKG-CODE").await?;
let certificate_id = order.certificate_id;                  // onu saxlayın: sonrakı hər çağırış onu tələb edir

// 2. Əgər yoxdursa, CSR və private key
let csr = client.ssl().generate_csr(
    SslGenerateCsrRequest::new("example.com")
        .country("TR").state("Istanbul").locality("Istanbul").organization("Example Ltd").organization_unit("IT"),
).await?;

// 3. 15 gün ərzində konfiqurasiya edin: CSR, təsdiq üsulu və əlaqə
client.ssl().configure(
    SslConfigureRequest::new(&certificate_id, &csr.csr, certificate_contact.clone())
        .authentication_type(AuthenticationType::Email)
        .approver_mail(&approvers[0])
        .web_server_type(WebServerType::Apache),
).await?;

// 4. Sertifikatın verilməsini gözləyin, sonra yükləyin
let status = client.ssl().status(&certificate_id).await?;
if status.is_issued {
    let content = client.ssl().content(&certificate_id).await?;
    std::fs::write("fullchain.pem", content.full_chain())?;
}

let details = client.ssl().get(&certificate_id).await?;     // abunəlik, konfiqurasiya, əlaqələr
client.ssl().reissue(SslReissueRequest::new(&certificate_id).csr(new_csr).private_key(new_key)).await?;
client.ssl().revoke(&certificate_id).await?;                // geri qaytarıla bilməz
```

**Sertifikatların siyahısı və ID-nin tapılması**

```rust
let page = client.ssl().list(SslListRequest::new().domain_name("example.com")).await?;   // ən yenisi birinci

let mut all = client.ssl().list_all(SslListRequest::new());
while let Some(c) = all.try_next().await? {
    println!("{} {} {} {:?}", c.certificate_id, c.domain_name, c.status, c.subscription_expires_at);
}

// Artıq sahib olduğunuz sertifikatın təsdiqləyici ünvanları
let mails = client.ssl().approvers(SslApproversRequest::for_certificate(&certificate_id)).await?;
```

**Yenilənmə** (bu əməliyyat balansınızdan ödəniş alır)

```rust
// details.subscription.expires_at tarixindən 30 gün əvvəldən etibarən mümkündür. Yenilənmə öz ID-si olan yeni sertifikatdır.
let renewal = client.ssl().renew(&certificate_id).await?;
client.ssl().configure(
    SslConfigureRequest::new(&renewal.certificate_id, &new_csr, certificate_contact)
        .authentication_type(AuthenticationType::Email)
        .approver_mail(&approvers[0]),
).await?;
```

Alınmış və ya yenilənmiş sertifikat 15 gün ərzində konfiqurasiya edilməlidir; əks halda ləğv olunur. Yeniləmək üçün
əvvəlcə `renew`, sonra yeni ID ilə `configure` çağırın. `initialize` ilə yenidən almayın: bu, alış qiyməti ilə ayrıca
bir alış olar və köhnə sertifikat heç vaxt yenilənmiş kimi işarələnməz. Boş və ya GUID olmayan sertifikat ID-si
(məsələn köhnə kodun ötürdüyü domen adı) heç nə göndərilməzdən əvvəl rədd edilir.

### Hostinq

```rust
let packages = client.hosting().packages().await?;
let created = client.hosting().create("example.com", "starter-linux").await?;          // balansdan ödəniş alır
let status = client.hosting().status(HostingStatusRequest::by_service_id(&created.service_id)).await?;
if status.is_active() {
    println!("{} {}", status.panel_address, status.username);
}
let hosting = client.hosting().get(HostingGetRequest::by_domain("example.com")).await?;
let plans = client.hosting().plans(None).await?;
```

### Sifarişlər və məhsullar

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

## Sorğular və `Request` trait-i

Hər əməliyyat CLI komandasının adını daşıyan sorğu tipidir: `dna domain renew` `DomainRenewRequest`-dir,
`dna domain renew-check` `DomainRenewCheckRequest`-dir, `dna dns add` `DnsAddRequest`-dir. Sahə metodları onları
sizin üçün qurur, siz də onları özünüz yarada və göndərə bilərsiniz:

```rust
let request = DomainRenewRequest::new("example.com", 2);
let endpoint = request.endpoint();
println!("{endpoint} is {}", endpoint.safety);   // "POST /api/v1/domains/renew is billable"

let result = client.send(&request).await?;

// Eyni şey, HTTP təfərrüatları ilə: korrelyasiya ID-si, yumşaq kvota vəziyyəti, cəhdlər, müddət
let response = client.send_with_response(&request).await?;
println!("{} quota {:?}", response.correlation_id, response.quota);
```

Rust-da varislik yoxdur; hər sorğunun əsası trait-dir:

| Trait | Nə üçün |
|---|---|
| `Request` | Hər əməliyyat. `type Response` cavab tipidir; heç nə qaytarmayan əməliyyatlar (kilid, kilidi açmaq, DNS qeydi əlavə etmək, ...) üçün `Empty`. |
| `PagedRequest<T>` | Siyahı əməliyyatları (`Response = Page<T>`). `Paging`-ə (skip, limit, sort) giriş verir və `Client::list_all` ilə işləyir. |

Sorğu özü haqqında `endpoint()` ilə məlumat verir: metod, yol, təhlükəsizlik sinfi və idempotent olub-olmaması. Digər üç
metodun defolt reallaşdırması var: `query()` query sətrini qaytarır (`Query` boş dəyərləri buraxır), `body()` JSON
gövdəsini qaytarır (`JsonBody` gövdə qurur və boş dəyərləri buraxır, ya da `serde_json::json!` istifadə edin),
`validate()` isə göndərmədən əvvəl problemləri toplayır. SDK-nın hələ əhatə etmədiyi endpoint-i çağırmaq üçün özünüz yazın:

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
        Endpoint::get("/api/v1/domains/notes")          // GET oxumadır
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

Sizin sorğunuz daxili sorğuların aldığı hər şeyi alır: giriş məlumatları, təhlükəsizlik sinfinə görə təkrarlar, sorğu tempi,
yalnız oxuma rejimi və xəta emalı. Dəyişiklik üçün `Endpoint::post(path, OperationSafety::Write)` (və ya `Billable`,
`Destructive`) istifadə edin və `.idempotent()`-i yalnız sorğunu iki dəfə göndərmək zərərsiz olduqda əlavə edin.

## Xətalar

Hər uğursuz sorğu `Error` qaytarır, sorğu yoxlamalardan keçmədikdə və heç nə göndərilmədikdə də.

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
        // Ödənişli və ya dəyişiklik edən sorğu şəbəkədə uğursuz oldu: API-yə çatmış ola bilər.
        // Yenidən göndərməzdən əvvəl vəziyyəti yoxlayın.
        let info = client.domains().info("example.com").await?;
    }
    Err(e) => return Err(e.into()),
}
```

| Metod | Mənası |
|---|---|
| `kind()` | Sabit təsnifat (aşağıdakı cədvəl). Mesaja görə deyil, buna görə şaxələnin. |
| `code()` | API-nin xəta kodu (məsələn `Dna.DomainService:Domain:10007`) və ya aşağıdakı SDK kodlarından biri. Ən çox rast gəlinənlər `domainnameapi::codes` içində sabitlərdir. |
| `message()` | Nə baş verdi. Şablon mesajlar API-nin təfərrüatları ilə əvəz olunur. |
| `status()` | HTTP statusu, cavab gəlmədikdə isə `None`. |
| `details()`, `validation_errors()` | Əlavə təfərrüat və sahə səviyyəsində problemlər (API-dən və ya göndərmədən əvvəlki yoxlamalardan). |
| `correlation_id()` | Gateway-in `X-Correlation-Id` dəyəri. Dəstəyə müraciət edəndə onu göstərin. |
| `hint()` | Növbəti addım üçün qısa təklif. |
| `is_retryable()` | Eyni sorğunu sonra təkrarlamağın uğurlu ola bilib-bilməməsi. |
| `is_outcome_unknown()` | Sorğu icra olunmuş ola bilər: dəyişiklik şəbəkədə uğursuz oldu və ya API reyestrin onu artıq icra etmiş ola biləcəyini bildirdi. |
| `is_not_found()`, `is_insufficient_balance()` | Tez-tez rast gəlinən hallar üçün qısayollar. |
| `operation()`, `safety()` | Uğursuz sorğu (`POST /api/v1/domains/renew`) və onun təhlükəsizlik sinfi. |
| `retry_after()`, `response_body()` | Gateway-in `Retry-After` dəyəri və xam cavab. |

`Error` `std::error::Error` və `Display` (`[CODE] message (HTTP 400); field: problem`) trait-lərini reallaşdırır, ona
görə `?`, `anyhow` və `thiserror` ilə işləyir.

| `kind()` | Nə vaxt |
|---|---|
| `Auth` | Yanlış və ya çatışmayan giriş məlumatları, ya da bu mühitdə naməlum və ya dayandırılmış reseller (401, yönləndirmə). |
| `Forbidden` | Autentifikasiya olunub, amma icazə yoxdur (403). |
| `NotFound` | Domen, əlaqə, sertifikat, sifariş və ya digər resurs mövcud deyil (404). |
| `Validation` | API girişi rədd etdi və ya biznes qaydası pozuldu: yanlış sahə, domen kilidlidir, balans çatmır (400, 422). |
| `Conflict` | Resursun vəziyyəti əməliyyata icazə vermir (409). |
| `RateLimited` | Avtomatik təkrarlardan sonra da sorğu limiti aşılıb (429). |
| `Server` | Server tərəfində xəta (5xx). Çox vaxt müvəqqətidir. |
| `Network`, `Timeout` | Cavab yoxdur: DNS, TLS, proxy, bağlantının kəsilməsi və ya timeout daxilində cavab gəlməməsi. |
| `InvalidInput` | Sorğu yoxlamalardan keçmədi və ya klient parametri yanlışdır; heç nə göndərilmədi. |
| `ReadOnly` | Yalnız oxuma rejimi tərəfindən bloklandı; heç nə göndərilmədi. |
| `Unknown` | Qalan hər şey, o cümlədən SDK-nın oxuya bilmədiyi cavab. |

`ErrorKind::as_str()` CLI-dəki adları verir (`auth`, `not_found`, `rate_limited`, ...). SDK-nın özünün təyin etdiyi kodlar:

| Kod | Mənası |
|---|---|
| `INVALID_RESELLER` | Gateway öz dayandırma səhifəsinə yönləndirdi: reseller ID **bu mühitdə** naməlum, qeyri-aktiv və ya dayandırılmışdır. |
| `INVALID_API_KEY` | Açar reseller ilə uyğun gəlmir. Canlı mühit və OTE açarları fərqlidir. |
| `MISSING_API_KEY`, `MISSING_RESELLER_ID`, `NO_CREDENTIALS` | Giriş məlumatlarından biri boşdur. |
| `SUBRESELLER_NOT_ALLOWED` | Alt reseller hesabları API-dən istifadə edə bilməz. Əsas reseller-in giriş məlumatlarından istifadə edin. |
| `UNAUTHORIZED` | Digər bütün 401 cavabları. |
| `RATE_LIMITED` | Təkrarlardan sonra 429. |
| `NETWORK`, `TIMEOUT` | Cavab gəlmədi. |
| `READ_ONLY` | Yalnız oxuma rejimi tərəfindən bloklandı. |
| `INVALID_INPUT`, `INVALID_CONFIGURATION` | Sorğu yoxlamalardan keçmədi və ya klient parametri yanlışdır. |
| `EMPTY_RESPONSE`, `INVALID_RESPONSE` | Uğurlu cavab boş idi və ya oxunmadı. |
| `HTTP_<status>` | API xəta kodu göndərmədi. |

## Təhlükəsizlik, təkrarlar və sorğu limitləri

Hər sorğunun təhlükəsizlik sinfi var, `dna` CLI-nin istifadə etdiyi ilə eynidir:

| `OperationSafety` | Mənası | Nümunələr |
|---|---|---|
| `Read` | Yan təsiri yoxdur, API POST və ya PUT istifadə etsə belə | axtarış, məlumat, siyahılar, yenilənmə və transfer yoxlamaları, CSR yaradılması, sertifikat statusu |
| `Write` | Geri qaytarıla bilən pulsuz dəyişiklik | DNS qeydləri, kilid, məxfilik, nameserver-lər, əlaqələr |
| `Billable` | Balansınızdan ödəniş alır | qeydiyyat, yenilənmə, transfer, bərpa, SSL alışı və yenilənməsi, hostinq sifarişi |
| `Destructive` | Nəyisə silir və ya geri qaytarmaq çətindir | DNS qeydinin silinməsi, DNSSEC-in götürülməsi, sertifikatın ləğvi |

- **Yalnız oxuma rejimi** (`read_only(true)` və ya `DNA_READ_ONLY=true`) `Read` olmayan hər şeyi göndərilməzdən əvvəl
  bloklayır. Onu araşdırma zamanı, hesabatlarda və dashboard-larda istifadə edin.
- **Təkrarlar.** 429 cavabları həmişə təkrarlanır, `Retry-After` nəzərə alınır. Bu, ödənişli sorğular üçün də
  təhlükəsizdir, çünki gateway onları heç nə icra olunmamış rədd edir. Şəbəkə xətaları, timeout-lar və 502/503/504
  cavabları yalnız oxuma sorğuları və CLI-nin idempotent kimi işarələdiyi yazma sorğuları (kilid, kilidin açılması,
  məxfilik, nameserver-lər, DNS yenilənməsi və tətbiqi) üçün təkrarlanır. Qeydiyyat və ya yenilənmə heç vaxt iki dəfə göndərilmir.
- **Templəmə.** Gateway hər açar üçün saniyədə 5 sorğuya, domen axtarışları üçün isə saniyədə 2 sorğuya icazə verir.
  Klient bu limitlərin altında qalmaq üçün sorğular arasında fasilə qoyur və DNS zona çağırışlarını 1 saniyə aralı
  göndərir. Domen məlumatı və TLD siyahısı üçün sorğu limiti yoxdur.
- **Yumşaq kvota.** Axtarışlar və məlumat sorğuları 1 xal, qalan hər şey 10 xal çəkir. 200 xaldan sonra gateway hər
  sorğunu 5 saniyəyə qədər ləngidir; xallar dəqiqədə 20% azalır. `send_with_response` vəziyyəti `quota` içində göstərir.

## Asinxron və bloklayan kod

SDK asinxrondur və Tokio runtime tələb edir (`#[tokio::main]`-in başlatdığı və ya serverinizin runtime-ı). Future-lar
`Send`-dir, ona görə `tokio::spawn` içində, axum və actix handler-lərində işləyir. Siyahıları səhifə-səhifə (`list`)
və ya elementlərdən ibarət `Stream` kimi (`list_all`) oxumaq olar; bu axın növbəti səhifəni yalnız lazım olduqda yükləyir:

```rust
use futures_util::TryStreamExt;

let expiring: Vec<Domain> = client
    .domains()
    .list_all(DomainListRequest::new())
    .try_filter(|d| std::future::ready(d.remaining_days < 30))
    .try_collect()
    .await?;
```

Bloklayan API yoxdur: o, az fayda üçün interfeysi iki dəfə böyüdərdi. Sinxron kodda çağırışları runtime üzərində
icra edin:

```rust
let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
let account = runtime.block_on(client.account().get())?;
```

## Kodunuzu test etmək

SDK-ya yalnız HTTP endpoint lazımdır. SDK-dan istifadə edən kodu API-ni çağırmadan test etmək üçün `base_url`-i lokal
serverə (məsələn `wiremock`, `httpmock` və ya `axum` ilə qurulmuş serverə) yönəldin və sorğu tempini söndürün:

```rust
let client = Client::builder()
    .credentials("test", "test")
    .base_url(format!("http://{}", server_address))
    .pace_requests(false)
    .build()?;
```

Modellər `Default` və `Deserialize` trait-lərini reallaşdırır, ona görə test məlumatları JSON-dan gələ bilər; mock-larınız
üçün xətaları `Error::new(kind, code, message)` ilə yaratmaq olar. Real API-yə qarşı testlər üçün öz açarı olan OTE-dən istifadə edin.

## Layihənin strukturu

```
src/
├── lib.rs           Crate sənədləşməsi və yenidən ixraclar (re-export)
├── client.rs        Client, ClientBuilder, Environment: göndərmə, təkrarlar, sorğu tempi, xətaların parsinqi, list_all
├── request.rs       Request, PagedRequest, Endpoint, OperationSafety, Query, JsonBody, Validator, Page, Empty
├── error.rs         Error, ErrorKind, codes
├── de.rs            Tolerant dekodlaşdırma və modelləri ilə enum-ları elan edən makrolar (daxili)
├── account.rs       AccountClient     Balanslar, profil və əməliyyatlar
├── contacts.rs      ContactsClient    Əlaqə handle-ları
├── domains/         DomainsClient     Domenlər, həmçinin forwarding(), child_name_servers() və documents()
├── dns.rs           DnsClient         Zona qeydləri
├── dnssec.rs        DnssecClient      DS qeydləri
├── ssl.rs           SslClient         Sertifikatlar
├── hosting.rs       HostingClient     Hostinq paketləri və xidmətləri
├── orders.rs        OrdersClient      Sifarişlər
└── products.rs      ProductsClient    TLD-lər, SSL məhsulları və qiymətlər
tests/               Saxta API testləri, spesifikasiya əhatə testi və testdata/openapi.json
examples/            İşə salına bilən proqramlar
```

Hər sahə modulu eyni formadadır: cavablarının modelləri ilə enum-ları, hər əməliyyat üçün bir sorğu tipi (`XxxRequest`)
və bir `XxxClient`.

## SDK-nın inkişafı

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                    # unit, saxta API, spesifikasiya və doc testləri; heç biri real API-ni çağırmır
cargo build --examples
```

Lokal toolchain yoxdursa, Docker istifadə edin:

```bash
docker run --rm -v "$PWD":/src -w /src rust:1.83 sh -c 'rustup component add rustfmt clippy; cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo test'
```

Testlər heç vaxt real API-ni çağırmır: 127.0.0.1 üzərində kiçik HTTP server istifadə edirlər (`tests/common`). CI onları
dəstəklənən minimal Rust versiyasında (1.83) və ən son stabil versiyada işə salır.

`tests/testdata/openapi.json` `dna` CLI-nin daxilinə yerləşdirdiyi API sənədidir. `tests/spec.rs` yoxlayır ki, ondakı
hər əməliyyatın CLI-nin metodu, yolu, təhlükəsizlik sinfi və idempotentliyi ilə tam bir sorğu tipi var, hər sorğu yalnız
sənəddə təsvir olunan query parametrlərini və gövdə xassələrini göndərir (iç-içə obyektlər daxil, məcburi olanlar mövcuddur)
və cavab tipləri sənəddəki adlardan istifadə edir. API yeni əməliyyat qazandıqda sənədi yeniləyin; sorğu tipi yaradılana
qədər test uğursuz olacaq.

## 1.x versiyasından keçid

2.0 yeni SDK-dır. 1.x PHP kitabxanasının port-u idi: tipsiz `serde_json::Value` zərfləri qaytaran bloklayan
`DomainNameApi` klienti. 2.0 asinxrondur, tipli sorğular və modellər, `Result<T, Error>` və builder ilə.

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
| `result["result"] == "ERROR"` | `e.kind()` və `e.code()` ilə `Err(e)` |

1.x-in transfer təsdiqi/ləğvi/rəddi və `sync_from_registry` çağırışları REST API sənədinin bir hissəsi deyil və 2.0-da
qarşılığı yoxdur. MSRV 1.83 olaraq qalır; `ureq` `reqwest` ilə əvəz olunub, ona görə 2.0 Tokio runtime tələb edir.
1.x xəta hesabatlarını Sentry-yə göndərirdi; 2.0 API-dən başqa heç yerə heç nə göndərmir.

## Keçidlər

- [`dna` CLI və MCP serveri](https://github.com/domainreseller/cli): eyni API terminaldan və AI agentlərindən.
- [API sənədləri](https://www.domainnameapi.com/docs/)
- [Dəyişikliklər jurnalı](CHANGELOG.md)

## Lisenziya

Apache License 2.0. [LICENSE](LICENSE) faylına baxın.
