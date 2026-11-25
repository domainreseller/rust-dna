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

[Domain Name API](https://www.domainnameapi.com) bayi platformunun resmi Rust SDK'sı. REST API'nin 67 işleminin
tamamını tipli istekler ve modellerle kapsar: alan adları, iletişim bilgileri, DNS, DNSSEC, SSL sertifikaları,
hosting, siparişler, ürünler ve hesabınız.

```rust
let client = Client::builder().credentials(reseller_id, api_key).environment(Environment::Ote).build()?;

let search = client.domains().search("example.com").await?;
if search.info.is_available() {
    let request = DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact), ["tr.apiname.com", "eu.apiname.com"]);
    client.domains().register_with_contacts(request).await?;
}
```

- **Her şey tipli.** Her işlem bir istek tipi, her yanıt dokümantasyonu olan bir struct'tır. Map yok, JSON ayrıştırma yok.
- **Kendi istekleriniz.** Her istek `Request` trait'ini uygular. SDK'nın henüz kapsamadığı bir uç nokta için bu trait'i kendiniz uygulayın.
- **Para harcayan işlemlerde güvenli.** Her istek, okuma mı yaptığını, yazma mı yaptığını, bakiyenizden ücret mi aldığını yoksa bir şey mi sildiğini bildirir. Ücretli istekler asla yeniden denenmez, salt-okunur mod ise her değişikliği engeller.
- **Yerleşik dayanıklılık.** Hız sınırlarına otomatik uyulur, 429 yanıtları yeniden denenir, okumalarda ağ hataları yeniden denenir.
- **Net hatalar.** Sabit bir türü, API'nin hata kodunu, alan düzeyinde doğrulama hatalarını, correlation ID'yi ve bir ipucunu taşıyan tek bir `Error` tipi.
- **Modern Rust.** Tokio üzerinde async, rustls ile `reqwest` (OpenSSL yok), serde modelleri, `Stream` olarak tembel (lazy) sayfalama, klonlanabilir `Send + Sync` bir istemci, `#![deny(missing_docs)]`.
- **Gizlilik.** Telemetri yok. API'ye gönderdiğiniz istekler dışında sürecinizden hiçbir şey dışarı çıkmaz.

[Domain Name API CLI'si `dna`](https://github.com/domainreseller/cli)'nın Rust karşılığıdır: aynı API
dokümanı, aynı işlemler ve gruplar, aynı güvenlik sınıfları, yeniden deneme kuralları ve hata kodları.

## İçindekiler

- [Kurulum](#kurulum)
- [Kimlik bilgileri](#kimlik-bilgileri)
- [Hızlı başlangıç](#hızlı-başlangıç)
- [Yapılandırma](#yapılandırma)
- [Kullanım](#kullanım): [hesap](#hesap) · [alan adları](#alan-adları) · [kişi kayıtları](#kişi-kayıtları) · [DNS](#dns) · [DNSSEC](#dnssec) · [SSL](#ssl) · [hosting](#hosting) · [siparişler ve ürünler](#siparişler-ve-ürünler)
- [`Request` trait'i ve istekler](#request-traiti-ve-istekler)
- [Hatalar](#hatalar)
- [Güvenlik, yeniden denemeler ve hız sınırları](#güvenlik-yeniden-denemeler-ve-hız-sınırları)
- [Asenkron ve bloklayan kod](#asenkron-ve-bloklayan-kod)
- [Kodunuzu test etme](#kodunuzu-test-etme)
- [Proje yapısı](#proje-yapısı)
- [Geliştirme](#geliştirme)
- [1.x'ten yükseltme](#1xten-yükseltme)

## Kurulum

> [!IMPORTANT]
> **Her zaman [en son sürümü](https://github.com/domainreseller/rust-dna/releases/latest) kullanın.** Yayınlanan paketler sürüm numaralıdır ve test edilmiştir.
> Yeşil **Code → Download ZIP** düğmesi ise geliştirme dalını indirir.

```toml
[dependencies]
domainnameapi = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"   # list_all akışlarını okumak için
```

Crate crates.io'da yayınlanana kadar onu sürüm etiketinden alın:

```toml
domainnameapi = { git = "https://github.com/domainreseller/rust-dna", tag = "v2.0.0" }
```

**Gereksinimler:** Rust 1.83 veya üstü ve bir Tokio çalışma zamanı (runtime). Bağımlılıklar şunlardır: `reqwest` (rustls ile,
OpenSSL yok), `serde`/`serde_json`, `tokio` (yalnızca zamanlayıcılar), `futures-util`, `chrono`, `base64` ve `httpdate`.

## Kimlik bilgileri

API her isteği iki değerle doğrular; SDK bunları sizin yerinize gönderir:

| Değer | Başlık | Nerede bulunur |
|---|---|---|
| Reseller ID (bir UUID) | `__reseller` | Bayi paneli → **API ayarları** |
| API anahtarı | `X-API-KEY` | Bayi paneli → **API ayarları** |

> [!NOTE]
> Bunlar **API kimlik bilgileridir**: panel girişiniz burada çalışmaz. **Canlı ortam ve OTE (test ortamı) farklı
> API anahtarları kullanır.** Birini diğerinde kullanmak `INVALID_API_KEY` veya `INVALID_RESELLER` hatasıyla sonuçlanır.

## Hızlı başlangıç

```rust
use domainnameapi::contacts::Contact;
use domainnameapi::domains::{DomainContacts, DomainListRequest, DomainRegisterWithContactsRequest};
use domainnameapi::{Client, Environment};
use futures_util::TryStreamExt;

#[tokio::main]
async fn main() -> Result<(), domainnameapi::Error> {
    // OTE test ortamıdır: gerçek API, kendi verileri, gerçek para yok.
    let client = Client::builder()
        .credentials("your-reseller-id", "your-api-key")
        .environment(Environment::Ote)
        .build()?;

    // 1. Kimlik bilgilerini ve bakiyeyi kontrol edin.
    let account = client.account().get().await?;
    println!("{}: {} USD", account.reseller_name, account.usd_balance);

    // 2. Alan adı müsait mi, fiyatı ne?
    let search = client.domains().search("my-new-domain.com").await?;
    println!("{} {} {}", search.info.status, search.info.price, search.info.currency);

    // 3. Dört rolün tamamı için tek bir iletişim kişisiyle kaydedin. Bu işlem bakiyeden ücret alır.
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

    // 4. Tüm alan adlarını listeleyin; sayfalar akış okundukça getirilir.
    let mut domains = client.domains().list_all(DomainListRequest::new());
    while let Some(domain) = domains.try_next().await? {
        println!("{} expires in {} days", domain.domain_name, domain.remaining_days);
    }
    Ok(())
}
```

[examples](examples) klasöründe bu senaryolar için çalıştırılabilir programlar vardır (`quickstart`, `register`, `dns`, `ssl`,
`custom_request`, `error_handling`). `DNA_*` ortam değişkenlerini okurlar; bu yüzden onları OTE'ye yönlendirin:

```bash
export DNA_RESELLER_ID=... DNA_API_KEY=... DNA_ENV=ote
cargo run --example quickstart -- example.com
```

## Yapılandırma

Bir istemci oluşturmanın üç yolu vardır:

```rust
// Canlı ortam, varsayılan ayarlar
let client = Client::new(reseller_id, api_key)?;

// Tüm ayarlar
let client = Client::builder()
    .credentials(reseller_id, api_key)
    .environment(Environment::Ote)
    .timeout(Duration::from_secs(30))
    .read_only(true)
    .build()?;

// dna CLI'nin okuduğu ortam değişkenlerinin aynısı ...
let client = Client::from_env()?;

// ... ve ayrıca kendi ayarlarınız
let client = ClientBuilder::from_env()?.http_client(my_reqwest_client).build()?;
```

| `ClientBuilder` metodu | Varsayılan | Anlamı |
|---|---|---|
| `credentials(reseller_id, api_key)` | (zorunlu) | Kimlik bilgileri. Biri boşsa `build()`, `NO_CREDENTIALS` hatasıyla başarısız olur. |
| `environment(...)` | `Production` | `Production` (`https://api.domainresellerapi.com`) veya `Ote` (`https://ote.domainresellerapi.com`). |
| `base_url(...)` | yok | Ortamın adresini geçersiz kılar, örneğin bir proxy için. Yol öneki korunur. |
| `timeout(...)` | 60 sn | Tek bir denemenin zaman sınırı. |
| `max_retries(...)` | 2 | Başarısız bir isteğin kaç kez tekrarlanacağı ([yeniden denemeler](#güvenlik-yeniden-denemeler-ve-hız-sınırları) bölümüne bakın). 0, yeniden denemeyi kapatır. |
| `max_retry_delay(...)` | 30 sn | Bir yeniden denemeden önceki en uzun bekleme. |
| `pace_requests(...)` | `true` | İstekleri, geçidin hız sınırlarının altında kalacak şekilde aralıklar. |
| `read_only(...)` | `false` | Okuma olmayan her isteği, gönderilmeden önce engeller. |
| `user_agent(...)` | `DomainNameAPI-Rust/<version>` | `User-Agent` başlığı. |
| `app(name, version)` | `rust`, SDK sürümü | `X-App` ve `X-Version` başlıkları: SDK'yı kullanan uygulama (aşağıya bakın). |
| `http_client(...)` | SDK'nın kendi istemcisi | Kendi `reqwest::Client` nesneniz (proxy, TLS ayarları). |

`from_env()` şunları okur:

| Değişken | Anlamı |
|---|---|
| `DNA_RESELLER_ID`, `DNA_API_KEY` | Kimlik bilgileri. |
| `DNA_ENV` | `production` veya `ote` (ayrıca `prod`, `live`, `test`, `sandbox`). |
| `DNA_TEST_MODE` | `true`, `1`, `on` veya `yes` OTE'yi seçer. `DNA_ENV` ayarlı değilse kullanılır. |
| `DNA_BASE_URL` | İsteğe bağlı temel adres geçersiz kılma. |
| `DNA_READ_ONLY` | `true`, salt-okunur modu açar. |
| `DNA_APP`, `DNA_APP_VERSION` | SDK'yı kullanan uygulama ve sürümü; `X-App` ve `X-Version` olarak gönderilir. |

`Client`, `Clone + Send + Sync`'tir ve klonlanması ucuzdur (bir `Arc` tutar). Her kimlik bilgisi kümesi için bir tane
oluşturun ve uygulamanın ömrü boyunca paylaşın: hız ayarlama istemci başına yapılır.

### Uygulamanızı tanıtma

Her istek, API ekibine isteği hangi entegrasyonun gönderdiğini bildiren `X-App` ve `X-Version` başlıklarını taşır.
Varsayılan değerler `rust` ve SDK sürümüdür. SDK üzerine bir panel modülü ya da başka bir ürün geliştiriyorsanız kendi
değerlerinizi verin:

```rust
let client = Client::builder()
    .credentials(reseller_id, api_key)
    .app("WiseCP5", "V2.0.20-beta") // X-App: wisecp5, X-Version: 2.0.20 olarak gönderilir
    .build()?;
```

Ad küçük harfe çevrilir ve yalnızca `a-z`, `0-9`, `.`, `_` ve `-` karakterleri kalır; sürümde yalnızca rakamlar ve
noktalar kalır. Boş bir ad `rust` değerine, boş bir sürüm SDK sürümüne döner. `from_env()` aynı değerleri `DNA_APP` ve
`DNA_APP_VERSION` değişkenlerinden okur. `User-Agent` başlığı değişmez.

## Kullanım

API'nin her alanının, CLI'nin komut gruplarıyla aynı adı taşıyan kendi istemcisi vardır: `client.account()`,
`client.contacts()`, `client.domains()` (`forwarding()`, `child_name_servers()` ve `documents()` ile birlikte),
`client.dns()`, `client.dnssec()`, `client.ssl()`, `client.hosting()`, `client.orders()` ve `client.products()`.
Zorunlu girdiler argüman olarak verilir; isteğe bağlı girdileri olan işlemler `new(...)` ile oluşturulan ve zincirleme
setter'larla ayarlanan bir istek alır. Bakiyenizden ücret alan metotlar bunu dokümantasyonlarında belirtir.

### Hesap

```rust
let account = client.account().get().await?;          // reseller_id, reseller_name, try_balance, usd_balance
let profile = client.account().profile().await?;      // name, company_name, account_currency, status, ...

// Bu ayın ücretleri, tüm sayfalar boyunca
let request = AccountTransactionsRequest::new()
    .charge_type(ChargeType::Subtract)
    .start_date("2026-11-01T00:00:00Z".parse()?)
    .transaction_types([TransactionType::Register, TransactionType::Renew]);
let mut transactions = client.account().all_transactions(request);
while let Some(t) = transactions.try_next().await? {
    println!("{:?} {} {} {}", t.creation_time, t.transaction_type, t.amount, t.amount_currency);
}
```

### Alan adları

**Arama**

```rust
let one = client.domains().search("example.com").await?;
let available = one.success && one.info.is_available();

let many = client.domains().bulk_search(["example.com", "example.net", "example.com.tr"]).await?;
for domain in many.available() {
    println!("{}: {} {}", domain.domain_name, domain.price, domain.currency);
}

// Para birimi ile
client.domains().search_with(DomainSearchRequest::new("example.com").currency("USD")).await?;
```

**Listeleme ve ayrıntılar**

```rust
let page = client.domains().list(DomainListRequest::new().limit(50).statuses([DomainStatus::Active])).await?;

let all: Vec<Domain> = client.domains().list_all(DomainListRequest::new().domain_name("example")).try_collect().await?;

let info = client.domains().info("example.com").await?;
println!("{} {:?}", info.auth_code, info.expiration_date);
if let Some(registrant) = info.contact(ContactType::Registrant) {
    println!("{}", registrant.email);
}
```

**Kayıt, yenileme, transfer, geri yükleme** (bunlar bakiyenizden ücret alır)

```rust
// Dört rolün tamamı için tek bir iletişim kişisi, iki yıl
let request = DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact.clone()), ["tr.apiname.com", "eu.apiname.com"])
    .years(2);
client.domains().register_with_contacts(request).await?;

// Her rol için farklı iletişim kişileri, kayıt kuruluşu öznitelikleri, trustee hizmeti
let request = DomainRegisterWithContactsRequest::new(
    "example.com.tr",
    DomainContacts { registrant: Some(owner), admin: Some(staff.clone()), tech: Some(staff.clone()), billing: Some(staff) },
    ["tr.apiname.com", "eu.apiname.com"],
)
.tld_attribute("<key>", "<value>"); // anahtarlar: client.products().tld("com.tr").await?.unwrap().attributes

// Mevcut iletişim kişisi tanımlayıcılarıyla (handle)
let request = DomainRegisterRequest::new("example.com", ContactHandles::same("DNA-12345"), ["tr.apiname.com", "eu.apiname.com"]);
client.domains().register(request).await?;

let renewal = client.domains().renew_check("example.com", Some(1)).await?;   // ücretsiz
if renewal.renewable {
    client.domains().renew("example.com", 1).await?;
}

let check = client.domains().transfer_check("example.com", "EPP-CODE").await?;  // ücretsiz
if check.transferable {
    client.domains().transfer(DomainTransferRequest::new("example.com", "EPP-CODE")).await?;
}

client.domains().restore("example.com").await?;
```

İstekler gönderilmeden önce denetlenir. Dört iletişim rolünün tamamını içermeyen bir kayıt, e-posta adresi olmayan bir
iletişim kişisi veya 0 yıllık bir süre, her sorunu listeleyen bir `ErrorKind::InvalidInput` hatası döndürür ve API'ye
hiçbir şey ulaşmaz.

> [!NOTE]
> API, bir kayıt, yenileme veya transfer isteğini HTTP 200 ve `"success": false` ile yanıtlayabilir (örneğin kayıt
> kuruluşu alan adını reddettiğinde). SDK, Java SDK'sı gibi bu yanıtı olduğu gibi döndürür:
> `DomainOperationResult::success` ve `message` değerlerini kontrol edin. Aynısı `SslOperationResult` ve `ContactCreateResult` için de geçerlidir.

**Ayarlar**

```rust
client.domains().lock("example.com").await?;
client.domains().unlock("example.com").await?;
client.domains().set_privacy("example.com", true).await?;
client.domains()
    .set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net", "ns2.example.net"]))
    .await?;
client.domains()   // DNSSEC ile imzalanmış bir alan adı
    .set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net"]).dnssec_mode(DnssecMode::DisableDnssec))
    .await?;
let auth_code = client.domains().update_auth_code("example.com").await?;

client.domains().update_contacts("example.com", DomainContacts::same(contact)).await?;
client.domains().set_contact_handles("example.com", ContactHandles { tech: "DNA-12345".into(), ..Default::default() }).await?;
```

**URL yönlendirme, child nameserver'lar ve belgeler**

```rust
client.domains().forwarding().set("example.com", "https://www.example.net", ForwardType::Standard).await?;
let forwarding = client.domains().forwarding().get("example.com").await?;   // ayarlı değilse None
client.domains().forwarding().delete("example.com").await?;

// Her adresin IP sürümü otomatik algılanır
client.domains().child_name_servers().add("example.com", "ns1.example.com", ["192.0.2.10", "2001:db8::10"]).await?;
client.domains().child_name_servers()
    .update(DomainHostUpdateRequest::new("example.com", "ns1.example.com", ["192.0.2.20"]))
    .await?;
client.domains().child_name_servers().delete("example.com", "ns1.example.com").await?;

// Kayıt kuruluşu belgeleri (çoğunlukla .tr)
let document = DocumentUpload::from_file(DocumentType::IdentityCard, "id.pdf")?;
client.domains().documents().upload("example.com.tr", [document]).await?;
let documents: Vec<Document> = client.domains().documents().list_all("example.com.tr").try_collect().await?;
```

### Kişi kayıtları

İletişim kişisi tanımlayıcıları (handle), yeniden kullanılabilir iletişim kayıtlarıdır. Bir kez oluşturun, birçok alan adında kullanın.

```rust
let created = client.contacts().create(contact.clone()).await?;
let handle = created.handle;                                         // örneğin DNA-12345

let details = client.contacts().get(&handle).await?;                 // iletişim kişisi ve doğrulama durumu
let result = client.contacts().update(&handle, contact).await?;
println!("{} of {} domains updated", result.stats.success_count, result.stats.total_domain_count);

let verifications = client.contacts().verification(&handle).await?;
client.contacts().resend_verification(&handle).await?;
```

### DNS

Platformun nameserver'larındaki alan adlarının bölge (zone) kayıtları. Alan adı bu nameserver'ları kullanmaya başlayınca
kayıtlar herkese açık olarak çözümlenir; bunu `apply` sağlar.

```rust
for set in client.dns().list("example.com").await? {
    println!("{} {} {} {:?}", set.name, set.record_type, set.ttl, set.values());
}

let dns = client.dns();
dns.add(DnsAddRequest::new("example.com", "www", RecordType::A, ["192.0.2.10"])).await?;
dns.add(DnsAddRequest::new("example.com", "@", "txt", ["v=spf1 include:_spf.example.net ~all"])).await?;   // "txt", TXT olur
dns.add(DnsAddRequest::new("example.com", "@", RecordType::Mx, ["10 mx1.example.net", "20 mx2.example.net"]).ttl(3600)).await?;
dns.update(DnsUpdateRequest::new("example.com", "www", RecordType::A, ["192.0.2.20"]).ttl(300)).await?;
dns.delete(DnsDeleteRequest::new("example.com", "www", RecordType::A, "192.0.2.20")).await?;

let applied = dns.apply("example.com").await?;
```

SDK, API'nin reddedeceği durumları önceden denetler: apex'te CNAME, önceliği olmayan bir MX, üç parçası eksik bir CAA
değeri, IPv4 olmayan bir A kaydı, 1–86400 dışında bir TTL. Bölge çağrıları, geçidin sınırı olan 1 saniye arayla yapılır.
OTE test ortamı bölge çağrılarını HTTP 500 ile yanıtlar; bu yüzden DNS kodu yalnızca canlı ortamda denenebilir.

### DNSSEC

```rust
let info = client.dnssec().get("example.com").await?;      // status, ds_records, allowed_algorithms, ...

let record: DsData = "12345 13 2 49FD46E6C4B45C55D4AC69CBD3CD34AC1AFE51DE".parse()?;
client.dnssec().add("example.com", [record.clone()]).await?;
client.dnssec().remove("example.com", record).await?;
client.dnssec().disable("example.com").await?;            // tüm DS kayıtlarını kaldırır
```

### SSL

Bir alan adının birden fazla sertifikası olabilir; bu nedenle mevcut bir sertifika hiçbir zaman alan adıyla değil,
**sertifika kimliği** (bir GUID) ile belirtilir. `initialize` ve `renew` bu kimliği döndürür: saklayın. `list`,
elinizdeki sertifikaların kimliklerini bulur.

```rust
// 1. Satın alma (bakiyeden ücret alır). Paket kodları client.products().ssls(...) üzerinden gelir.
let approvers = client.ssl().approvers(SslApproversRequest::for_purchase("example.com", "PKG-CODE")).await?;
let order = client.ssl().initialize("example.com", "PKG-CODE").await?;
let certificate_id = order.certificate_id;                  // saklayın: sonraki her çağrı bunu alır

// 2. Elinizde yoksa bir CSR ve özel anahtar
let csr = client.ssl().generate_csr(
    SslGenerateCsrRequest::new("example.com")
        .country("TR").state("Istanbul").locality("Istanbul").organization("Example Ltd").organization_unit("IT"),
).await?;

// 3. 15 gün içinde yapılandırın: CSR, doğrulama yöntemi ve iletişim kişisi
client.ssl().configure(
    SslConfigureRequest::new(&certificate_id, &csr.csr, certificate_contact.clone())
        .authentication_type(AuthenticationType::Email)
        .approver_mail(&approvers[0])
        .web_server_type(WebServerType::Apache),
).await?;

// 4. Sertifikanın düzenlenmesini bekleyin, ardından indirin
let status = client.ssl().status(&certificate_id).await?;
if status.is_issued {
    let content = client.ssl().content(&certificate_id).await?;
    std::fs::write("fullchain.pem", content.full_chain())?;
}

let details = client.ssl().get(&certificate_id).await?;     // abonelik, yapılandırma, iletişim kişileri
client.ssl().reissue(SslReissueRequest::new(&certificate_id).csr(new_csr).private_key(new_key)).await?;
client.ssl().revoke(&certificate_id).await?;                // geri alınamaz
```

**Sertifikaları listeleme ve kimlik bulma**

```rust
let page = client.ssl().list(SslListRequest::new().domain_name("example.com")).await?;   // en yenisi önce

let mut all = client.ssl().list_all(SslListRequest::new());
while let Some(c) = all.try_next().await? {
    println!("{} {} {} {:?}", c.certificate_id, c.domain_name, c.status, c.subscription_expires_at);
}

// Elinizdeki bir sertifikanın onaylayıcı adresleri
let mails = client.ssl().approvers(SslApproversRequest::for_certificate(&certificate_id)).await?;
```

**Yenileme** (bu işlem bakiyenizden ücret alır)

```rust
// details.subscription.expires_at tarihinden 30 gün öncesinden itibaren yapılabilir. Yenileme, kendi kimliği olan yeni bir sertifikadır.
let renewal = client.ssl().renew(&certificate_id).await?;
client.ssl().configure(
    SslConfigureRequest::new(&renewal.certificate_id, &new_csr, certificate_contact)
        .authentication_type(AuthenticationType::Email)
        .approver_mail(&approvers[0]),
).await?;
```

Satın alınan veya yenilenen bir sertifika 15 gün içinde yapılandırılmalıdır; aksi halde iptal edilir. Yenilemek için
önce `renew`, ardından yeni kimlikle `configure` çağırın. `initialize` ile yeniden satın almayın: bu, satın alma fiyatı
üzerinden ayrı bir satın alma olur ve eski sertifika hiçbir zaman yenilenmiş olarak işaretlenmez. Boş veya GUID olmayan
bir sertifika kimliği (örneğin eski kodun verdiği bir alan adı), hiçbir şey gönderilmeden önce reddedilir.

### Hosting

```rust
let packages = client.hosting().packages().await?;
let created = client.hosting().create("example.com", "starter-linux").await?;          // bakiyeden ücret alır
let status = client.hosting().status(HostingStatusRequest::by_service_id(&created.service_id)).await?;
if status.is_active() {
    println!("{} {}", status.panel_address, status.username);
}
let hosting = client.hosting().get(HostingGetRequest::by_domain("example.com")).await?;
let plans = client.hosting().plans(None).await?;
```

### Siparişler ve ürünler

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

## `Request` trait'i ve istekler

Her işlem, CLI komutunun adını taşıyan bir istek tipidir: `dna domain renew` için `DomainRenewRequest`,
`dna domain renew-check` için `DomainRenewCheckRequest`, `dna dns add` için `DnsAddRequest`. Alan metotları bunları
sizin yerinize oluşturur; isterseniz bunları kendiniz de oluşturup gönderebilirsiniz:

```rust
let request = DomainRenewRequest::new("example.com", 2);
let endpoint = request.endpoint();
println!("{endpoint} is {}", endpoint.safety);   // "POST /api/v1/domains/renew is billable"

let result = client.send(&request).await?;

// Aynısı, HTTP ayrıntılarıyla birlikte: correlation ID, yumuşak kota durumu, denemeler, süre
let response = client.send_with_response(&request).await?;
println!("{} quota {:?}", response.correlation_id, response.quota);
```

Rust'ta kalıtım yoktur; her isteğin temeli bir trait'tir:

| Trait | Ne için |
|---|---|
| `Request` | Her işlem. `type Response` yanıt tipidir; hiçbir şey döndürmeyen işlemler (kilitleme, kilit açma, DNS kaydı ekleme, ...) için `Empty`. |
| `PagedRequest<T>` | Liste işlemleri (`Response = Page<T>`). `Paging` değerine (skip, limit, sort) erişim sağlar ve `Client::list_all` ile çalışır. |

Bir istek, ne olduğunu `endpoint()` ile bildirir: metot, yol, güvenlik sınıfı ve idempotent olup olmadığı. Diğer üç
metodun varsayılanları vardır: `query()` sorgu dizesini döndürür (`Query` boş değerleri dışarıda bırakır), `body()` JSON
gövdesini döndürür (`JsonBody` bir gövde oluşturur ve boş değerleri dışarıda bırakır; ya da `serde_json::json!` kullanın),
`validate()` ise göndermeden önce sorunları toplar. SDK'nın henüz kapsamadığı bir uç noktayı çağırmak için kendi
isteğinizi yazın:

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
        Endpoint::get("/api/v1/domains/notes")          // GET bir okumadır
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

İsteğiniz, yerleşik isteklerin aldığı her şeyi alır: kimlik bilgileri, güvenlik sınıfına göre yeniden denemeler, hız
ayarlama, salt-okunur mod ve hata yönetimi. Bir değişiklik için `Endpoint::post(path, OperationSafety::Write)` (veya
`Billable`, `Destructive`) kullanın ve `.idempotent()` çağrısını yalnızca isteği iki kez göndermek zararsızsa ekleyin.

## Hatalar

Başarısız her istek bir `Error` döndürür; istek denetimlerden geçemediği ve hiçbir şey gönderilmediği durumda da.

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
        // Ücretli veya değişiklik yapan bir istek ağda başarısız oldu: API'ye ulaşmış olabilir.
        // Yeniden göndermeden önce durumu kontrol edin.
        let info = client.domains().info("example.com").await?;
    }
    Err(e) => return Err(e.into()),
}
```

| Metot | Anlamı |
|---|---|
| `kind()` | Sabit bir sınıflandırma (aşağıdaki tablo). Mesaja göre değil, buna göre dallanın. |
| `code()` | API'nin hata kodu (örneğin `Dna.DomainService:Domain:10007`) veya aşağıdaki SDK kodlarından biri. Yaygın olanlar `domainnameapi::codes` içinde sabit olarak bulunur. |
| `message()` | Ne olduğu. Kalıp mesajların yerine API'nin ayrıntıları konur. |
| `status()` | HTTP durumu; yanıt gelmediyse `None`. |
| `details()`, `validation_errors()` | Ek ayrıntılar ve alan düzeyindeki sorunlar (API'den veya göndermeden önceki denetimlerden). |
| `correlation_id()` | Geçidin `X-Correlation-Id` değeri. Destekle iletişime geçerken bunu belirtin. |
| `hint()` | Sonra ne yapılacağına dair kısa bir öneri. |
| `is_retryable()` | Aynı isteği daha sonra tekrarlamanın başarılı olup olamayacağı. |
| `is_outcome_unknown()` | İstek çalışmış olabilir: bir değişiklik ağda başarısız oldu veya API, kayıt kuruluşunun işlemi zaten yapmış olabileceğini bildirdi. |
| `is_not_found()`, `is_insufficient_balance()` | Yaygın durumlar için kısayollar. |
| `operation()`, `safety()` | Başarısız olan istek (`POST /api/v1/domains/renew`) ve onun güvenlik sınıfı. |
| `retry_after()`, `response_body()` | Geçidin `Retry-After` değeri ve ham yanıt. |

`Error`, `std::error::Error` ve `Display` (`[CODE] message (HTTP 400); field: problem`) trait'lerini uygular; bu yüzden
`?`, `anyhow` ve `thiserror` ile çalışır.

| `kind()` | Ne zaman |
|---|---|
| `Auth` | Yanlış veya eksik kimlik bilgileri ya da bu ortamda bilinmeyen veya askıya alınmış bir bayi (401, yönlendirme). |
| `Forbidden` | Kimlik doğrulandı ama izin yok (403). |
| `NotFound` | Alan adı, iletişim kişisi, sertifika, sipariş veya başka bir kaynak yok (404). |
| `Validation` | API girdiyi reddetti ya da bir iş kuralı başarısız oldu: hatalı alan, kilitli alan adı, yetersiz bakiye (400, 422). |
| `Conflict` | Kaynağın durumu işleme izin vermiyor (409). |
| `RateLimited` | Otomatik yeniden denemelerden sonra hâlâ hız sınırında (429). |
| `Server` | Sunucu tarafı hata (5xx). Çoğu zaman geçicidir. |
| `Network`, `Timeout` | Yanıt yok: DNS, TLS, proxy, bağlantı sıfırlanması veya zaman aşımı içinde yanıt gelmemesi. |
| `InvalidInput` | İstek denetimlerden geçemedi veya bir istemci ayarı geçersiz; hiçbir şey gönderilmedi. |
| `ReadOnly` | Salt-okunur mod tarafından engellendi; hiçbir şey gönderilmedi. |
| `Unknown` | SDK'nın okuyamadığı bir yanıt dahil, geri kalan her şey. |

`ErrorKind::as_str()`, CLI'deki adları verir (`auth`, `not_found`, `rate_limited`, ...). SDK'nın kendisinin belirlediği kodlar:

| Kod | Anlamı |
|---|---|
| `INVALID_RESELLER` | Geçit, askıya alma sayfasına yönlendirdi: reseller ID **bu ortamda** bilinmiyor, etkin değil veya askıya alınmış. |
| `INVALID_API_KEY` | Anahtar bayiyle eşleşmiyor. Canlı ortam ve OTE anahtarları farklıdır. |
| `MISSING_API_KEY`, `MISSING_RESELLER_ID`, `NO_CREDENTIALS` | Bir kimlik bilgisi boş. |
| `SUBRESELLER_NOT_ALLOWED` | Alt bayi hesapları API'yi kullanamaz. Ana bayinin kimlik bilgilerini kullanın. |
| `UNAUTHORIZED` | Diğer tüm 401 yanıtları. |
| `RATE_LIMITED` | Yeniden denemelerden sonra 429. |
| `NETWORK`, `TIMEOUT` | Yanıt gelmedi. |
| `READ_ONLY` | Salt-okunur mod tarafından engellendi. |
| `INVALID_INPUT`, `INVALID_CONFIGURATION` | Bir istek denetimlerden geçemedi veya bir istemci ayarı geçersiz. |
| `EMPTY_RESPONSE`, `INVALID_RESPONSE` | Başarılı bir yanıt boştu veya okunamadı. |
| `HTTP_<status>` | API hata kodu göndermedi. |

## Güvenlik, yeniden denemeler ve hız sınırları

Her isteğin bir güvenlik sınıfı vardır; `dna` CLI'nin kullandığıyla aynıdır:

| `OperationSafety` | Anlamı | Örnekler |
|---|---|---|
| `Read` | API POST veya PUT kullansa bile yan etkisi yok | arama, ayrıntılar, listeler, yenileme ve transfer denetimleri, CSR üretimi, sertifika durumu |
| `Write` | Geri alınabilen ücretsiz bir değişiklik | DNS kayıtları, kilit, gizlilik, nameserver'lar, iletişim bilgileri |
| `Billable` | Bakiyenizden ücret alır | kayıt, yenileme, transfer, geri yükleme, SSL satın alma ve yenileme, hosting siparişi |
| `Destructive` | Bir şeyi siler veya geri alması zordur | DNS kaydı silme, DNSSEC kaldırma, sertifika iptali |

- **Salt-okunur mod** (`read_only(true)` veya `DNA_READ_ONLY=true`), `Read` olmayan her şeyi gönderilmeden önce
  engeller. Keşif yaparken, raporlarda ve panolarda kullanın.
- **Yeniden denemeler.** 429 yanıtları her zaman, `Retry-After` dikkate alınarak yeniden denenir. Bu, ücretli istekler
  için bile güvenlidir, çünkü geçit bunları hiçbir şey çalışmadan önce reddeder. Ağ hataları, zaman aşımları ve
  502/503/504 yanıtları yalnızca okumalarda ve CLI'nin idempotent olarak işaretlediği yazmalarda (kilitleme, kilit açma,
  gizlilik, nameserver'lar, DNS güncelleme ve uygulama) yeniden denenir. Bir kayıt veya yenileme asla iki kez gönderilmez.
- **Hız ayarlama.** Geçit, anahtar başına saniyede 5 isteğe, alan adı aramalarında ise saniyede 2 isteğe izin verir.
  İstemci, istekleri bu sınırların altında kalacak şekilde aralıklar ve DNS bölge çağrılarını 1 saniye arayla yapar.
  Alan adı bilgisi ve TLD listesi hız sınırına tabi değildir.
- **Yumuşak kota.** Aramalar ve bilgi sorguları 1 puan, diğer her şey 10 puan harcar. 200 puanın üzerinde geçit her
  isteği 5 saniyeye kadar yavaşlatır; puanlar dakikada %20 azalır. `send_with_response`, durumu `quota` içinde gösterir.

## Asenkron ve bloklayan kod

SDK asenkrondur ve bir Tokio çalışma zamanına ihtiyaç duyar (`#[tokio::main]`'in başlattığı veya sunucunuzunki).
Future'lar `Send`'dir; bu yüzden `tokio::spawn` içinde, axum ve actix handler'larında çalışırlar. Listeler sayfa sayfa
(`list`) veya öğelerden oluşan bir `Stream` olarak (`list_all`) okunabilir; bu akış bir sonraki sayfayı yalnızca
gerektiğinde getirir:

```rust
use futures_util::TryStreamExt;

let expiring: Vec<Domain> = client
    .domains()
    .list_all(DomainListRequest::new())
    .try_filter(|d| std::future::ready(d.remaining_days < 30))
    .try_collect()
    .await?;
```

Bloklayan bir API yoktur: az bir kazanç için arayüzü iki katına çıkarırdı. Senkron kodda çağrıları bir çalışma zamanı
üzerinde çalıştırın:

```rust
let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
let account = runtime.block_on(client.account().get())?;
```

## Kodunuzu test etme

SDK'nın tek ihtiyacı bir HTTP uç noktasıdır. SDK'yı kullanan kodu API'yi çağırmadan test etmek için `base_url`'i yerel
bir sunucuya (örneğin `wiremock`, `httpmock` veya `axum` ile oluşturulmuş bir sunucuya) yönlendirin ve hız ayarlamayı kapatın:

```rust
let client = Client::builder()
    .credentials("test", "test")
    .base_url(format!("http://{}", server_address))
    .pace_requests(false)
    .build()?;
```

Modeller `Default` ve `Deserialize` trait'lerini uygular; bu yüzden test verileri JSON'dan gelebilir. Mock'larınız için
hatalar `Error::new(kind, code, message)` ile oluşturulabilir. Gerçek API'ye karşı testler için, kendi anahtarıyla OTE'yi kullanın.

## Proje yapısı

```
src/
├── lib.rs           Crate dokümantasyonu ve yeniden dışa aktarımlar
├── client.rs        Client, ClientBuilder, Environment: gönderme, yeniden denemeler, hız ayarlama, hata ayrıştırma, list_all
├── request.rs       Request, PagedRequest, Endpoint, OperationSafety, Query, JsonBody, Validator, Page, Empty
├── error.rs         Error, ErrorKind, codes
├── de.rs            Hoşgörülü çözümleme ve modelleri ile enum'ları tanımlayan makrolar (dahili)
├── account.rs       AccountClient     Bakiyeler, profil ve işlemler
├── contacts.rs      ContactsClient    İletişim kişisi tanımlayıcıları (handle)
├── domains/         DomainsClient     Alan adları, ayrıca forwarding(), child_name_servers() ve documents()
├── dns.rs           DnsClient         Bölge kayıtları
├── dnssec.rs        DnssecClient      DS kayıtları
├── ssl.rs           SslClient         Sertifikalar
├── hosting.rs       HostingClient     Hosting paketleri ve hizmetleri
├── orders.rs        OrdersClient      Siparişler
└── products.rs      ProductsClient    TLD'ler, SSL ürünleri ve fiyatlar
tests/               Sahte API testleri, spesifikasyon kapsam testi ve testdata/openapi.json
examples/            Çalıştırılabilir programlar
```

Her alan modülü aynı biçimdedir: yanıtlarının modelleri ile enum'ları, işlem başına bir istek tipi (`XxxRequest`) ve
bir `XxxClient`.

## Geliştirme

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                    # birim, sahte API, spesifikasyon ve doküman testleri; hiçbiri gerçek API'yi çağırmaz
cargo build --examples
```

Yerel bir araç zinciri (toolchain) yoksa Docker kullanın:

```bash
docker run --rm -v "$PWD":/src -w /src rust:1.83 sh -c 'rustup component add rustfmt clippy; cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo test'
```

Testler gerçek API'yi asla çağırmaz: 127.0.0.1 üzerinde küçük bir HTTP sunucusu kullanırlar (`tests/common`). CI,
testleri desteklenen en düşük Rust sürümünde (1.83) ve en son kararlı sürümde çalıştırır.

`tests/testdata/openapi.json`, `dna` CLI'nin içine gömdüğü API dokümanıdır. `tests/spec.rs`, içindeki her işlemin
CLI'nin metodu, yolu, güvenlik sınıfı ve idempotentliği ile tam olarak bir istek tipine sahip olduğunu, her isteğin
yalnızca dokümanda tanımlanan sorgu parametrelerini ve gövde özelliklerini gönderdiğini (iç içe nesneler dahil, zorunlu
olanlar mevcut olacak şekilde) ve yanıt tiplerinin dokümandaki adları kullandığını denetler. API'ye yeni bir işlem
eklendiğinde dokümanı güncelleyin; istek tipi yazılana kadar test başarısız olur.

## 1.x'ten yükseltme

2.0 yeni bir SDK'dır. 1.x, PHP kütüphanesinin bir uyarlamasıydı: tipsiz `serde_json::Value` zarfları döndüren, bloklayan
bir `DomainNameApi` istemcisi. 2.0 asenkrondur; tipli istekler ve modeller, `Result<T, Error>` ve bir builder içerir.

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
| `result["result"] == "ERROR"` | `e.kind()` ve `e.code()` ile `Err(e)` |

1.x'teki transfer onaylama/iptal/reddetme ve `sync_from_registry` çağrıları REST API dokümanında yer almaz ve 2.0'da
karşılıkları yoktur. MSRV 1.83 olarak kalır; `ureq` yerine `reqwest` kullanılır, bu yüzden 2.0 bir Tokio çalışma zamanı
gerektirir. 1.x, hata raporlarını Sentry'ye gönderiyordu; 2.0, API dışında hiçbir yere hiçbir şey göndermez.

## Bağlantılar

- [`dna` CLI ve MCP sunucusu](https://github.com/domainreseller/cli): aynı API, terminalden ve yapay zeka ajanlarından.
- [API dokümantasyonu](https://www.domainnameapi.com/docs/)
- [Değişiklik günlüğü](CHANGELOG.md)

## Lisans

Apache License 2.0. Bkz. [LICENSE](LICENSE).
