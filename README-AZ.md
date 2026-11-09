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

# rust-dna — Rust üçün DomainNameAPI müştərisi

DomainNameAPI domen qeydiyyatçısının REST API-si üçün Rust müştərisi. Bu, istinad PHP kitabxanası [php-dna](https://github.com/domainreseller/php-dna)-nın sadiq portudur: eyni metodlar, şəbəkəyə gedən eyni sorğular və eyni qaytarılan dəyərlər. Hər port php-dna-nın 126 uyğunluq (conformance) ssenarisi ilə yoxlanılır.

## 📦 Yükləmə — həmişə Releases istifadə edin!

⬇️ **Ən son sınaqdan keçmiş versiyanı buradan əldə edin: https://github.com/domainreseller/rust-dna/releases/latest**

> ⚠️ Yaşıl **Code → Download ZIP** düyməsindən **istifadə etməyin** — o, xam inkişaf budağını yükləyir. Release paketləri versiyalanmış, sınaqdan keçmiş və istifadəyə hazırdır.

## Quraşdırma və İnteqrasiya Təlimatı

### Minimum Tələblər

- Rust 1.83 və ya daha yeni (edition 2021)
- Async icra mühiti tələb olunmur: API bloklayıcıdır (blocking)
- İcra asılılıqları: `ureq` (HTTP, rustls TLS), `rustls` və `serde_json`. OpenSSL lazım deyil.

### Quraşdırma

**A) Git asılılığı (release etiketinə bağlanmış)**

```toml
[dependencies]
domainnameapi = { git = "https://github.com/domainreseller/rust-dna", tag = "v1.0.0" }
```

**B) Yüklənmiş release**

Release arxivini layihənizin yanında açın və Cargo-nu ona yönləndirin:

```toml
[dependencies]
domainnameapi = { path = "../rust-dna" }
```

Crate `serde_json::Value` tipini və `json!` makrosunu yenidən ixrac edir, ona görə `serde_json`-u özünüz əlavə etməyiniz lazım deyil (istəsəniz əlavə edə bilərsiniz).

## 🔑 API Məlumatları — Reseller ID və API Key

Kitabxana DomainNameAPI REST API-si ilə əlaqə saxlayır və konstruktora ötürülən dəqiq iki giriş məlumatı tələb edir:

| Parametr | Dəyər |
|---|---|
| Birinci | Reseller ID — `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` formatında UUID |
| İkinci | API Key |

> 💡 **Reseller ID** və **API Key** məlumatlarınızı DomainNameAPI panelinizdə **API Ayarları** bölməsində tapa bilərsiniz.
> ⚠️ Bunlar **API giriş məlumatlarıdır** — panelə giriş məlumatlarınız burada **işləmir**.

```rust
use domainnameapi::DomainNameApi;

let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";
let api_key = "your-api-key";

// Live environment
let mut dna = DomainNameApi::new(reseller_id, api_key, false);

// Test (OTE) environment
let mut dna = DomainNameApi::new(reseller_id, api_key, true);
```

Hər iki məlumatın əvvəlindəki və sonundakı boşluqlar silinir. Panellər seçim qutularını tez-tez mətn kimi saxladığı üçün üçüncü arqument mətn də qəbul edir: `"true"`, `"1"`, `"on"` və `"yes"` (böyük/kiçik hərfdən asılı olmayaraq) test mühitini seçir, qalan hər şey (`"false"` və `"off"` daxil olmaqla) canlı mühiti seçir.

## Sürətli başlanğıc

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

[examples/](examples) qovluğundakı hər nümunə `cargo run --example <ad>` ilə işə düşür, məsələn `cargo run --example get_details`.

## API-nin quruluşu

