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

# rust-dna — عميل DomainNameAPI للغة Rust

عميل بلغة Rust لواجهة REST البرمجية الخاصة بمسجّل النطاقات DomainNameAPI. وهو نقل أمين لمكتبة PHP المرجعية [php-dna](https://github.com/domainreseller/php-dna): الدوال نفسها، والطلبات نفسها المرسلة عبر الشبكة، والقيم المُعادة نفسها. يُتحقَّق من كل نسخة منقولة مقابل 134 حالة توافق (conformance fixtures) من php-dna.

## 📦 التنزيل — استخدم دائمًا صفحة الإصدارات (Releases)!

⬇️ **احصل على أحدث إصدار مُختبَر من هنا: https://github.com/domainreseller/rust-dna/releases/latest**

> ⚠️ **لا** تستخدم الزر الأخضر **Code → Download ZIP** — فهو ينزّل فرع التطوير الخام. حزم الإصدارات مرقّمة ومُختبَرة وجاهزة للإنتاج.

## دليل التثبيت والتكامل

### الحد الأدنى من المتطلبات

- Rust 1.83 أو أحدث (إصدار 2021)
- لا حاجة إلى بيئة تشغيل غير متزامنة: الواجهة البرمجية حاجبة (blocking)
- اعتماديات التشغيل: `ureq` (HTTP مع TLS عبر rustls) و`rustls` و`serde_json`. لا حاجة إلى OpenSSL.

### التثبيت

**أ) اعتمادية Git (مثبّتة على وسم إصدار)**

```toml
[dependencies]
domainnameapi = { git = "https://github.com/domainreseller/rust-dna", tag = "v1.0.1" }
```

**ب) إصدار مُنزَّل**

فكّ ضغط أرشيف الإصدار بجوار مشروعك ووجّه Cargo إليه:

```toml
[dependencies]
domainnameapi = { path = "../rust-dna" }
```

تعيد الحزمة (crate) تصدير `serde_json::Value` والماكرو `json!`، لذا لا تحتاج إلى إضافة `serde_json` بنفسك (ويمكنك ذلك إن شئت).

## 🔑 بيانات اعتماد API — Reseller ID و API Key

تتواصل المكتبة مع واجهة REST الخاصة بـ DomainNameAPI وتحتاج إلى بيانَي اعتماد اثنين بالضبط، يُمرَّران إلى المُنشئ:

| المعامل | القيمة |
|---|---|
| الأول | Reseller ID — معرّف UUID مثل `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` |
| الثاني | API Key |

> 💡 تجد **Reseller ID** و**API Key** في لوحة تحكم DomainNameAPI ضمن **إعدادات API**.
> ⚠️ هذه **بيانات اعتماد API** — بيانات تسجيل الدخول إلى اللوحة **لن** تعمل هنا.

```rust
use domainnameapi::DomainNameApi;

let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";
let api_key = "your-api-key";

// Live environment
let mut dna = DomainNameApi::new(reseller_id, api_key, false);

// Test (OTE) environment
let mut dna = DomainNameApi::new(reseller_id, api_key, true);
```

تُزال المسافات من بداية ونهاية بيانَي الاعتماد. يقبل المعامل الثالث أيضًا نصوصًا، لأن لوحات التحكم كثيرًا ما تخزّن مربعات الاختيار كنص: `"true"` و`"1"` و`"on"` و`"yes"` (دون تمييز حالة الأحرف) تختار بيئة الاختبار، وأي قيمة أخرى (بما فيها `"false"` و`"off"`) تختار البيئة الحية.

## البدء السريع

```rust
use domainnameapi::DomainNameApi;

fn main() {
    let mut dna = DomainNameApi::new("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx", "your-api-key", false);

    let details = dna.get_details("example.com");
    if details["result"] == "OK" {
        println!("Expires: {}", details["data"]["Dates"]["Expiration"]);
    } else {
        eprintln!("{}: {}", details["error"]["Code"], details["error"]["Message"]);
    }
}
```

كل مثال في المجلد [examples/](examples) يُشغَّل عبر `cargo run --example <الاسم>`، مثل `cargo run --example get_details`.

## بنية الواجهة البرمجية

- **نوع عميل واحد**، `DomainNameApi`. تأخذ الدوال `&mut self` (يحتفظ العميل بآخر طلب لأغراض التشخيص).
- **القيم المُعادة من نوع `serde_json::Value`** وبالبنية نفسها التي تعيدها php-dna تمامًا — وليست نماذج ذات أنواع. أسماء المفاتيح، والأعلام النصية (`"true"` / `"false"`)، والمبالغ المنسّقة (`"11.9900"`) والأغلفة كلها متطابقة:
  - النجاح: `{"result": "OK", "data": {...}}`
  - الفشل: `{"result": "ERROR", "error": {"Code": "...", "Message": "...", "Details": "..."}}`
  - بعض الدوال تستخدم غلافًا آخر منقولًا من php-dna: تعيد `get_current_balance` كائن `OperationResult`، وتعيد `check_availability` قائمة مجرّدة عند النجاح، وتعيد `get_reseller_details` حقولها في المستوى الأعلى.
