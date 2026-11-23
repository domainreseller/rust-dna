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

<div dir="rtl">

# حزمة DomainNameAPI لـ Rust (rust-dna)

</div>

<p>
  <a href="https://github.com/domainreseller/rust-dna/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/domainreseller/rust-dna/ci.yml?branch=main&label=CI&style=flat-square" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-0B8DB5?style=flat-square" alt="Apache-2.0"></a>
  <img src="https://img.shields.io/badge/Rust-1.83%2B-0B8DB5?style=flat-square" alt="Rust 1.83+">
</p>

<div dir="rtl">

حزمة SDK الرسمية لـ Rust الخاصة بمنصة الموزّعين [Domain Name API](https://www.domainnameapi.com). تغطي جميع عمليات REST API
البالغ عددها 67 عملية بطلبات ونماذج مكتوبة بأنواع صريحة: النطاقات، وجهات الاتصال، وDNS، وDNSSEC، وشهادات SSL،
والاستضافة، والطلبات، والمنتجات، وحسابك.

</div>

```rust
let client = Client::builder().credentials(reseller_id, api_key).environment(Environment::Ote).build()?;

let search = client.domains().search("example.com").await?;
if search.info.is_available() {
    let request = DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact), ["tr.apiname.com", "eu.apiname.com"]);
    client.domains().register_with_contacts(request).await?;
}
```

<div dir="rtl">

- **كل شيء مكتوب بأنواع صريحة.** كل عملية هي نوع طلب، وكل إجابة هي بنية (struct) مزوّدة بالتوثيق. لا خرائط (maps) ولا تحليل JSON.
- **طلباتك الخاصة.** كل طلب ينفّذ السمة (trait) `Request`. نفّذها لنقطة نهاية لا تغطيها الحزمة بعد.
- **آمنة مع الأموال.** يعلن كل طلب ما إذا كان يقرأ، أو يكتب، أو يخصم من رصيدك، أو يحذف شيئًا. لا يُعاد إرسال الطلبات المدفوعة أبدًا، ووضع القراءة فقط يمنع أي تغيير.
- **مرونة مدمجة.** تُراعى حدود معدل الطلبات تلقائيًا، وتُعاد محاولة إجابات 429، وتُعاد محاولة أخطاء الشبكة في عمليات القراءة.
- **أخطاء واضحة.** نوع `Error` واحد مع تصنيف ثابت، ورمز الخطأ من API، وأخطاء التحقق على مستوى الحقول، ومعرّف الارتباط (correlation ID) وتلميح.
- **Rust حديثة.** عمل غير متزامن (async) على Tokio، و`reqwest` مع rustls (بلا OpenSSL)، ونماذج serde، وترقيم صفحات كسول على هيئة `Stream`، وعميل `Send + Sync` قابل للنسخ (clone)، و`#![deny(missing_docs)]`.
- **خصوصية تامة.** لا قياس عن بُعد (telemetry). لا يغادر عمليتك شيء سوى الطلبات التي ترسلها إلى API.

وهي النظير في Rust لـ [`dna`، أداة CLI الخاصة بـ Domain Name API](https://github.com/domainreseller/cli): وثيقة API نفسها،
والعمليات والمجموعات نفسها، وفئات الأمان نفسها، وقواعد إعادة المحاولة ورموز الأخطاء نفسها.

## المحتويات

- [التثبيت](#التثبيت)
- [بيانات الاعتماد](#بيانات-الاعتماد)
- [البدء السريع](#البدء-السريع)
- [الإعداد](#الإعداد)
- [الاستخدام](#الاستخدام): [الحساب](#الحساب) · [النطاقات](#النطاقات) · [جهات الاتصال](#جهات-الاتصال) · [DNS](#dns) · [DNSSEC](#dnssec) · [SSL](#ssl) · [الاستضافة](#الاستضافة) · [الطلبات والمنتجات](#الطلبات-والمنتجات)
- [الطلبات والسمة `Request`](#الطلبات-والسمة-request)
- [الأخطاء](#الأخطاء)
- [الأمان وإعادة المحاولة وحدود معدل الطلبات](#الأمان-وإعادة-المحاولة-وحدود-معدل-الطلبات)
- [الشيفرة غير المتزامنة والشيفرة الحاجبة](#الشيفرة-غير-المتزامنة-والشيفرة-الحاجبة)
- [اختبار شيفرتك](#اختبار-شيفرتك)
- [بنية المشروع](#بنية-المشروع)
- [التطوير](#التطوير)
- [الترقية من 1.x](#الترقية-من-1x)

## التثبيت

> [!IMPORTANT]
> **استخدم دائمًا [أحدث إصدار](https://github.com/domainreseller/rust-dna/releases/latest).** حزم الإصدارات مرقّمة بإصدارات ومُختبرة.
> أما زر **Code → Download ZIP** الأخضر فينزّل فرع التطوير بدلًا منها.

</div>

```toml
[dependencies]
domainnameapi = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"   # لقراءة تدفقات list_all
```

<div dir="rtl">

إلى أن تُنشر الحزمة (crate) على crates.io، خذها من وسم (tag) الإصدار:

</div>

```toml
domainnameapi = { git = "https://github.com/domainreseller/rust-dna", tag = "v2.0.0" }
```

<div dir="rtl">

**المتطلبات:** Rust 1.83 أو أحدث، وبيئة تشغيل Tokio. التبعيات هي `reqwest` (مع rustls، بلا OpenSSL)،
و`serde`/`serde_json`، و`tokio` (للمؤقّتات فقط)، و`futures-util`، و`chrono`، و`base64` و`httpdate`.

## بيانات الاعتماد

تصادق API على كل طلب بقيمتين، وترسلهما الحزمة نيابةً عنك:

| القيمة | الترويسة (Header) | أين تجدها |
|---|---|---|
| Reseller ID (معرّف UUID) | `__reseller` | لوحة الموزّع ← **API settings** |
| API key | `X-API-KEY` | لوحة الموزّع ← **API settings** |

> [!NOTE]
> هذه **بيانات اعتماد API**: بيانات تسجيل الدخول إلى اللوحة لا تعمل هنا. **تستخدم بيئة الإنتاج وبيئة OTE (بيئة الاختبار)
> مفاتيح API مختلفة.** استخدام مفتاح إحداهما مع الأخرى يفشل بالخطأ `INVALID_API_KEY` أو `INVALID_RESELLER`.

## البدء السريع

</div>

```rust
use domainnameapi::contacts::Contact;
use domainnameapi::domains::{DomainContacts, DomainListRequest, DomainRegisterWithContactsRequest};
use domainnameapi::{Client, Environment};
use futures_util::TryStreamExt;

#[tokio::main]
async fn main() -> Result<(), domainnameapi::Error> {
    // OTE هي بيئة الاختبار: API حقيقية، وبيانات خاصة بها، ولا أموال حقيقية.
    let client = Client::builder()
        .credentials("your-reseller-id", "your-api-key")
        .environment(Environment::Ote)
        .build()?;

    // 1. تحقّق من بيانات الاعتماد والرصيد.
    let account = client.account().get().await?;
    println!("{}: {} USD", account.reseller_name, account.usd_balance);

    // 2. هل النطاق متاح، وبأي سعر؟
    let search = client.domains().search("my-new-domain.com").await?;
    println!("{} {} {}", search.info.status, search.info.price, search.info.currency);

    // 3. سجّله بجهة اتصال واحدة للأدوار الأربعة كلها. هذا يخصم من الرصيد.
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

    // 4. اعرض كل النطاقات؛ تُجلب الصفحات أثناء قراءة التدفق (stream).
    let mut domains = client.domains().list_all(DomainListRequest::new());
    while let Some(domain) = domains.try_next().await? {
        println!("{} expires in {} days", domain.domain_name, domain.remaining_days);
    }
    Ok(())
}
```

<div dir="rtl">

يحتوي مجلد [الأمثلة](examples) على برامج قابلة للتشغيل لهذه السيناريوهات (`quickstart`، و`register`، و`dns`، و`ssl`،
و`custom_request`، و`error_handling`). وهي تقرأ متغيرات البيئة `DNA_*`، لذا وجّهها إلى OTE:

</div>

```bash
export DNA_RESELLER_ID=... DNA_API_KEY=... DNA_ENV=ote
cargo run --example quickstart -- example.com
```

<div dir="rtl">

## الإعداد

هناك ثلاث طرق لإنشاء عميل:

</div>

```rust
// بيئة الإنتاج، بالإعدادات الافتراضية
let client = Client::new(reseller_id, api_key)?;

// كل الإعدادات
let client = Client::builder()
    .credentials(reseller_id, api_key)
    .environment(Environment::Ote)
    .timeout(Duration::from_secs(30))
    .read_only(true)
    .build()?;

// متغيرات البيئة نفسها التي تقرؤها أداة dna CLI ...
let client = Client::from_env()?;

// ... إضافةً إلى إعداداتك الخاصة
let client = ClientBuilder::from_env()?.http_client(my_reqwest_client).build()?;
```

<div dir="rtl">

| دالة `ClientBuilder` | الافتراضي | المعنى |
|---|---|---|
| `credentials(reseller_id, api_key)` | (مطلوب) | بيانات الاعتماد. تفشل `build()` بالخطأ `NO_CREDENTIALS` عندما تكون إحداهما فارغة. |
| `environment(...)` | `Production` | `Production` (‏`https://api.domainresellerapi.com`) أو `Ote` (‏`https://ote.domainresellerapi.com`). |
| `base_url(...)` | لا شيء | يتجاوز عنوان البيئة، مثلًا عند استخدام وكيل (proxy). يُحتفَظ ببادئة المسار (path prefix). |
| `timeout(...)` | 60 ثانية | المهلة الزمنية لمحاولة واحدة. |
| `max_retries(...)` | 2 | عدد مرات تكرار الطلب الفاشل (انظر [إعادة المحاولة](#الأمان-وإعادة-المحاولة-وحدود-معدل-الطلبات)). القيمة 0 توقف إعادة المحاولة. |
| `max_retry_delay(...)` | 30 ثانية | أطول انتظار قبل إعادة المحاولة. |
| `pace_requests(...)` | `true` | يوزّع الطلبات زمنيًا لتبقى ضمن حدود معدل الطلبات في البوابة (gateway). |
| `read_only(...)` | `false` | يمنع كل طلب ليس قراءةً، قبل إرساله. |
| `user_agent(...)` | `DomainNameAPI-Rust/<version>` | ترويسة `User-Agent`. |
| `http_client(...)` | العميل الخاص بالحزمة | `reqwest::Client` الخاص بك (وكيل، إعدادات TLS). |

تقرأ `from_env()` ما يلي:

| المتغير | المعنى |
|---|---|
| `DNA_RESELLER_ID`, `DNA_API_KEY` | بيانات الاعتماد. |
| `DNA_ENV` | `production` أو `ote` (وأيضًا `prod` و`live` و`test` و`sandbox`). |
| `DNA_TEST_MODE` | القيم `true` أو `1` أو `on` أو `yes` تختار OTE. تُستخدم عند عدم ضبط `DNA_ENV`. |
| `DNA_BASE_URL` | تجاوز اختياري للعنوان الأساسي. |
| `DNA_READ_ONLY` | القيمة `true` تفعّل وضع القراءة فقط. |

النوع `Client` هو `Clone + Send + Sync`، ونسخه (clone) رخيص (فهو يحتفظ بـ `Arc`). أنشئ عميلًا واحدًا لكل مجموعة بيانات اعتماد
وشاركه طوال عمر التطبيق: فتنظيم وتيرة الطلبات يجري لكل عميل على حدة.

## الاستخدام

لكل مجال في API عميله الخاص، ويحمل اسمًا مطابقًا لمجموعات الأوامر في CLI: `client.account()`،
و`client.contacts()`، و`client.domains()` (مع `forwarding()` و`child_name_servers()` و`documents()`)،
و`client.dns()`، و`client.dnssec()`، و`client.ssl()`، و`client.hosting()`، و`client.orders()` و`client.products()`.
المدخلات المطلوبة تُمرَّر وسائطَ؛ أما العمليات ذات المدخلات الاختيارية فتأخذ طلبًا يُبنى بـ `new(...)` ودوال ضبط (setters)
متسلسلة. وتذكر الدوال التي تخصم من رصيدك ذلك في توثيقها.

### الحساب

</div>

```rust
let account = client.account().get().await?;          // reseller_id, reseller_name, try_balance, usd_balance
let profile = client.account().profile().await?;      // name, company_name, account_currency, status, ...

// رسوم هذا الشهر، عبر كل الصفحات
let request = AccountTransactionsRequest::new()
    .charge_type(ChargeType::Subtract)
    .start_date("2026-11-01T00:00:00Z".parse()?)
    .transaction_types([TransactionType::Register, TransactionType::Renew]);
let mut transactions = client.account().all_transactions(request);
while let Some(t) = transactions.try_next().await? {
    println!("{:?} {} {} {}", t.creation_time, t.transaction_type, t.amount, t.amount_currency);
}
```

<div dir="rtl">

### النطاقات

**البحث**

</div>

```rust
let one = client.domains().search("example.com").await?;
let available = one.success && one.info.is_available();

let many = client.domains().bulk_search(["example.com", "example.net", "example.com.tr"]).await?;
for domain in many.available() {
    println!("{}: {} {}", domain.domain_name, domain.price, domain.currency);
}

// مع عملة
client.domains().search_with(DomainSearchRequest::new("example.com").currency("USD")).await?;
```

<div dir="rtl">

**القوائم والتفاصيل**

</div>

```rust
let page = client.domains().list(DomainListRequest::new().limit(50).statuses([DomainStatus::Active])).await?;

let all: Vec<Domain> = client.domains().list_all(DomainListRequest::new().domain_name("example")).try_collect().await?;

let info = client.domains().info("example.com").await?;
println!("{} {:?}", info.auth_code, info.expiration_date);
if let Some(registrant) = info.contact(ContactType::Registrant) {
    println!("{}", registrant.email);
}
```

<div dir="rtl">

**التسجيل والتجديد والنقل والاستعادة** (تخصم من رصيدك)

</div>

```rust
// جهة اتصال واحدة للأدوار الأربعة كلها، لمدة سنتين
let request = DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact.clone()), ["tr.apiname.com", "eu.apiname.com"])
    .years(2);
client.domains().register_with_contacts(request).await?;

// جهات اتصال مختلفة لكل دور، وسمات جهة التسجيل، وخدمة الوكيل الأمين (trustee)
let request = DomainRegisterWithContactsRequest::new(
    "example.com.tr",
    DomainContacts { registrant: Some(owner), admin: Some(staff.clone()), tech: Some(staff.clone()), billing: Some(staff) },
    ["tr.apiname.com", "eu.apiname.com"],
)
.tld_attribute("<key>", "<value>"); // المفاتيح: client.products().tld("com.tr").await?.unwrap().attributes

// بمعرّفات جهات اتصال موجودة
let request = DomainRegisterRequest::new("example.com", ContactHandles::same("DNA-12345"), ["tr.apiname.com", "eu.apiname.com"]);
client.domains().register(request).await?;

let renewal = client.domains().renew_check("example.com", Some(1)).await?;   // مجاني
if renewal.renewable {
    client.domains().renew("example.com", 1).await?;
}

let check = client.domains().transfer_check("example.com", "EPP-CODE").await?;  // مجاني
if check.transferable {
    client.domains().transfer(DomainTransferRequest::new("example.com", "EPP-CODE")).await?;
}

client.domains().restore("example.com").await?;
```

<div dir="rtl">

تُفحص الطلبات قبل إرسالها. التسجيل دون أدوار جهات الاتصال الأربعة جميعها، أو جهة اتصال بلا عنوان بريد إلكتروني،
أو مدة 0 سنة، يُعيد خطأ `ErrorKind::InvalidInput` يسرد كل المشكلات، ولا يصل شيء إلى API.

> [!NOTE]
> قد تجيب API عن طلب تسجيل أو تجديد أو نقل بحالة HTTP 200 مع `"success": false` (مثلًا عندما ترفض
> جهة التسجيل (registry) النطاق). تُعيد الحزمة هذه الإجابة كما هي، كما تفعل حزمة Java: تحقّق من
> `DomainOperationResult::success` و`message`. وينطبق الأمر نفسه على `SslOperationResult` و`ContactCreateResult`.

**الإعدادات**

</div>

```rust
client.domains().lock("example.com").await?;
client.domains().unlock("example.com").await?;
client.domains().set_privacy("example.com", true).await?;
client.domains()
    .set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net", "ns2.example.net"]))
    .await?;
client.domains()   // نطاق موقَّع بـ DNSSEC
    .set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net"]).dnssec_mode(DnssecMode::DisableDnssec))
    .await?;
let auth_code = client.domains().update_auth_code("example.com").await?;

client.domains().update_contacts("example.com", DomainContacts::same(contact)).await?;
client.domains().set_contact_handles("example.com", ContactHandles { tech: "DNA-12345".into(), ..Default::default() }).await?;
```

<div dir="rtl">

**إعادة توجيه URL وخوادم الأسماء الفرعية والمستندات**

</div>

```rust
client.domains().forwarding().set("example.com", "https://www.example.net", ForwardType::Standard).await?;
let forwarding = client.domains().forwarding().get("example.com").await?;   // None عندما لا تكون مضبوطة
client.domains().forwarding().delete("example.com").await?;

// يُكتشف إصدار IP لكل عنوان تلقائيًا
client.domains().child_name_servers().add("example.com", "ns1.example.com", ["192.0.2.10", "2001:db8::10"]).await?;
client.domains().child_name_servers()
    .update(DomainHostUpdateRequest::new("example.com", "ns1.example.com", ["192.0.2.20"]))
    .await?;
client.domains().child_name_servers().delete("example.com", "ns1.example.com").await?;

// مستندات جهة التسجيل (غالبًا لـ .tr)
let document = DocumentUpload::from_file(DocumentType::IdentityCard, "id.pdf")?;
client.domains().documents().upload("example.com.tr", [document]).await?;
let documents: Vec<Document> = client.domains().documents().list_all("example.com.tr").try_collect().await?;
```

<div dir="rtl">

### جهات الاتصال

معرّفات جهات الاتصال (handles) هي جهات اتصال قابلة لإعادة الاستخدام. أنشئ واحدة مرة واحدة واستخدمها لنطاقات كثيرة.

</div>

```rust
let created = client.contacts().create(contact.clone()).await?;
let handle = created.handle;                                         // مثلًا DNA-12345

let details = client.contacts().get(&handle).await?;                 // جهة الاتصال وحالة التحقق منها
let result = client.contacts().update(&handle, contact).await?;
println!("{} of {} domains updated", result.stats.success_count, result.stats.total_domain_count);

let verifications = client.contacts().verification(&handle).await?;
client.contacts().resend_verification(&handle).await?;
```

<div dir="rtl">

### DNS

سجلات المنطقة (zone) للنطاقات الموجودة على خوادم أسماء المنصة. تُحَلّ السجلات علنًا بمجرد أن يستخدم النطاق تلك الخوادم،
وهذا ما تتولاه `apply`.

</div>

```rust
for set in client.dns().list("example.com").await? {
    println!("{} {} {} {:?}", set.name, set.record_type, set.ttl, set.values());
}

let dns = client.dns();
dns.add(DnsAddRequest::new("example.com", "www", RecordType::A, ["192.0.2.10"])).await?;
dns.add(DnsAddRequest::new("example.com", "@", "txt", ["v=spf1 include:_spf.example.net ~all"])).await?;   // "txt" تعني TXT
dns.add(DnsAddRequest::new("example.com", "@", RecordType::Mx, ["10 mx1.example.net", "20 mx2.example.net"]).ttl(3600)).await?;
dns.update(DnsUpdateRequest::new("example.com", "www", RecordType::A, ["192.0.2.20"]).ttl(300)).await?;
dns.delete(DnsDeleteRequest::new("example.com", "www", RecordType::A, "192.0.2.20")).await?;

let applied = dns.apply("example.com").await?;
```

<div dir="rtl">

تفحص الحزمة ما سترفضه API: سجل CNAME عند الجذر (apex)، أو سجل MX بلا أولويته، أو قيمة CAA ناقصة الأجزاء الثلاثة،
أو سجل A ليس عنوان IPv4، أو TTL خارج النطاق 1–86400. تُباعَد استدعاءات المنطقة بثانية واحدة، وهي حدّ البوابة.
تجيب بيئة الاختبار OTE عن استدعاءات المنطقة بحالة HTTP 500، لذا لا يمكن تجربة شيفرة DNS إلا في بيئة الإنتاج.

### DNSSEC

</div>

```rust
let info = client.dnssec().get("example.com").await?;      // status, ds_records, allowed_algorithms, ...

let record: DsData = "12345 13 2 49FD46E6C4B45C55D4AC69CBD3CD34AC1AFE51DE".parse()?;
client.dnssec().add("example.com", [record.clone()]).await?;
client.dnssec().remove("example.com", record).await?;
client.dnssec().disable("example.com").await?;            // يزيل كل سجلات DS
```

<div dir="rtl">

### SSL

يمكن أن يكون للنطاق الواحد عدة شهادات، ولذلك تُحدَّد الشهادة القائمة عبر **معرّف الشهادة** (من النوع GUID)،
لا عبر اسم النطاق أبدًا. تعيد `initialize` و`renew` هذا المعرّف: احتفظ به. وتعثر `list` على معرّفات الشهادات
التي تملكها بالفعل.

</div>

```rust
// 1. الشراء (يخصم من الرصيد). تأتي رموز الحزم من client.products().ssls(...).
let approvers = client.ssl().approvers(SslApproversRequest::for_purchase("example.com", "PKG-CODE")).await?;
let order = client.ssl().initialize("example.com", "PKG-CODE").await?;
let certificate_id = order.certificate_id;                  // احتفظ به: كل استدعاء لاحق يأخذه

// 2. طلب CSR ومفتاح خاص، إن لم يكونا لديك
let csr = client.ssl().generate_csr(
    SslGenerateCsrRequest::new("example.com")
        .country("TR").state("Istanbul").locality("Istanbul").organization("Example Ltd").organization_unit("IT"),
).await?;

// 3. الإعداد خلال 15 يومًا: CSR، وطريقة التحقق، وجهة الاتصال
client.ssl().configure(
    SslConfigureRequest::new(&certificate_id, &csr.csr, certificate_contact.clone())
        .authentication_type(AuthenticationType::Email)
        .approver_mail(&approvers[0])
        .web_server_type(WebServerType::Apache),
).await?;

// 4. انتظر الإصدار، ثم نزّل الشهادة
let status = client.ssl().status(&certificate_id).await?;
if status.is_issued {
    let content = client.ssl().content(&certificate_id).await?;
    std::fs::write("fullchain.pem", content.full_chain())?;
}

let details = client.ssl().get(&certificate_id).await?;     // الاشتراك، والإعداد، وجهات الاتصال
client.ssl().reissue(SslReissueRequest::new(&certificate_id).csr(new_csr).private_key(new_key)).await?;
client.ssl().revoke(&certificate_id).await?;                // لا يمكن التراجع عنه
```

<div dir="rtl">

**عرض الشهادات والعثور على المعرّف**

</div>

```rust
let page = client.ssl().list(SslListRequest::new().domain_name("example.com")).await?;   // الأحدث أولًا

let mut all = client.ssl().list_all(SslListRequest::new());
while let Some(c) = all.try_next().await? {
    println!("{} {} {} {:?}", c.certificate_id, c.domain_name, c.status, c.subscription_expires_at);
}

// عناوين المعتمِدين (approvers) لشهادة تملكها بالفعل
let mails = client.ssl().approvers(SslApproversRequest::for_certificate(&certificate_id)).await?;
```

<div dir="rtl">

**التجديد** (يخصم من رصيدك)

</div>

```rust
// مسموح به بدءًا من 30 يومًا قبل details.subscription.expires_at. التجديد شهادة جديدة بمعرّف خاص بها.
let renewal = client.ssl().renew(&certificate_id).await?;
client.ssl().configure(
    SslConfigureRequest::new(&renewal.certificate_id, &new_csr, certificate_contact)
        .authentication_type(AuthenticationType::Email)
        .approver_mail(&approvers[0]),
).await?;
```

<div dir="rtl">

يجب إعداد الشهادة المشتراة أو المجدَّدة خلال 15 يومًا، وإلا أُلغيت. للتجديد استدعِ `renew` ثم `configure`
بالمعرّف الجديد. لا تشترِ مجددًا عبر `initialize`: فذلك شراء مستقل بسعر الشراء، ولن توسَم الشهادة القديمة بأنها
مجدَّدة أبدًا. معرّف الشهادة الفارغ أو الذي ليس GUID (مثل اسم نطاق تمرّره شيفرة أقدم) يُرفض قبل إرسال أي شيء.

### الاستضافة

</div>

```rust
let packages = client.hosting().packages().await?;
let created = client.hosting().create("example.com", "starter-linux").await?;          // يخصم من الرصيد
let status = client.hosting().status(HostingStatusRequest::by_service_id(&created.service_id)).await?;
if status.is_active() {
    println!("{} {}", status.panel_address, status.username);
}
let hosting = client.hosting().get(HostingGetRequest::by_domain("example.com")).await?;
let plans = client.hosting().plans(None).await?;
```

<div dir="rtl">

### الطلبات والمنتجات

</div>

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

<div dir="rtl">

## الطلبات والسمة `Request`

كل عملية هي نوع طلب يحمل اسم أمر CLI المقابل لها: `dna domain renew` هو `DomainRenewRequest`،
و`dna domain renew-check` هو `DomainRenewCheckRequest`، و`dna dns add` هو `DnsAddRequest`. تبنيها لك دوال المجالات،
ويمكنك أيضًا إنشاؤها وإرسالها بنفسك:

</div>

```rust
let request = DomainRenewRequest::new("example.com", 2);
let endpoint = request.endpoint();
println!("{endpoint} is {}", endpoint.safety);   // "POST /api/v1/domains/renew is billable"

let result = client.send(&request).await?;

// الأمر نفسه، مع تفاصيل HTTP: معرّف الارتباط، وحالة الحصة المرنة، وعدد المحاولات، والمدة
let response = client.send_with_response(&request).await?;
println!("{} quota {:?}", response.correlation_id, response.quota);
```

<div dir="rtl">

لا توجد وراثة في Rust؛ أساس كل طلب هو سمة (trait):

| السمة (Trait) | الغرض |
|---|---|
| `Request` | كل العمليات. `type Response` هو نوع الإجابة؛ و`Empty` للعمليات التي لا تُعيد شيئًا (قفل، فك القفل، إضافة سجل DNS، ...). |
| `PagedRequest<T>` | عمليات القوائم (`Response = Page<T>`). تتيح الوصول إلى `Paging` (skip وlimit وsort)، وتعمل مع `Client::list_all`. |

يصف الطلبُ نفسه عبر `endpoint()`: الأسلوب (method)، والمسار، وفئة الأمان، وما إذا كان متساوي القوى (idempotent). أما الدوال
الثلاث الأخرى فلها تطبيقات افتراضية: تُعيد `query()` سلسلة الاستعلام (يحذف `Query` القيم الفارغة)، وتُعيد `body()` جسم JSON
(يبني `JsonBody` جسمًا مع حذف القيم الفارغة، أو استخدم `serde_json::json!`)، وتجمع `validate()` المشكلات
قبل الإرسال. لاستدعاء نقطة نهاية لا تغطيها الحزمة بعد، اكتب طلبك الخاص:

</div>

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
        Endpoint::get("/api/v1/domains/notes")          // GET عملية قراءة
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

<div dir="rtl">

يحصل طلبك على كل ما تحصل عليه الطلبات المدمجة: بيانات الاعتماد، وإعادة المحاولة بحسب فئة الأمان، وتنظيم وتيرة الطلبات،
ووضع القراءة فقط، ومعالجة الأخطاء. للتغيير، استخدم `Endpoint::post(path, OperationSafety::Write)` (أو `Billable` أو `Destructive`)،
ولا تُضف `.idempotent()` إلا عندما يكون إرساله مرتين غير ضار.

## الأخطاء

كل طلب فاشل يُعيد `Error`، حتى عندما يفشل الطلب في الفحوص ولا يُرسَل شيء.

</div>

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
        // فشل طلب مدفوع أو مُغيِّر بسبب الشبكة: ربما وصل إلى API.
        // تحقّق من الحالة قبل إرساله مرة أخرى.
        let info = client.domains().info("example.com").await?;
    }
    Err(e) => return Err(e.into()),
}
```

<div dir="rtl">

| الدالة | المعنى |
|---|---|
| `kind()` | تصنيف ثابت (الجدول أدناه). اتخذ قراراتك بناءً عليه، لا على الرسالة. |
| `code()` | رمز الخطأ من API (مثل `Dna.DomainService:Domain:10007`)، أو أحد رموز الحزمة أدناه. الرموز الشائعة متاحة ثوابتَ في `domainnameapi::codes`. |
| `message()` | ما الذي حدث. تُستبدل الرسائل الجاهزة العامة بتفاصيل API. |
| `status()` | حالة HTTP، أو `None` إن لم تصل أي إجابة. |
| `details()`, `validation_errors()` | تفاصيل إضافية ومشكلات على مستوى الحقول (من API، أو من الفحوص التي تسبق الإرسال). |
| `correlation_id()` | قيمة `X-Correlation-Id` من البوابة. اذكرها عند التواصل مع الدعم. |
| `hint()` | اقتراح قصير بما يجب فعله بعد ذلك. |
| `is_retryable()` | هل يمكن أن ينجح تكرار الطلب نفسه لاحقًا. |
| `is_outcome_unknown()` | ربما نُفّذ الطلب: فشل تغيير بسبب الشبكة، أو أفادت API بأن جهة التسجيل ربما نفّذته بالفعل. |
| `is_not_found()`, `is_insufficient_balance()` | اختصارات للحالات الشائعة. |
| `operation()`, `safety()` | الطلب الفاشل (`POST /api/v1/domains/renew`) وفئة الأمان الخاصة به. |
| `retry_after()`, `response_body()` | قيمة `Retry-After` من البوابة والإجابة الخام. |

ينفّذ `Error` السمتين `std::error::Error` و`Display` (‏`[CODE] message (HTTP 400); field: problem`)، لذا يعمل
مع `?` و`anyhow` و`thiserror`.

| `kind()` | متى |
|---|---|
| `Auth` | بيانات اعتماد خاطئة أو مفقودة، أو موزّع غير معروف أو موقوف في هذه البيئة (401، إعادة توجيه). |
| `Forbidden` | مصادَق عليه لكن غير مسموح له (403). |
| `NotFound` | النطاق أو جهة الاتصال أو الشهادة أو الطلب أو أي مورد آخر غير موجود (404). |
| `Validation` | رفضت API المدخلات أو فشلت قاعدة عمل: حقل غير صالح، أو نطاق مقفل، أو رصيد غير كافٍ (400، 422). |
| `Conflict` | حالة المورد لا تسمح بالعملية (409). |
| `RateLimited` | ما زال الطلب محدودًا بمعدل الطلبات بعد إعادة المحاولة التلقائية (429). |
| `Server` | فشل من جهة الخادم (5xx). غالبًا مؤقت. |
| `Network`, `Timeout` | لا إجابة: DNS أو TLS أو وكيل (proxy) أو إعادة ضبط الاتصال، أو لا إجابة خلال المهلة الزمنية. |
| `InvalidInput` | فشل الطلب في الفحوص، أو أن أحد إعدادات العميل غير صالح؛ ولم يُرسَل شيء. |
| `ReadOnly` | مُنع بسبب وضع القراءة فقط؛ ولم يُرسَل شيء. |
| `Unknown` | أي حالة أخرى، بما فيها إجابة لم تستطع الحزمة قراءتها. |

تُعطي `ErrorKind::as_str()` الأسماء المستخدمة في CLI (‏`auth`، `not_found`، `rate_limited`، ...). الرموز التي تضبطها الحزمة بنفسها:

| الرمز | المعنى |
|---|---|
| `INVALID_RESELLER` | أعادت البوابة التوجيه إلى صفحة الإيقاف: معرّف الموزّع غير معروف أو غير نشط أو موقوف **في هذه البيئة**. |
| `INVALID_API_KEY` | المفتاح لا يطابق الموزّع. مفاتيح الإنتاج وOTE مختلفة. |
| `MISSING_API_KEY`, `MISSING_RESELLER_ID`, `NO_CREDENTIALS` | إحدى بيانات الاعتماد فارغة. |
| `SUBRESELLER_NOT_ALLOWED` | لا يمكن لحسابات الموزّعين الفرعيين استخدام API. استخدم بيانات اعتماد الموزّع الرئيسي. |
| `UNAUTHORIZED` | أي 401 آخر. |
| `RATE_LIMITED` | 429 بعد إعادة المحاولة. |
| `NETWORK`, `TIMEOUT` | لم تصل أي إجابة. |
| `READ_ONLY` | مُنع بسبب وضع القراءة فقط. |
| `INVALID_INPUT`, `INVALID_CONFIGURATION` | فشل أحد الطلبات في الفحوص، أو أن أحد إعدادات العميل غير صالح. |
| `EMPTY_RESPONSE`, `INVALID_RESPONSE` | كانت الإجابة الناجحة فارغة أو تعذّرت قراءتها. |
| `HTTP_<status>` | لم ترسل API رمز خطأ. |

## الأمان وإعادة المحاولة وحدود معدل الطلبات

لكل طلب فئة أمان، وهي نفسها التي تستخدمها أداة `dna` CLI:

| `OperationSafety` | تعني | أمثلة |
|---|---|---|
| `Read` | بلا آثار جانبية، حتى لو استخدمت API الأسلوب POST أو PUT | البحث، التفاصيل، القوائم، فحوص التجديد والنقل، إنشاء CSR، حالة الشهادة |
| `Write` | تغيير مجاني يمكن التراجع عنه | سجلات DNS، القفل، الخصوصية، خوادم الأسماء، جهات الاتصال |
| `Billable` | يخصم من رصيدك | التسجيل، التجديد، النقل، الاستعادة، شراء SSL وتجديده، طلب استضافة |
| `Destructive` | يحذف شيئًا أو يصعب التراجع عنه | حذف سجل DNS، إزالة DNSSEC، إلغاء شهادة |

- **وضع القراءة فقط** (‏`read_only(true)` أو `DNA_READ_ONLY=true`) يمنع كل ما ليس `Read` قبل إرساله.
  استخدمه أثناء الاستكشاف، وفي التقارير ولوحات المعلومات.
- **إعادة المحاولة.** تُعاد محاولة إجابات 429 دائمًا، مع احترام `Retry-After`. وهذا آمن حتى للطلبات المدفوعة، لأن
  البوابة ترفضها قبل تنفيذ أي شيء. أما أخطاء الشبكة والمهلات وإجابات 502/503/504 فلا تُعاد محاولتها إلا لعمليات القراءة
  ولعمليات الكتابة التي تصنّفها CLI بأنها متساوية القوى (idempotent): القفل، وفك القفل، والخصوصية، وخوادم الأسماء، وتحديث DNS
  وتطبيقه. لا يُرسَل التسجيل أو التجديد مرتين أبدًا.
- **تنظيم وتيرة الطلبات.** تسمح البوابة بـ 5 طلبات في الثانية لكل مفتاح، و2 في الثانية لعمليات البحث عن النطاقات. يباعد العميل
  بين الطلبات ليبقى ضمن هذه الحدود، ويباعد بين استدعاءات منطقة DNS بثانية واحدة. معلومات النطاق وقائمة TLD
  غير محدودتين بمعدل الطلبات.
- **الحصة المرنة (soft quota).** تكلّف عمليات البحث والاستعلام عن المعلومات نقطة واحدة، وكل ما عداها 10 نقاط. بعد تجاوز 200 نقطة تبطئ البوابة كل
  طلب بما يصل إلى 5 ثوانٍ؛ وتتناقص النقاط بنسبة 20% في الدقيقة. تعرض `send_with_response` الحالة في `quota`.

## الشيفرة غير المتزامنة والشيفرة الحاجبة

الحزمة غير متزامنة (async) وتحتاج إلى بيئة تشغيل Tokio (التي يشغّلها `#[tokio::main]`، أو بيئة خادمك). الـ futures
من النوع `Send`، لذا تعمل داخل `tokio::spawn` وفي معالجات axum وactix. يمكن قراءة القوائم صفحةً صفحة (`list`) أو على هيئة
`Stream` من العناصر (`list_all`)، وهو لا يجلب الصفحة التالية إلا عند الحاجة إليها:

</div>

```rust
use futures_util::TryStreamExt;

let expiring: Vec<Domain> = client
    .domains()
    .list_all(DomainListRequest::new())
    .try_filter(|d| std::future::ready(d.remaining_days < 30))
    .try_collect()
    .await?;
```

<div dir="rtl">

لا توجد API حاجبة (blocking): فهي ستضاعف حجم الواجهة مقابل فائدة قليلة. في الشيفرة المتزامنة، شغّل الاستدعاءات على
بيئة تشغيل:

</div>

```rust
let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
let account = runtime.block_on(client.account().get())?;
```

<div dir="rtl">

## اختبار شيفرتك

لا تحتاج الحزمة إلا إلى نقطة نهاية HTTP. لاختبار الشيفرة التي تستخدمها دون استدعاء API، وجّه `base_url` إلى خادم
محلي (مثلًا خادم مبني بـ `wiremock` أو `httpmock` أو `axum`) وأوقف تنظيم وتيرة الطلبات:

</div>

```rust
let client = Client::builder()
    .credentials("test", "test")
    .base_url(format!("http://{}", server_address))
    .pace_requests(false)
    .build()?;
```

<div dir="rtl">

تنفّذ النماذج `Default` و`Deserialize`، لذا يمكن أن تأتي بيانات الاختبار من JSON؛ ويمكن إنشاء الأخطاء لكائناتك الوهمية (mocks)
عبر `Error::new(kind, code, message)`. للاختبار على API الحقيقية، استخدم OTE بمفتاحها الخاص.

## بنية المشروع

</div>

```
src/
├── lib.rs           توثيق الحزمة (crate) وإعادة التصدير (re-exports)
├── client.rs        Client, ClientBuilder, Environment: الإرسال، إعادة المحاولة، تنظيم الوتيرة، تحليل الأخطاء، list_all
├── request.rs       Request, PagedRequest, Endpoint, OperationSafety, Query, JsonBody, Validator, Page, Empty
├── error.rs         Error, ErrorKind, codes
├── de.rs            فك ترميز متسامح ووحدات الماكرو التي تعرّف النماذج والقيم المُعدَّدة (داخلي)
├── account.rs       AccountClient     الأرصدة والملف الشخصي والمعاملات
├── contacts.rs      ContactsClient    معرّفات جهات الاتصال
├── domains/         DomainsClient     النطاقات، إضافةً إلى forwarding() و child_name_servers() و documents()
├── dns.rs           DnsClient         سجلات المنطقة
├── dnssec.rs        DnssecClient      سجلات DS
├── ssl.rs           SslClient         الشهادات
├── hosting.rs       HostingClient     حزم الاستضافة وخدماتها
├── orders.rs        OrdersClient      الطلبات
└── products.rs      ProductsClient    نطاقات TLD ومنتجات SSL والأسعار
tests/               اختبارات على API وهمية، واختبار تغطية المواصفة، و testdata/openapi.json
examples/            برامج قابلة للتشغيل
```

<div dir="rtl">

لكل وحدة (module) مجال الشكل نفسه: النماذج والقيم المُعدَّدة (enums) الخاصة بإجاباتها، ونوع طلب واحد لكل عملية
(`XxxRequest`)، و`XxxClient`.

## التطوير

</div>

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                    # اختبارات الوحدة وAPI الوهمية والمواصفة والتوثيق؛ لا يستدعي أيٌّ منها API الحقيقية
cargo build --examples
```

<div dir="rtl">

إن لم تكن لديك سلسلة أدوات (toolchain) محلية، فاستخدم Docker:

</div>

```bash
docker run --rm -v "$PWD":/src -w /src rust:1.83 sh -c 'rustup component add rustfmt clippy; cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo test'
```

<div dir="rtl">

لا تستدعي الاختبارات API الحقيقية أبدًا: فهي تستخدم خادم HTTP صغيرًا على 127.0.0.1 (`tests/common`). يشغّلها CI على
أدنى إصدار مدعوم من Rust ‏(1.83) وعلى أحدث إصدار مستقر.

الملف `tests/testdata/openapi.json` هو وثيقة API التي تضمّنها أداة `dna` CLI. يتحقق `tests/spec.rs` من أن لكل عملية
فيه نوع طلب واحدًا بالضبط بالأسلوب والمسار وفئة الأمان وخاصية تساوي القوى (idempotency) نفسها في CLI، ومن أن كل طلب
لا يرسل إلا معاملات استعلام وخصائص جسم تصفها الوثيقة (بما فيها الكائنات المتداخلة، مع وجود الخصائص المطلوبة)،
ومن أن أنواع الإجابات تستخدم الأسماء الواردة في الوثيقة. عندما تضيف API عملية جديدة، حدّث الوثيقة، وسيفشل الاختبار
إلى أن يوجد نوع الطلب.

## الترقية من 1.x

الإصدار 2.0 حزمة SDK جديدة. كان الإصدار 1.x نسخة منقولة من مكتبة PHP: عميل `DomainNameApi` حاجب (blocking) يُعيد
أغلفة `serde_json::Value` غير مكتوبة بأنواع. أما 2.0 فغير متزامن (async)، مع طلبات ونماذج مكتوبة بأنواع صريحة، و`Result<T, Error>`، وبانٍ (builder).

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
| `result["result"] == "ERROR"` | `Err(e)` مع `e.kind()` و`e.code()` |

استدعاءات الموافقة على النقل وإلغائه ورفضه (approve/cancel/reject) واستدعاء `sync_from_registry` في 1.x ليست جزءًا من
وثيقة REST API، ولا مقابل لها في 2.0. يبقى الحد الأدنى لإصدار Rust (MSRV) هو 1.83؛ وحلّت `reqwest` محل `ureq`، لذا يحتاج 2.0
إلى بيئة تشغيل Tokio. كان الإصدار 1.x يرسل تقارير الأخطاء إلى Sentry؛ أما 2.0 فلا يرسل شيئًا إلى أي جهة سوى API.

## الروابط

- [`dna` CLI وخادم MCP](https://github.com/domainreseller/cli): API نفسها من الطرفية ومن وكلاء الذكاء الاصطناعي.
- [توثيق API](https://www.domainnameapi.com/docs/)
- [سجل التغييرات](CHANGELOG.md)

## الترخيص

Apache License 2.0. راجع [LICENSE](LICENSE).

</div>