- **Tək müştəri tipi**: `DomainNameApi`. Metodlar `&mut self` qəbul edir (müştəri diaqnostika üçün son sorğunu yadda saxlayır).
- **Qaytarılan dəyərlər `serde_json::Value`-dur** və php-dna-nın qaytardığı strukturla tam eynidir — tipli modellər deyil. Açar adları, mətn bayraqları (`"true"` / `"false"`), formatlanmış məbləğlər (`"11.9900"`) və zərflər eynidir:
  - uğur: `{"result": "OK", "data": {...}}`
  - xəta: `{"result": "ERROR", "error": {"Code": "...", "Message": "...", "Details": "..."}}`
  - bir neçə metod php-dna-dan götürülmüş başqa zərf istifadə edir: `get_current_balance` `OperationResult` obyekti qaytarır, `check_availability` uğur halında sadə siyahı qaytarır, `get_reseller_details` sahələrini ən üst səviyyədə qaytarır.
- **Xətalar atılmır, qaytarılır.** Gateway xətaları, HTTP xətaları və şəbəkə nasazlıqlarının hamısı xəta zərfi kimi qayıdır; heç nə panic yaratmır.
- **Strukturlaşdırılmış girişlər** (kontaktlar, nameserver siyahıları, filtrlər, TLD atributları, qeyd məzmunları) `json!` ilə yaradılan `&Value` dəyərləridir. Kontakt xəritələri php-dna-nın açarlarından istifadə edir (`FirstName`, `EMail`, `AddressLine1`, `PhoneCountryCode`, ...); həm düz, həm də iç-içə (`Address.Line1`, `Phone.Phone.Number`) kontakt formaları qəbul olunur. Nameserver siyahıları siyahı və ya xəritə ola bilər (dəyərlər istifadə olunur).
- **Rust-da standart arqumentlər yoxdur.** Hər metod php-dna-nın tam arqument siyahısını eyni ardıcıllıqla qəbul edir. php-dna-nın standart dəyərləri sabitlər kimi ixrac olunur: `DEFAULT_TTL` (3600), `DEFAULT_REASON` ("Owner request"), `DEFAULT_FORWARD_TYPE` ("Standard"), `DEFAULT_CURRENCY` ("USD"), `DEFAULT_TLD_COUNT` (20), `DEFAULT_COMMAND` ("create"). Dörd isteğe bağlı arqumenti olan `register_with_contact_info` üçün əlavə olaraq `Default` dəyəri php-dna-nın standartları olan `RegisterOptions` strukturunu qəbul edən `register_with_contact_info_opts` də var.

## İstifadə

Bütün nümunələr `use domainnameapi::{json, DomainNameApi};` sətrini və `let mut dna = DomainNameApi::new(...)` ilə yaradılmış müştərini nəzərdə tutur.

#### Domen Qeydiyyatı

Qeyd: .tr domenləri üçün əlavə parametrlər tələb olunur. Əlavə atributlar arqumenti .tr kimi əlavə məlumat tələb edən domen adları üçün istifadə olunur.

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

`.tr` domeni üçün TRABIS atributlarını ötürün. Kitabxana yalnız gateway-in qəbul etdiklərini göndərir (`TRABISDOMAINCATEGORY`, `TRABISCITIZENID`, `TRABISTAXOFFICE`, `TRABISTAXNUMBER`), `1`/`0` dəyərlərini `Owner`/`Corporate`-a çevirir və köhnə `TRABISCITIZIENID` yazılışını düzəldir:

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
<summary>Domen Qeydiyyatı üçün Nümunə Çıxış</summary>

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

#### Domen Yeniləmə

```rust
let result = dna.renew("example.com", 1);
```

<details>
<summary>Domen Yeniləmə üçün Nümunə Çıxış</summary>

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

#### Domen Transferi

```rust
let result = dna.transfer("example.com", "Xy9#mK2$", 3);

// php-dna's facade sends no contacts with a transfer; to send them:
let result = dna.transfer_with_contacts("example.com", "Xy9#mK2$", 3, &contacts);
```

<details>
<summary>Domen Transferi üçün Nümunə Çıxış</summary>

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

#### Transfer İdarəetməsi