- **تُعاد الأخطاء ولا تُرمى.** أخطاء البوابة وأخطاء HTTP وأعطال الشبكة تعود جميعها كغلاف خطأ؛ لا شيء يسبب panic. كما تعيد عمليات التسجيل والنقل والتجديد خطأً عندما تجيب البوابة بـ HTTP 200 مع `success: false`.
- **المدخلات المُهيكلة** (جهات الاتصال، قوائم خوادم الأسماء، المرشحات، سمات TLD، محتويات السجلات) هي قيم `&Value` تُبنى باستخدام `json!`. تستخدم خرائط جهات الاتصال مفاتيح php-dna (`FirstName` و`EMail` و`AddressLine1` و`PhoneCountryCode`، ...)؛ ويُقبل الشكلان المسطّح والمتداخل (`Address.Line1` و`Phone.Phone.Number`). يمكن أن تكون قوائم خوادم الأسماء قائمة أو خريطة (تُستخدم القيم).
- **لا توجد معاملات افتراضية في Rust.** تأخذ كل دالة قائمة معاملات php-dna الكاملة بالترتيب نفسه. تُصدَّر القيم الافتراضية لـ php-dna كثوابت: `DEFAULT_TTL` (3600) و`DEFAULT_REASON` ("Owner request") و`DEFAULT_FORWARD_TYPE` ("Standard") و`DEFAULT_CURRENCY` ("USD") و`DEFAULT_TLD_COUNT` (20) و`DEFAULT_COMMAND` ("create"). ولـ `register_with_contact_info`، التي لها أربعة معاملات اختيارية، توجد أيضًا `register_with_contact_info_opts` التي تأخذ بنية `RegisterOptions` تكون قيمتها `Default` هي القيم الافتراضية لـ php-dna.

## الاستخدام

تفترض كل المقتطفات وجود `use domainnameapi::{json, DomainNameApi};` وعميل `let mut dna = DomainNameApi::new(...)`.

#### تسجيل النطاق

ملاحظة: تتطلب نطاقات .tr معاملات إضافية. يُستخدم معامل السمات الإضافية لأسماء النطاقات التي تحتاج إلى معلومات إضافية مثل .tr.

```rust
let contact = json!({
    "FirstName": "John",
    "LastName": "Doe",
    "Company": "Example Corp",
    "EMail": "john.doe@example.com",
    "AddressLine1": "123 Lorem Street",
    "AddressLine2": "Suite 456",
    "AddressLine3": "",
    "City": "Springfield",
    "Country": "US",
    "Fax": "1234567890",
    "FaxCountryCode": "1",
    "Phone": "9876543210",
    "PhoneCountryCode": 1,
    "Type": "Contact",
    "ZipCode": "12345",
    "State": "IL"
});
let contacts = json!({
    "Administrative": contact,
    "Billing": contact,
    "Technical": contact,
    "Registrant": contact
});

let result = dna.register_with_contact_info(
    "example.com",
    1,
    &contacts,
    &json!(["ns1.example.com", "ns2.example.com"]),
    true,  // registrar (theft protection) lock
    false, // privacy protection
    &json!({}),
);

// Same, with php-dna's defaults for the optional arguments
// (default nameservers, lock on, privacy off, no attributes):
use domainnameapi::RegisterOptions;
let result = dna.register_with_contact_info_opts("example.com", 1, &contacts, &RegisterOptions::default());
```

لنطاق `.tr` مرّر سمات TRABIS. ترسل المكتبة فقط ما تقبله البوابة (`TRABISDOMAINCATEGORY` و`TRABISCITIZENID` و`TRABISTAXOFFICE` و`TRABISTAXNUMBER`)، وتحوّل `1`/`0` إلى `Owner`/`Corporate`، وتصحّح التهجئة القديمة `TRABISCITIZIENID`:

```rust
let result = dna.register_with_contact_info_opts(
    "example.com.tr",
    1,
    &contacts,
    &RegisterOptions {
        name_servers: json!(["ns1.example.com", "ns2.example.com"]),
        additional_attributes: json!({
            "TRABISDOMAINCATEGORY": 1,
            "TRABISCITIZIENID": "12345678901",
            "TRABISNAMESURNAME": "John Doe",
            "TRABISCOUNTRYID": "840",
            "TRABISCITYID": "17"
        }),
        ..RegisterOptions::default()
    },
);
```

<details>
<summary>مثال على المخرجات لتسجيل النطاق</summary>