```rust
// Can the domain be transferred with this auth code?
let check = dna.check_transfer("example.com", "Xy9#mK2$");

// Cancel an incoming transfer, approve or reject an outgoing one
let result = dna.cancel_transfer("example.com");
let result = dna.approve_transfer("example.com");
let result = dna.reject_transfer("example.com");
```

<details>
<summary>Transfer İdarəetməsi üçün Nümunə Çıxış</summary>

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

`check_transfer` reyestr domenin transfer edilə bilmədiyini bildirdikdə `"result": "ERROR"` və `error` (`TRANSFER_NOT_AVAILABLE`) qaytarır; `data` hər iki halda doldurulur. `cancel_transfer`, `approve_transfer` və `reject_transfer` bunu qaytarır:

```json
{"result": "OK", "data": {"DomainName": "example.com", "Status": "Cancelled"}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Domen Siyahısı

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

`PageNumber` 0-dan başlayır. Gateway-in öz `MaxResultCount`, `SkipCount` və `Sorting` parametrləri, həmçinin `DomainName` kimi filtrlər də ötürülə bilər.

<details>
<summary>Domen Siyahısı üçün Nümunə Çıxış</summary>

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

#### TLD Siyahısı

```rust
let result = dna.get_tld_list(100);
```

<details>
<summary>TLD Siyahısı üçün Nümunə Çıxış</summary>

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

#### Domen Mövcudluğunun Yoxlanması

```rust
let result = dna.check_availability(["hello", "world123x0"], ["com", "net"], 1, "create");
```

<details>
<summary>Domen Mövcudluğunun Yoxlanması üçün Nümunə Çıxış</summary>

Uğur halında nəticə sadə siyahıdır (`result` açarı yoxdur); xəta halında isə xəta zərfidir.

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

#### Domen Təfərrüatları

```rust
let result = dna.get_details("example.com");

// Re-read the domain from the registry (same result shape):
let result = dna.sync_from_registry("example.com");
```

<details>
<summary>Domen Təfərrüatları üçün Nümunə Çıxış</summary>

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

#### Nameserver Dəyişdirilməsi

```rust
let result = dna.modify_name_server("example.com", &json!(["ns1.example.com", "ns2.example.com"]));
```

<details>
<summary>Nameserver Dəyişdirilməsi üçün Nümunə Çıxış</summary>

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

#### Domen Kilidini Aktivləşdirmə

```rust
let result = dna.enable_theft_protection_lock("example.com");
```

<details>
<summary>Domen Kilidini Aktivləşdirmə üçün Nümunə Çıxış</summary>

```json
{"result": "OK", "data": {"LockStatus": true}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Domen Kilidini Deaktivləşdirmə

```rust
let result = dna.disable_theft_protection_lock("example.com");
```

<details>
<summary>Domen Kilidini Deaktivləşdirmə üçün Nümunə Çıxış</summary>

```json
{"result": "OK", "data": {"LockStatus": false}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Child Nameserver Əlavə Etmə

```rust
let result = dna.add_child_name_server("example.com", "ns1.example.com", "1.2.3.4");
```

<details>
<summary>Child Nameserver Əlavə Etmə üçün Nümunə Çıxış</summary>

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

#### Child Nameserver Silmə

```rust
let result = dna.delete_child_name_server("example.com", "ns1.example.com");
```

<details>
<summary>Child Nameserver Silmə üçün Nümunə Çıxış</summary>

```json
{"result": "OK", "data": {"NameServer": "ns1.example.com"}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Child Nameserver Yeniləmə

```rust
let result = dna.modify_child_name_server("example.com", "ns1.example.com", "1.2.3.4");
```

<details>
<summary>Child Nameserver Yeniləmə üçün Nümunə Çıxış</summary>

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

#### Domen Məxfiliyinin Dəyişdirilməsi

```rust
let result = dna.modify_privacy_protection_status("example.com", true, "Owner request");
```

<details>
<summary>Domen Məxfiliyinin Dəyişdirilməsi üçün Nümunə Çıxış</summary>

```json
{"data": {"PrivacyProtectionStatus": true}, "result": "OK"}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Domen Kontaktlarının Alınması

```rust
let result = dna.get_contacts("example.com");
```

<details>
<summary>Domen Kontaktlarının Alınması üçün Nümunə Çıxış</summary>

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

#### Domen Kontaktlarının Saxlanması

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
<summary>Domen Kontaktlarının Saxlanması üçün Nümunə Çıxış</summary>

```json
{"result": "OK", "data": {"contacts": []}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### DNS Zona Qeydləri

Gateway qeydləri **dəstlər** şəklində saxlayır (bir və ya bir neçə dəyər daşıyan bir ad + tip). Dəsti yazmaq onun bütün dəyərlərini əvəz edir, istənilən dəyəri silmək isə bütün dəsti silir. `*_value` metodları oxu-birləşdir-yaz əməliyyatını sizin əvəzinizə edir ki, panel qeydləri bir-bir idarə edə bilsin. Adlar nisbi (`www`), apex üçün `@` / boş və ya tam ixtisaslı ola bilər; onlar gateway-in gözlədiyi nisbi etiketə endirilir. Dəstəklənən tiplər: A, AAAA, ALIAS, CAA, CNAME, DS, MX, NAPTR, PTR, SRV, TLSA, TXT. `apply` `true` olduqda (php-dna standartı) dəyişikliklər dərhal tətbiq olunur; əks halda `apply_resource_records` çağırın. Gateway sorğu partlayışlarını məhdudlaşdırdığı üçün zona çağırışları arasında ən azı bir saniyə saxlanılır.

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
<summary>DNS Zona Qeydləri üçün Nümunə Çıxış</summary>

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

Artıq mövcud olan (silmədə isə mövcud olmayan) dəyər heç nəyi dəyişmir:

```json
{"result": "OK", "data": {"Name": "", "Type": "MX", "Contents": ["10 mx1.example.com."], "Changed": false}}
```

Dəstəklənməyən tip heç bir sorğu edilmədən rədd olunur:

```json
{"result": "ERROR", "error": {"Code": "ADD_ZONE", "Message": "Record type \"NS\" is not supported by the DNS zone API. Supported types: A, AAAA, ALIAS, CAA, CNAME, DS, MX, NAPTR, PTR, SRV, TLSA, TXT.", "Details": "..."}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### URL Yönləndirmə

```rust
let current = dna.get_forwarding("example.com");
let result = dna.set_forwarding("example.com", "https://www.example.org", "Standard"); // or "Frame" (masked)
let result = dna.delete_forwarding("example.com");
```

<details>
<summary>URL Yönləndirmə üçün Nümunə Çıxış</summary>

```json
{"result": "OK", "data": {"Enabled": true, "RedirectAddress": "https://www.example.org/landing", "ForwardType": "Frame"}}
```

Yönləndirmə qurulmayıbsa, `get_forwarding` `{"result": "OK", "data": {"Enabled": false, "RedirectAddress": "", "ForwardType": ""}}` qaytarır. `set_forwarding` `{"result": "OK", "data": {"RedirectAddress": "...", "ForwardType": "Standard"}}` qaytarır.

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Reseller Məlumatlarının Alınması

```rust
let result = dna.get_reseller_details();
let balance = dna.get_current_balance("USD"); // "USD", "TRY", "EUR" or "GBP"
```

<details>
<summary>Reseller Məlumatları üçün Nümunə Çıxış</summary>

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
    {"balance": "1250.5", "currency": "TL", "symbol": "TL"}
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

Xəta halında `OperationResult` `"FAILED"` olur, `ErrorCode` kodu saxlayır (məs. `"API_429"`) və `Balance` `"0.00"` olur.

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Köməkçi funksiyalar (şəbəkəsiz)

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
<summary>validate_contact üçün Nümunə Çıxış</summary>

```json
{
  "FirstName": "Jo",
  "Phone": "12025550100",
  "Country": "TR",
  "LastName": "Isimyok",
  "AddressLine1": "Addres yok",
  "City": "ISTANBUL",
  "State": "ISTANBUL",
  "ZipCode": "34000",
  "PhoneCountryCode": "90",
  "Fax": "",
  "FaxCountryCode": ""
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

## Xətaların idarə edilməsi

Hər şəbəkə metodu bir dəyər qaytarır; `result` sahəsini yoxlayın:

```rust
let out = dna.get_details("missing.com");
if out["result"] == "ERROR" {
    println!("{}", out["error"]["Code"]);    // "API_404"
    println!("{}", out["error"]["Message"]); // "The domain name you requested could not be found. ..."
    println!("{}", out["error"]["Details"]);
}
```

- HTTP xətaları `API_<status>` kodundan istifadə edir (`API_400`, `API_404`, `API_429`, `API_500`, ...). Boş xəta gövdəsi `HTTP 429 - Too Many Requests` kimi oxunaqlı mesaja çevrilir.
- 301/302 cavabı səhv giriş məlumatları deməkdir: `Invalid API credentials`.
- Gateway-in doğrulama xətaları sahənin adı göstərilməklə `Details` daxilində birləşdirilir, məs. `Contacts Phone - The field Phone must be a string with a maximum length of 16.`
- Şəbəkə nasazlıqları xətadır, heç vaxt boş uğur deyil. Vaxt aşımı `Could not reach the DomainNameAPI service: the connection timed out. (...)` kimi oxunur; digər nasazlıqlar `Curl error during request: ...` kimi oxunur. Zərf kodu `API_<curl xəta nömrəsi>`-dir (vaxt aşımı üçün `API_28`, DNS üçün `API_6`, rədd edilmiş bağlantı üçün `API_7`), `get_parsed_response_data()["Code"]` isə `CURL_<n>` dəyərini saxlayır.
- Yerli doğrulama (dəstəklənməyən qeyd tipi, çatışmayan yönləndirmə ünvanı, yoxlanılacaq domen olmaması) heç bir sorğudan əvvəl izahlı kodla uğursuz olur (`ADD_ZONE`, `SET_FORWARD`, `AVAILABILITY`, ...).

### Diaqnostika

```rust
dna.get_request_data();        // {"url", "method", "payload"} of the last request
dna.get_response_data();       // decoded last response, or {"raw_response": "<body>"}
dna.get_response_headers();    // {"url", "content_type", "http_code", "total_time", "headers"}
dna.get_parsed_response_data();// parsed body on success, {"Code", "Message", "Details"} on failure
dna.get_last_function();       // e.g. "request:GET domains/info"
dna.get_service_url();
dna.set_service_url("https://ote.domainresellerapi.com/api/v1");
```

### Xüsusi transport

HTTP qatı `Transport` trait-idir. Standart olaraq `UreqTransport` istifadə olunur (rustls ilə bloklayıcı `ureq`, ümumi 30 s və bağlantı üçün 10 s vaxt aşımı, yönləndirmələr izlənmir). Proksi, xüsusi müştəri və ya testlər üçün onu əvəz edin:

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

## Test rejimi

`https://ote.domainresellerapi.com/api/v1` ünvanındakı OTE test mühitindən istifadə etmək üçün konstruktorun üçüncü arqumenti kimi `true` (və ya `"true"`, `"1"`, `"on"`, `"yes"`) ötürün. Canlı çağırışlar `https://api.domainresellerapi.com/api/v1` ünvanına gedir.

## Testlər

Test paketi nə şəbəkə, nə də giriş məlumatları tələb edir:

```bash
cargo test
```

O, HTTP qatı, xətaların uyğunlaşdırılması, telemetriya maskalanması və köməkçi funksiyalar üçün vahid testləri, həmçinin `tests/conformance/` qovluğundakı 126 php-dna ssenarisinin hamısını yenidən oynadan və şəbəkəyə gedən hər sorğunun və hər qaytarılan dəyərin php-dna ilə eyni olduğunu yoxlayan uyğunluq testini ehtiva edir.

## Xəta hesabatı (telemetriya)

API çağırışı uğursuz olduqda kitabxana DomainNameAPI-nin öz xəta izləmə sisteminə (Sentry) xəta hesabatı göndərə bilər. Uğurlu çağırışların kiçik təsadüfi hissəsi (4%, üstəlik 1 saniyədən yavaş hər çağırış) performans qeydi kimi göndərilir. Bu, DomainNameAPI-yə problemləri tapıb düzəltməyə kömək edir. Xəta hesabatı **standart olaraq aktivdir**.

> ℹ️ 1.0.0 versiyasında Rust kitabxanası üçün Sentry layihəsi hələ yaradılmayıb, ona görə daxili DSN boşdur və heç nə göndərilmir. Aşağıdakı davranış DSN quraşdırıldıqdan sonra (və ya `set_telemetry_dsn` ilə özünüz DSN təyin etdikdə) qüvvəyə minir.

**Nə göndərilir:**

- xəta kodu və mesajı
- çağırılan son nöqtənin yolu və HTTP metodu
- çağırışın müddəti
- tətbiqin adı (`DomainNameAPI-Rust`) və kitabxananın versiyası
- reseller ID və domen adı
- serverin ictimai IP ünvanı (arxa planda `api.ipify.org` vasitəsilə müəyyən edilir)
- əməliyyat sisteminin istifadəçi adı
- domenin kontakt məlumatları daxil olmaqla API sorğusu və cavabı; yalnız giriş məlumatları maskalanır — EPP/auth kodları, API açarları, parollar, gizli açarlar və tokenlər `***` ilə əvəz olunur

**Heç vaxt göndərilməyənlər:** oxunaqlı formada EPP/auth kodları, API açarları, parollar və ya tokenlər.

Hesabatlar qısa vaxt aşımları ilə arxa plan axınından göndərilir; onlar heç vaxt çağırışınızı bloklamır və uğursuz etmir. "Domen tapılmadı" və ya "kifayət etməyən balans" kimi gözlənilən xətalar bildirilmir.

**Niyə və nə qədər müddət:** məlumatlar yalnız istifadənin ölçülməsi və problemlərin erkən diaqnostikası üçün istifadə olunur və DomainNameAPI-nin öz Sentry serverində 30 gün saxlanılır, sonra avtomatik silinir.

**Necə söndürmək və ya azaltmaq olar:**

```rust
let mut dna = DomainNameApi::new(reseller_id, api_key, false);
dna.set_error_reporting_enabled(false); // nothing is sent

// or keep it on but send only the error code and message, endpoint,
// timing, versions and reseller ID (no request/response data, domain, IP or user name)
dna.set_telemetry_minimal(true);
```

> 🔒 **Təhlükəsizlik:** TLS sertifikatının yoxlanması aktivdir. Köhnə CA paketi olan hostda `dna.set_ssl_verification(false);` ilə söndürülə bilər — bu tövsiyə olunmur; CA paketini yeniləmək daha yaxşı həlldir.

## Geri Dönüş və Xəta Kodları ilə Açıqlamaları

| Kod | Açıqlama | Təfərrüat |
|-----|----------|-----------|
| 1000 | Əmr uğurla tamamlandı | Əməliyyat uğurlu. |
| 1001 | Əmr uğurla tamamlandı; hərəkət gözləmədə | Əməliyyat uğurlu. Bununla belə, əməliyyat hazırda tamamlanması üçün növbəyə alınıb. |
| 2003 | Tələb olunan parametr çatışmır | Parametr çatışmır xətası. Məsələn: əlaqə məlumatlarında telefon girişi yox. |
| 2105 | Obyekt yenilənmə üçün uyğun deyil | Domen statusu yenilənmə üçün uyğun deyil, yeniləmə əməliyyatları üçün kilidlənib. Status "clientupdateprohibited" olmamalıdır. Digər status şərtlərindən qaynaqlanana bilər. |
| 2200 | Kimlik doğrulama xətası | İcazə xətası, təhlükəsizlik kodu səhvdir və ya domen başqa qeydiyyat şirkətində mövcuddur. |
| 2302 | Obyekt mövcuddur | Domen adı və ya nameserver məlumatları verilənlər bazasında mövcuddur. Qeydiyyat edilə bilməz. |
| 2303 | Obyekt mövcud deyil | Domen adı və ya nameserver məlumatları verilənlər bazasında mövcud deyil. Yeni qeydiyyat tələb olunur. |
| 2304 | Obyekt statusu əməliyyatı qadağan edir | Domen statusu yeniləmələr üçün uyğun deyil, yeniləmə əməliyyatları üçün kilidlənib. Status "clientupdateprohibited" olmamalıdır. Digər status şərtlərindən qaynaqlanana bilər. |

| KOD | TƏFƏRRÜAT |
|-----|-----------|
| 101 | Sistem Xətası Təfərrüatlı ({0}) Üçün! |
| 102 | Çoxlu Xətalar Təfərrüatlı ({0}) Üçün! |
| 103 | Naməlum Xətalar Təfərrüatlı ({0}) Üçün! |
| 200 | API əmri tapılmadı ({0})! |
| 210 | API xidməti tapılmadı ({0})! |
| 211 | API xidmət provayderi təyin edilməyib! |
| 300 | Satıcı tapılmadı! |
| 301 | Hazırkı IP ünvanınız girişə icazəli deyil. İcazəli IP ünvanından qoşulduğunuzdan əmin olun və yenidən cəhd edin |
| 310 | TLD dəstəklənmir! |
| 320 | API tapılmadı! |
| 321 | Valyuta dəstəklənmir! |
| 330 | Tələb olunan parametr(lər) təyin edilməyib ({0}). |
| – | Bütün əlaqə obyektlərini tam göndərdiyinizə əmin olun |
| 340 | Qiymət tərifə tapılmadı ({0}[{1}]-{2}{3}). |
| 350 | Kifayət qədər satıcı balansı yoxdur. (Satıcı Id : {0} - Hazırkı Balans : {1} {2}). |
| 350 | Mühasibat valyutası uyğunlaşmır və ya balans kifayət deyil. |
| 360 | Sahə ({0}) üçün etibarsız API sorğusu. |
| 360 | API kvotası aşılıb! |
| 361 | Throttled xətası! |
| 362 | Premium domen hazırda qeydiyyat üçün əlçatan deyil. |
| 363 | Domen avtomatik yenilənmə dövründə olduğu üçün əməliyyat ləğv edildi |
| 364 | Bu domen hazırda qeydiyyat ilə bağlı problem səbəbindən əməliyyat üçün əlçatan deyil |

| KOD | TƏFƏRRÜAT |
|-----|-----------|
| 400 | Etibarsız əlaqə ({0}). |
| 401 | Əlaqə məlumatı sinxronlaşdırıla bilmir. |
| 402 | Qeydiyyat dəftərindən əlaqə məlumatlarına giriş yoxdur. |
| 403 | SİSTEMDƏ ÇATIŞMAYAN MƏLUMAT VAR. STANDART MƏLUMATLARINIZI DAXİL EDİN VƏ YA DƏSTƏK KOMANDASI İLƏ ƏLAQƏ SAXLAYIN. |
| 404 | DOMEN ƏLAQƏSI YENİLƏNƏ BİLMİR. QEYDİYYAT DƏFTƏRİNDƏN İCAZƏ. |
| 410 | Əlaqə tapılmadı. |
| 410 | Əlaqə tapılmadı ({0}). |
| 420 | Əlaqə {0} üçün etibarsız Api əmri. |
| 430 | Əlaqə api-si tapılmadı. |
| 440 | Əlaqə sinxronlaşdırılmayıb. |
| 450 | Çox çoxlu domen əlaqələri.. ! |
| 451 | Əlaqə yeniləməsi ilə davam edilə bilmədi. |

| KOD | TƏFƏRRÜAT |
|-----|-----------|
| 500 | Etibarsız domen id. |
| 500 | Etibarsız domen id ({0}). |
| 501 | Domen sinxronlaşdırıla bilmədi({0}) |
| 502 | Daxili transfer uğursuz oldu |
| 503 | Domen qeydiyyatı əlçatan deyil. |
| 504 | Domen məlumatları uyğunlaşmır .. Əvvəl :{0} |
| 505 | IP Ünvanını daxil edin. |
| 506 | Domen Transferi Başladıldı Lakin Əlaqə Məlumatı Oxuna Bilmədi .. |
| 507 | İcazə xətası. |
| 510 | Domen tapılmadı. |
| 511 | Vaxtı keçmiş domenlər tapıla bilmir. |
| 512 | Domen yenilənə bilmir. |
| 513 | Domen yenilənə bilən statusda deyil. Yenilənə bilməsi üçün aktiv olmalıdır |
| 514 | Bərpa Dövrü Gözlənilir. |
| 520 | Domen "{0}" üçün etibarsız Api əmri. |
| 530 | Etibarsız domen müddəti. Müddət {0} ilə {1} il arasında olmalıdır. Tələb olunan müddət {2} ildir. |
| 540 | Domen hazırkı tarixdən {0} ildən artıq uzadıla bilməz. |
| 550 | Etibarsız domen adı. Domen {0} ilə {1} simvol uzunluğunda olmalıdır. Tələb olunan domen adı {2} simvoldur. |
| 560 | Etibarsız nameserver sayı. Domen {0} ilə {1} nameserverə sahib olmalıdır. Tələb olunan nameserver sayı {2}. |
| 561 | Daxil olan sorğuda nameserver məlumatı tapılmadı |
| 570 | Idn tld "{0}" üçün dəstəklənmir. |
| 571 | Müddət etibarsızdır. |
| 572 | Əmr etibarsızdır. |
| 573 | Domen adları tapılmadı. |
| 574 | TLD adları tapılmadı. |
| 575 | Domen yenilənə bilən statusda deyil. Nameserverlər yenilənə bilməsi üçün aktiv olmalıdır |
| 576 | Domen son istifadə tarixindən əvvəlki son 1 ay ərzində yenilənə bilər |
| 580 | Transfer tld "{0}" üçün dəstəklənmir. |
| 581 | Uşaq nameserveri tapılmadı |
| 582 | Transfer başqa satıcı tərəfindən başladıldı. |
| 583 | Transfer başladılmadı. Dəstək komandası ilə əlaqə saxlayın. |
| 584 | Obyekt statusu əməliyyatı qadağan edir |
| 590 | Bu transfer üçün auth kodu tələb olunur. |
| 591 | Auth kodu etibarsızdır. |
| 592 | Domen üzərində transfer kilidi mövcuddur. |
| 593 | Domen transfer edilə bilməz, Status Məlumatı və Transfer Kilidi tələb olunan kriterlərə uyğun gəlməlidir. (Status Məlumatı: #ok) |
| 594 | Domen nameserver ünvanı həll edilə bilmədi ({0}) |
| 595 | Əlaqə məlumatı oxuna bilmir (whois.registrar.tld) Məxfilik qoruma statusunun açıq olduğundan əmin olun |
| 596 | Əlaqə məlumatı təsdiqlənə bilmədi |
| 597 | Tld Tapılmadı |
| 598 | Naməlum xəta baş verdi |
| 599 | Domen Yönləndirməsi Tapılmadı |

## Lisenziya

Apache License 2.0 — bax: [LICENSE](LICENSE).