```json
{
  "data": {
    "ID": 123456,
    "Status": "Active",
    "DomainName": "example.com",
    "AuthCode": "Xy9#mK2$pL5@vN8",
    "LockStatus": "true",
    "PrivacyProtectionStatus": "false",
    "IsChildNameServer": "false",
    "Contacts": {
      "Billing": {"ID": 987654},
      "Technical": {"ID": 987654},
      "Administrative": {"ID": 987654},
      "Registrant": {"ID": 987654}
    },
    "Dates": {
      "Start": "2024-03-15T10:00:00",
      "Expiration": "2025-03-15T10:00:00",
      "RemainingDays": 365
    },
    "NameServers": ["ns1.example.com", "ns2.example.com"],
    "Additional": [],
    "ChildNameServers": []
  },
  "result": "OK"
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### تجديد النطاق

```rust
let result = dna.renew("example.com", 1);
```

<details>
<summary>مثال على المخرجات لتجديد النطاق</summary>

```json
{
  "result": "OK",
  "data": {
    "ExpirationDate": "2025-03-15T10:00:00"
  }
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### نقل النطاق

لا تُرسل عمليات النقل بيانات جهات الاتصال أبدًا: يتسلّم المسجّل جهات الاتصال من المسجّل السابق، لذا يحمل كل طلب نقل `contacts: []`.

```rust
let result = dna.transfer("example.com", "Xy9#mK2$", 3);
```

<details>
<summary>مثال على المخرجات لنقل النطاق</summary>

```json
{
  "data": {
    "ID": 0,
    "Status": "Active",
    "DomainName": "",
    "AuthCode": "",
    "LockStatus": "false",
    "PrivacyProtectionStatus": "false",
    "IsChildNameServer": "false",
    "Contacts": {
      "Billing": {"ID": 0},
      "Technical": {"ID": 0},
      "Administrative": {"ID": 0},
      "Registrant": {"ID": 0}
    },
    "Dates": {"Start": "", "Expiration": "2027-11-09T12:00:00", "RemainingDays": 365},
    "NameServers": [],
    "Additional": [],
    "ChildNameServers": []
  },
  "result": "OK"
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### إدارة عمليات النقل

```rust
// Can the domain be transferred with this auth code?
let check = dna.check_transfer("example.com", "Xy9#mK2$");

// Cancel an incoming transfer, approve or reject an outgoing one
let result = dna.cancel_transfer("example.com");
let result = dna.approve_transfer("example.com");
let result = dna.reject_transfer("example.com");
```

<details>
<summary>مثال على المخرجات لإدارة عمليات النقل</summary>

```json
{
  "result": "OK",
  "data": {
    "TransferAvailabilityStatus": true,
    "AuthCodeIsRequired": true,
    "AuthCodeIsValid": true,
    "UserTransferRequired": false,
    "TransferLock": false,
    "Message": "Domain can be transferred",
    "MessageKey": "TransferAvailable",
    "Additional": []
  }
}
```

تعيد `check_transfer` القيمة `"result": "ERROR"` مع `error` (`TRANSFER_NOT_AVAILABLE`) عندما يفيد السجل بأن النطاق غير قابل للنقل؛ ويكون `data` ممتلئًا في الحالتين. وتعيد `cancel_transfer` و`approve_transfer` و`reject_transfer`:

```json
{"result": "OK", "data": {"DomainName": "example.com", "Status": "Cancelled"}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### قائمة النطاقات

```rust
let result = dna.get_list(&json!({
    "OrderColumn": "Id",
    "OrderDirection": "ASC",
    "PageNumber": 0,
    "PageSize": 1000
}));

// No filters:
let result = dna.get_list(&json!({}));
```

يبدأ `PageNumber` من 0. ويمكن أيضًا تمرير معاملات البوابة الخاصة `MaxResultCount` و`SkipCount` و`Sorting`، ومرشحات مثل `DomainName`.

<details>
<summary>مثال على المخرجات لقائمة النطاقات</summary>

```json
{
  "data": {
    "Domains": [
      {
        "ID": 564346,
        "Status": "Active",
        "DomainName": "example.com",
        "AuthCode": "",
        "LockStatus": "true",
        "PrivacyProtectionStatus": "false",
        "IsChildNameServer": "false",
        "Contacts": {
          "Billing": {"ID": ""},
          "Technical": {"ID": ""},
          "Administrative": {"ID": ""},
          "Registrant": {"ID": ""}
        },
        "Dates": {
          "Start": "2024-03-15T10:00:00",
          "Expiration": "2025-03-15T10:00:00",
          "RemainingDays": 365
        },
        "NameServers": ["ns1.example.com", "ns2.example.com"],
        "Additional": [],
        "ChildNameServers": []
      }
    ]
  },
  "result": "OK",
  "TotalCount": 1
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### قائمة TLD

```rust
let result = dna.get_tld_list(100);
```

<details>
<summary>مثال على المخرجات لقائمة TLD</summary>

```json
{
  "data": [
    {
      "id": 1,
      "status": "Active",
      "maxchar": 63,
      "maxperiod": 10,
      "minchar": 1,
      "minperiod": 1,
      "tld": "com",
      "pricing": {
        "registration": {"1": "10.8100"},
        "renew": {"1": "11.0100"},
        "transfer": {"1": "10.6100"},
        "restore": {"1": "99.9000"}
      },
      "currencies": {
        "registration": "USD",
        "renew": "USD",
        "transfer": "USD",
        "restore": "USD"
      }
    }
  ],
  "result": "OK"
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### التحقق من توفر النطاق

```rust
let result = dna.check_availability(["hello", "world123x0"], ["com", "net"], 1, "create");
```

<details>
<summary>مثال على المخرجات للتحقق من التوفر</summary>

عند النجاح تكون النتيجة قائمة مجرّدة (بلا مفتاح `result`)؛ وعند الفشل تكون غلاف الخطأ.

```json
[
  {
    "TLD": "com",
    "DomainName": "hello",
    "Status": "notavailable",
    "Command": "create",
    "Period": 1,
    "IsFee": false,
    "Price": "10.8100",
    "Currency": "USD",
    "Reason": "Domain exists"
  },
  {
    "TLD": "com",
    "DomainName": "world123x0",
    "Status": "available",
    "Command": "create",
    "Period": 1,
    "IsFee": false,
    "Price": "10.8100",
    "Currency": "USD",
    "Reason": null
  }
]
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### تفاصيل النطاق

```rust
let result = dna.get_details("example.com");

// Re-read the domain from the registry (same result shape):
let result = dna.sync_from_registry("example.com");
```

<details>
<summary>مثال على المخرجات لتفاصيل النطاق</summary>

```json
{
  "data": {
    "ID": 564346,
    "Status": "Active",
    "DomainName": "example.com",
    "AuthCode": "DHQ!K52",
    "LockStatus": "true",
    "PrivacyProtectionStatus": "false",
    "IsChildNameServer": "true",
    "Contacts": {
      "Billing": {"ID": "DNA-B-4"},
      "Technical": {"ID": "DNA-T-3"},
      "Administrative": {"ID": "DNA-A-2"},
      "Registrant": {"ID": "DNA-R-1"}
    },
    "Dates": {
      "Start": "2025-05-26T16:08:37",
      "Expiration": "2027-05-26T16:08:37",
      "RemainingDays": 449
    },
    "NameServers": ["ns1.example.com", "ns2.example.com"],
    "Additional": {"TRABISDOMAINCATEGORY": "1"},
    "ChildNameServers": [
      {"ns": "ns1.example.com", "ip": "1.2.3.4"}
    ]
  },
  "result": "OK"
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### تعديل خوادم الأسماء

```rust
let result = dna.modify_name_server("example.com", &json!(["ns1.example.com", "ns2.example.com"]));
```

<details>
<summary>مثال على المخرجات لتعديل خوادم الأسماء</summary>

```json
{
  "result": "OK",
  "data": {
    "NameServers": ["ns1.example.com", "ns2.example.com"]
  }
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### تفعيل قفل النطاق

```rust
let result = dna.enable_theft_protection_lock("example.com");
```

<details>
<summary>مثال على المخرجات لتفعيل القفل</summary>

```json
{"result": "OK", "data": {"LockStatus": true}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### تعطيل قفل النطاق

```rust
let result = dna.disable_theft_protection_lock("example.com");
```

<details>
<summary>مثال على المخرجات لتعطيل القفل</summary>

```json
{"result": "OK", "data": {"LockStatus": false}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### إضافة خادم أسماء فرعي

```rust
let result = dna.add_child_name_server("example.com", "ns1.example.com", "1.2.3.4");
```

<details>
<summary>مثال على المخرجات لإضافة خادم أسماء فرعي</summary>

```json
{
  "result": "OK",
  "data": {
    "NameServer": "ns1.example.com",
    "IPAdresses": ["1.2.3.4"]
  }
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### حذف خادم أسماء فرعي

```rust
let result = dna.delete_child_name_server("example.com", "ns1.example.com");
```

<details>
<summary>مثال على المخرجات لحذف خادم أسماء فرعي</summary>

```json
{"result": "OK", "data": {"NameServer": "ns1.example.com"}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### تحديث خادم أسماء فرعي

```rust
let result = dna.modify_child_name_server("example.com", "ns1.example.com", "1.2.3.4");
```

<details>
<summary>مثال على المخرجات لتحديث خادم أسماء فرعي</summary>

```json
{
  "result": "OK",
  "data": {
    "NameServer": "ns1.example.com",
    "IPAdresses": ["1.2.3.4"]
  }
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### تعديل خصوصية النطاق

`status` قيمة من نوع `bool` أو قيمة كما تخزّنها لوحات التحكم: `"1"` و`"on"` و`"true"` و`"yes"` و`1` تعني التفعيل، وأي قيمة أخرى تعني الإيقاف. يُحتفظ بمعامل السبب (reason) للتوافق فقط ولا يُرسَل، لأن البوابة لا تحتوي على مثل هذا الحقل.

```rust
let result = dna.modify_privacy_protection_status("example.com", true, "Owner request");
```

<details>
<summary>مثال على المخرجات لتعديل الخصوصية</summary>

```json
{"data": {"PrivacyProtectionStatus": true}, "result": "OK"}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### جلب جهات اتصال النطاق

```rust
let result = dna.get_contacts("example.com");
```

<details>
<summary>مثال على المخرجات لجلب جهات الاتصال</summary>

```json
{
  "data": {
    "contacts": {
      "Administrative": {
        "ID": "DNA-A-2",
        "Status": "Active",
        "AuthCode": "",
        "FirstName": "John",
        "LastName": "Doe",
        "Company": "Example Corp",
        "EMail": "john@example.com",
        "Type": "Administrative",
        "Address": {
          "Line1": "123 Main Street",
          "Line2": "",
          "Line3": "",
          "State": "IL",
          "City": "Springfield",
          "Country": "US",
          "ZipCode": "62701"
        },
        "Phone": {
          "Phone": {"Number": "5551234567", "CountryCode": "1"},
          "Fax": {"Number": "", "CountryCode": ""}
        },
        "Additional": []
      },
      "Billing": {"...": "same shape"},
      "Registrant": {"...": "same shape"},
      "Technical": {"...": "same shape"}
    }
  },
  "result": "OK"
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### حفظ جهات اتصال النطاق

```rust
let contact = json!({
    "FirstName": "John",
    "LastName": "Doe",
    "Company": "Example Corp",
    "EMail": "john@example.com",
    "AddressLine1": "123 Main Street",
    "AddressLine2": "",
    "AddressLine3": "",
    "City": "Springfield",
    "Country": "US",
    "Fax": "5559876543",
    "FaxCountryCode": "1",
    "Phone": "5551234567",
    "PhoneCountryCode": 1,
    "Type": "Contact",
    "ZipCode": "62701",
    "State": "IL"
});

let result = dna.save_contacts("example.com", &json!({
    "Administrative": contact,
    "Billing": contact,
    "Technical": contact,
    "Registrant": contact
}));
```

<details>
<summary>مثال على المخرجات لحفظ جهات الاتصال</summary>

```json
{"result": "OK", "data": {"contacts": []}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### سجلات منطقة DNS

تخزّن البوابة السجلات على شكل **مجموعات** (اسم + نوع يحمل قيمة واحدة أو أكثر). كتابة مجموعة تستبدل جميع قيمها، وحذف أي قيمة يزيل المجموعة كاملة. تتولى دوال `*_value` عملية القراءة ثم الدمج ثم الكتابة نيابةً عنك، كي تتعامل لوحة التحكم مع السجلات واحدًا تلو الآخر. يمكن أن تكون الأسماء نسبية (`www`)، أو `@` / فارغة للجذر (apex)، أو مؤهلة بالكامل؛ وتُختصر إلى التسمية النسبية التي تتوقعها البوابة. الأنواع المدعومة: A وAAAA وALIAS وCAA وCNAME وDS وMX وNAPTR وPTR وSRV وTLSA وTXT. تُطبَّق التغييرات فورًا عندما تكون `apply` مساوية لـ `true` (القيمة الافتراضية في php-dna)؛ وإلا فاستدعِ `apply_resource_records`. يُترك فاصل لا يقل عن ثانية واحدة بين استدعاءات المنطقة لأن البوابة تحدّ من الطلبات المتلاحقة. إذا فشل التطبيق التلقائي، تعيد الدالة خطأ التطبيق (ويبقى التغيير معلّقًا). وتذكر النتائج النوع بأحرف كبيرة وقيمة TTL المرسلة فعليًا (ما دون 1 يصبح 3600، وما فوق 86400 يصبح 86400).

```rust
// List every record (one row per value; names come back as FQDNs)
let records = dna.get_resource_records("example.com");

// Make a set hold exactly these values (an empty list removes the set)
dna.set_resource_record_set("example.com", "@", "MX", ["10 mx1.example.com", "20 mx2.example.com"], 3600, true);

// Add, remove or replace one value, keeping the others (ttl None keeps the set's TTL)
dna.add_resource_record_value("example.com", "@", "MX", "30 mx3.example.com", None, true);
dna.remove_resource_record_value("example.com", "@", "MX", "30 mx3.example.com", true);
dna.replace_resource_record_value("example.com", "@", "A", "192.0.2.1", "192.0.2.2", None, true);

// Remove a whole set (pass a current value to save a lookup, or "")
dna.delete_resource_record_set("example.com", "www", "CNAME", true, "");

// Low-level calls, mirroring the gateway endpoints
dna.add_resource_record("example.com", "www", "A", &json!("192.0.2.5"), 3600, true);
dna.edit_resource_record("example.com", "www", "www", "A", &json!("192.0.2.6"), 600, true);
dna.delete_resource_record("example.com", "www", "192.0.2.5", "A", true);
dna.apply_resource_records("example.com");
```

<details>
<summary>مثال على المخرجات لسجلات منطقة DNS</summary>

`get_resource_records`:

```json
{
  "result": "OK",
  "data": {
    "records": [
      {"Name": "example.com.", "Type": "A", "TTL": 3600, "Content": "192.0.2.1", "Disabled": false},
      {"Name": "www.example.com.", "Type": "CNAME", "TTL": 300, "Content": "example.com.", "Disabled": false},
      {"Name": "example.com.", "Type": "MX", "TTL": 3600, "Content": "10 mx1.example.com.", "Disabled": false}
    ]
  }
}
```

`set_resource_record_set` / `add_resource_record_value`:

```json
{"result": "OK", "data": {"Name": "", "Type": "MX", "TTL": 3600, "Contents": ["10 mx1.example.com.", "20 mx2.example.com.", "30 mx3.example.com"]}}
```

القيمة الموجودة أصلًا (أو غير الموجودة عند الحذف) لا تغيّر شيئًا:

```json
{"result": "OK", "data": {"Name": "", "Type": "MX", "Contents": ["10 mx1.example.com."], "Changed": false}}
```

يُرفض النوع غير المدعوم قبل إرسال أي طلب:

```json
{"result": "ERROR", "error": {"Code": "ADD_ZONE", "Message": "Record type \"NS\" is not supported by the DNS zone API. Supported types: A, AAAA, ALIAS, CAA, CNAME, DS, MX, NAPTR, PTR, SRV, TLSA, TXT.", "Details": "..."}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### إعادة توجيه URL

```rust
let current = dna.get_forwarding("example.com");
let result = dna.set_forwarding("example.com", "https://www.example.org", "Standard"); // or "Frame" (masked)
let result = dna.delete_forwarding("example.com");
```

<details>
<summary>مثال على المخرجات لإعادة توجيه URL</summary>

```json
{"result": "OK", "data": {"Enabled": true, "RedirectAddress": "https://www.example.org/landing", "ForwardType": "Frame"}}
```

إذا لم تكن إعادة التوجيه مُعدّة (تجيب البوابة بـ `Redirect information not found`)، تعيد `get_forwarding` القيمة `{"result": "OK", "data": {"Enabled": false, "RedirectAddress": "", "ForwardType": ""}}`؛ وأي فشل آخر، مثل عدم وجود النطاق، يُعاد كخطأ. وتعيد `set_forwarding` القيمة `{"result": "OK", "data": {"RedirectAddress": "...", "ForwardType": "Standard"}}`. وتعيد `delete_forwarding` القيمة `{"result": "OK", "data": {"DomainName": "example.com", "Removed": true}}`، أو `"Removed": false` إذا لم تكن هناك إعادة توجيه مُعدّة.

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### جلب تفاصيل الموزّع

```rust
let result = dna.get_reseller_details();
let balance = dna.get_current_balance("USD"); // "USD" or "TRY"
```

<details>
<summary>مثال على المخرجات لتفاصيل الموزّع</summary>

```json
{
  "result": "OK",
  "id": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "active": true,
  "name": "Example Reseller",
  "balance": "601.64",
  "currency": "USD",
  "symbol": "$",
  "balances": [
    {"balance": "601.64", "currency": "USD", "symbol": "$"},
    {"balance": "1250.50", "currency": "TL", "symbol": "TL"}
  ]
}
```

`get_current_balance`:

```json
{
  "ErrorCode": 0,
  "OperationMessage": "Command completed succesfully.",
  "OperationResult": "SUCCESS",
  "Balance": "601.64",
  "CurrencyId": 2,
  "CurrencyInfo": null,
  "CurrencyName": "USD",
  "CurrencySymbol": "$"
}
```

يُبلغ الحساب عن رصيدَي USD وTRY فقط؛ وأي عملة أخرى (EUR وGBP، ...) تفشل مع `ErrorCode` بالقيمة `"BALANCE"`. عند الفشل تكون `OperationResult` مساوية لـ `"FAILED"`، ويحمل `ErrorCode` رمز الخطأ (مثل `"API_429"`)، وتكون `Balance` مساوية لـ `"0.00"`.

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### أدوات مساعدة (بدون شبكة)

```rust
let is_tr = dna.is_tr_tld("example.com.tr"); // true

// Complete and normalise a contact the way registration does
let contact = dna.validate_contact(&json!({
    "FirstName": " Jo ",
    "Phone": "+1 (202) 555-0100",
    "Country": "Turkey"
}));
```

<details>
<summary>مثال على المخرجات لـ validate_contact</summary>

```json
{
  "FirstName": "Jo",
  "Phone": "2025550100",
  "Country": "TR",
  "LastName": "Isimyok",
  "AddressLine1": "Addres yok",
  "City": "ISTANBUL",
  "ZipCode": "34000",
  "PhoneCountryCode": "1",
  "State": "ISTANBUL",
  "Fax": "",
  "FaxCountryCode": ""
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

## معالجة الأخطاء

تعيد كل دالة شبكية قيمة؛ تحقّق من `result`:

```rust
let out = dna.get_details("missing.com");
if out["result"] == "ERROR" {
    println!("{}", out["error"]["Code"]);        // "API_404"
    println!("{}", out["error"]["GatewayCode"]); // "Dna.DomainService:Domain:10007"
    println!("{}", out["error"]["Message"]);     // "The domain name you requested could not be found. ..."
    println!("{}", out["error"]["Details"]);
}
```

- تستخدم أخطاء HTTP الرمز `API_<الحالة>` (`API_400` و`API_404` و`API_429` و`API_500`، ...). ويتحوّل جسم الخطأ الفارغ إلى رسالة مقروءة مثل `HTTP 429 - Too Many Requests`. وعندما ترسل البوابة رمز خطأ خاصًا بها، يحمله الغلاف أيضًا في `GatewayCode` (مثل `"Dna.DomainService:Domain:10007"`).
- الرد 301/302 يعني بيانات اعتماد خاطئة: `Invalid API credentials`.
- تُجمَع أخطاء التحقق الصادرة عن البوابة في `Details` مع ذكر اسم الحقل، مثل `Contacts Phone - The field Phone must be a string with a maximum length of 16.`
- أعطال الشبكة أخطاء، ولا تكون أبدًا نجاحًا فارغًا. تُقرأ مهلة الانتظار هكذا: `Could not reach the DomainNameAPI service: the connection timed out. (...)`؛ وتُقرأ الأعطال الأخرى هكذا: `Curl error during request: ...`. رمزها هو `CURL_<رقم خطأ curl>` (`CURL_28` لانتهاء المهلة، و`CURL_6` لـ DNS، و`CURL_7` للاتصال المرفوض)؛ أما `API_<n>` فمخصّص لحالات HTTP.
- يفشل التحقق المحلي (نوع سجل غير مدعوم، عنوان إعادة توجيه مفقود، عدم وجود نطاقات للتحقق) قبل أي طلب برمز وصفي (`ADD_ZONE` و`SET_FORWARD` و`AVAILABILITY`، ...).

### التشخيص

```rust
dna.get_request_data();        // {"url", "method", "payload"} of the last request
dna.get_response_data();       // decoded last response, or {"raw_response": "<body>"}
dna.get_response_headers();    // {"url", "content_type", "http_code", "total_time", "headers"}
dna.get_parsed_response_data();// parsed body on success, {"Code", "Message", "Details"} on failure
dna.get_last_function();       // e.g. "request:GET domains/info"
dna.get_service_url();
dna.set_service_url("https://ote.domainresellerapi.com/api/v1");
```

### ناقل مخصّص

طبقة HTTP هي السمة (trait) `Transport`. الافتراضي هو `UreqTransport` (مكتبة `ureq` الحاجبة مع rustls، بمهلة إجمالية 30 ثانية و10 ثوانٍ للاتصال، ولا تُتبَع عمليات إعادة التوجيه). استبدله للوكلاء (proxies) أو العملاء المخصّصين أو الاختبارات:

```rust
use domainnameapi::{HttpRequest, HttpResponse, Transport, TransportError};

struct MyTransport;
impl Transport for MyTransport {
    fn send(&self, req: &HttpRequest) -> Result<HttpResponse, TransportError> {
        // perform req.method on req.url with req.headers and req.body
        Ok(HttpResponse { status: 200, body: "{}".into(), headers: vec![] })
    }
}

dna.set_transport(Box::new(MyTransport));
```

## وضع الاختبار

مرّر `true` (أو `"true"` أو `"1"` أو `"on"` أو `"yes"`) كمعامل ثالث للمُنشئ لاستخدام بيئة الاختبار OTE على `https://ote.domainresellerapi.com/api/v1`. تذهب الاستدعاءات الحية إلى `https://api.domainresellerapi.com/api/v1`.

## الاختبارات

لا تحتاج مجموعة الاختبارات إلى شبكة ولا إلى بيانات اعتماد:

```bash
cargo test
```

تحتوي على اختبارات وحدة لطبقة HTTP، وتحويل الأخطاء، وإخفاء بيانات القياس عن بُعد، والأدوات المساعدة، إضافةً إلى اختبار توافق يعيد تشغيل جميع حالات php-dna البالغ عددها 134 من `tests/conformance/` ويتحقق من أن كل طلب مُرسَل وكل قيمة مُعادة يطابقان php-dna.

## الإبلاغ عن الأخطاء (القياس عن بُعد)

عندما يفشل استدعاء API، يمكن للمكتبة إرسال تقرير خطأ إلى نظام تتبّع الأخطاء الخاص بـ DomainNameAPI (Sentry). وتُرسَل نسبة عشوائية صغيرة من الاستدعاءات الناجحة (4%، إضافةً إلى كل استدعاء يستغرق أكثر من ثانية) كسجل أداء. يساعد هذا DomainNameAPI على اكتشاف المشكلات وإصلاحها. الإبلاغ عن الأخطاء **مفعّل افتراضيًا**.

**ما الذي يُرسَل:**

- رمز الخطأ ورسالته
- مسار نقطة النهاية المستدعاة وطريقة HTTP
- مدة الاستدعاء
- اسم التطبيق (`DomainNameAPI-Rust`) وإصدار المكتبة
- معرّف الموزّع واسم النطاق
- عنوان IP العام للخادم (يُستعلم عنه في الخلفية عبر `api.ipify.org`)
- اسم مستخدم نظام التشغيل
- طلب API واستجابته، بما في ذلك بيانات جهات اتصال النطاق؛ تُخفى بيانات الاعتماد فقط — تُستبدل رموز EPP/auth ومفاتيح API وكلمات المرور والأسرار والرموز المميزة بـ `***`

**ما لا يُرسَل أبدًا:** رموز EPP/auth أو مفاتيح API أو كلمات المرور أو الرموز المميزة بصيغة مقروءة.

تُرسَل التقارير من خيط (thread) في الخلفية بمهل قصيرة؛ ولا تحجب استدعاءك أبدًا ولا تتسبب في فشله. ولا يُبلَّغ عن الأخطاء المتوقعة مثل "النطاق غير موجود" أو "الرصيد غير كافٍ".

**لماذا وإلى متى:** تُستخدم البيانات فقط لقياس الاستخدام والتشخيص المبكر للمشكلات، وتُحفظ لمدة 30 يومًا على خادم Sentry الخاص بـ DomainNameAPI، ثم تُحذف تلقائيًا.

**كيفية إيقافه أو تقليله:**

```rust
let mut dna = DomainNameApi::new(reseller_id, api_key, false);
dna.set_error_reporting_enabled(false); // nothing is sent

// or keep it on but send only the error code and message, endpoint,
// timing, versions and reseller ID (no request/response data, domain, IP or user name)
dna.set_telemetry_minimal(true);
```

> 🔒 **الأمان:** التحقق من شهادات TLS مفعّل. على خادم ذي حزمة CA قديمة يمكن تعطيله عبر `dna.set_ssl_verification(false);` — وهذا غير مُستحسن؛ تحديث حزمة CA هو الحل الأفضل.

## رموز العودة والأخطاء مع تفسيراتها

| الرمز | الوصف | التفاصيل |
|-------|--------|-----------|
| 1000 | تم تنفيذ الأمر بنجاح | العملية ناجحة |
| 1001 | تم تنفيذ الأمر بنجاح؛ الإجراء معلق | العملية ناجحة ولكنها في قائمة الانتظار للإكمال |
| 2003 | معلمة مطلوبة مفقودة | خطأ في المعلمات، مثل عدم إدخال رقم الهاتف في معلومات الاتصال |
| 2105 | الكائن غير مؤهل للتجديد | حالة النطاق غير مناسبة للتجديد، مقفلة للتحديثات |
| 2200 | خطأ في المصادقة | خطأ في التفويض، رمز الأمان خاطئ أو النطاق مسجل لدى شركة تسجيل أخرى |
| 2302 | الكائن موجود | اسم النطاق أو معلومات خادم الأسماء موجودة في قاعدة البيانات، لا يمكن التسجيل |
| 2303 | الكائن غير موجود | اسم النطاق أو معلومات خادم الأسماء غير موجودة في قاعدة البيانات، يجب إنشاء تسجيل جديد |
| 2304 | حالة الكائن تمنع العملية | حالة النطاق غير مناسبة للتحديث، مقفلة للتحديثات |

| الرمز | التفاصيل                                                                                                                               |
|------|---------------------------------------------------------------------------------------------------------------------------------------|
| 101  | خطأ النظام مفصل لـ ({0})!                                                                                                              |
| 102  | أخطاء متعددة مفصلة لـ ({0})!                                                                                                           |
| 103  | أخطاء غير معروفة مفصلة لـ ({0})!                                                                                                        |
| 200  | أمر API غير موجود ({0})!                                                                                                              |
| 210  | خدمة API غير موجودة ({0})!                                                                                                            |
| 211  | مزود خدمة API غير مُعيَّن!                                                                                                             |
| 300  | الموزع غير موجود!                                                                                                                     |
| 301  | عنوان IP الحالي الخاص بك غير مخول للوصول. تأكد من أنك تتصل من عنوان IP مخول وحاول مرة أخرى                                              |
| 310  | TLD غير مدعوم!                                                                                                                        |
| 320  | API غير موجود!                                                                                                                        |
| 321  | العملة غير مدعومة!                                                                                                                    |
| 330  | المعاملات المطلوبة غير مُعيَّنة ({0}).                                                                                                  |
| –    | تأكد من إرسال جميع كائنات الاتصال بالكامل                                                                                                |
| 340  | تعريف السعر غير موجود ({0}[{1}]-{2}{3}).                                                                                              |
| 350  | رصيد الموزع غير كافٍ. (معرف الموزع : {0} - الرصيد الحالي : {1} {2}).                                                                   |
| 350  | عملة المحاسبة غير متطابقة أو الرصيد غير كافٍ.                                                                                          |
| 360  | طلب API غير صالح للحقل ({0}).                                                                                                          |
| 360  | تم تجاوز حصة API!                                                                                                                     |
| 361  | خطأ في التحكم في المعدل!                                                                                                               |
| 362  | النطاق المميز غير متاح للتسجيل الآن.                                                                                                    |
| 363  | تم إلغاء العملية لأن النطاق في فترة التجديد الآلي                                                                                       |
| 364  | هذا النطاق غير متاح حاليًا للمعاملة بسبب مشكلة في السجل                                                                               |

| الرمز | التفاصيل                                                                                       |
|------|----------------------------------------------------------------------------------------------|
| 400  | اتصال غير صالح ({0}).                                                                          |
| 401  | لا يمكن مزامنة معلومات الاتصال.                                                                |
| 402  | لا يوجد وصول لمعلومات الاتصال من السجل.                                                        |
| 403  | النظام يحتوي على معلومات مفقودة. أدخل معلوماتك الافتراضية أو اتصل بفريق الدعم.                     |
| 404  | لا يمكن تحديث اتصال النطاق. إذن من السجل.                                                       |
| 410  | الاتصال غير موجود.                                                                            |
| 410  | الاتصال غير موجود ({0}).                                                                       |
| 420  | أمر Api غير صالح للاتصال {0}.                                                                 |
| 430  | API الاتصال غير موجود.                                                                        |
| 440  | الاتصال غير متزامن.                                                                           |
| 450  | اتصالات النطاق كثيرة جداً.. !                                                                  |
| 451  | فشل في المتابعة مع تحديث الاتصال.                                                              |

| الرمز | التفاصيل                                                                                                                               |
|------|---------------------------------------------------------------------------------------------------------------------------------------|
| 500  | معرف النطاق غير صالح.                                                                                                                 |
| 500  | معرف النطاق غير صالح ({0}).                                                                                                           |
| 501  | لا يمكن مزامنة النطاق ({0})                                                                                                          |
| 502  | فشل النقل الداخلي                                                                                                                    |
| 503  | تسجيل النطاق غير متاح.                                                                                                               |
| 504  | معلومات النطاق غير متطابقة .. قبل :{0}                                                                                               |
| 505  | أدخل عنوان IP.                                                                                                                       |
| 506  | تم بدء نقل النطاق لكن لا يمكن قراءة معلومات الاتصال ..                                                                              |
| 507  | خطأ في التفويض.                                                                                                                      |
| 510  | النطاق غير موجود.                                                                                                                    |
| 511  | لا يمكن العثور على النطاقات المنتهية الصلاحية.                                                                                        |
| 512  | النطاق غير قابل للتجديد.                                                                                                             |
| 513  | النطاق ليس في حالة قابلة للتحديث. يجب أن يكون نشطاً ليتم تحديثه                                                                      |
| 514  | فترة الاسترداد متوقعة.                                                                                                               |
| 520  | أمر Api غير صالح للنطاق "{0}".                                                                                                       |
| 530  | فترة النطاق غير صالحة. يجب أن تكون الفترة من {0} إلى {1} سنة. الفترة المطلوبة هي {2} سنة.                                         |
| 540  | لا يمكن تمديد النطاق إلى ما بعد {0} سنة من التاريخ الحالي.                                                                          |
| 550  | اسم النطاق غير صالح. يجب أن يكون النطاق من {0} إلى {1} حرف. اسم النطاق المطلوب هو {2} حرف.                                     |
| 560  | عدد خوادم الأسماء غير صالح. يجب أن يحتوي النطاق على {0} إلى {1} خادم أسماء. عدد خوادم الأسماء المطلوب هو {2}.                    |
| 561  | لم يتم العثور على معلومات خادم الأسماء في الطلب الوارد                                                                             |
| 570  | Idn غير مدعوم لـ TLD "{0}".                                                                                                          |
| 571  | الفترة غير صالحة.                                                                                                                   |
| 572  | الأمر غير صالح.                                                                                                                     |
| 573  | أسماء النطاقات غير موجودة.                                                                                                           |
| 574  | أسماء TLD غير موجودة.                                                                                                               |
| 575  | النطاق ليس في حالة قابلة للتحديث. يجب أن يكون نشطاً لتحديث خوادم الأسماء                                                            |
| 576  | يمكن تجديد النطاق في الشهر الأخير قبل تاريخ انتهاء الصلاحية                                                                        |
| 580  | النقل غير مدعوم لـ TLD "{0}".                                                                                                        |
| 581  | خادم الأسماء الفرعي غير موجود                                                                                                        |
| 582  | تم بدء النقل بواسطة موزع آخر.                                                                                                        |
| 583  | النقل غير مُعيَّن. يرجى الاتصال بفريق الدعم.                                                                                         |
| 584  | حالة الكائن تمنع العملية                                                                                                             |
| 590  | رمز التفويض مطلوب لهذا النقل.                                                                                                        |
| 591  | رمز التفويض غير صالح.                                                                                                               |
| 592  | قفل النقل موجود على النطاق.                                                                                                          |
| 593  | لا يمكن نقل النطاق، معلومات الحالة وقفل النقل يجب أن تتوافق مع المعايير المطلوبة. (معلومات الحالة: #ok)                           |
| 594  | لا يمكن حل عنوان خادم أسماء النطاق ({0})                                                                                            |
| 595  | لا يمكن قراءة معلومات الاتصال (whois.registrar.tld) تأكد من أن حالة حماية الخصوصية مفتوحة                                          |
| 596  | لا يمكن التحقق من معلومات الاتصال                                                                                                   |
| 597  | TLD غير موجود                                                                                                                       |
| 598  | حدث خطأ غير معروف                                                                                                                   |
| 599  | إعادة توجيه النطاق غير موجود                                                                                                          |

## الترخيص

رخصة Apache 2.0 — راجع [LICENSE](LICENSE).
