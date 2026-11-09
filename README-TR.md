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

# rust-dna — Rust için DomainNameAPI istemcisi

DomainNameAPI alan adı kayıt REST API'si için Rust istemcisi. Referans PHP kütüphanesi [php-dna](https://github.com/domainreseller/php-dna)'nın birebir bir uyarlamasıdır: aynı metotlar, ağa giden aynı istekler ve aynı dönüş değerleri. Her uyarlama php-dna'nın 126 uyumluluk (conformance) senaryosuna karşı doğrulanır.

## 📦 İndirme — her zaman Releases kullanın!

⬇️ **En son test edilmiş sürümü buradan alın: https://github.com/domainreseller/rust-dna/releases/latest**

> ⚠️ Yeşil **Code → Download ZIP** düğmesini **kullanmayın** — bu, ham geliştirme dalını indirir. Release paketleri sürümlenmiş, test edilmiş ve canlı kullanıma hazırdır.

## Kurulum ve Entegrasyon Rehberi

### Minimum Gereksinimler

- Rust 1.83 veya daha yeni (edition 2021)
- Async çalışma zamanı gerekmez: API engelleyici (blocking) çalışır
- Çalışma zamanı bağımlılıkları: `ureq` (HTTP, rustls TLS), `rustls` ve `serde_json`. OpenSSL gerekmez.

### Kurulum

**A) Git bağımlılığı (bir release etiketine sabitlenmiş)**

```toml
[dependencies]
domainnameapi = { git = "https://github.com/domainreseller/rust-dna", tag = "v1.0.0" }
```

**B) İndirilen release paketi**

Release arşivini projenizin yanına açın ve Cargo'yu ona yönlendirin:

```toml
[dependencies]
domainnameapi = { path = "../rust-dna" }
```

Crate, `serde_json::Value` tipini ve `json!` makrosunu yeniden dışa aktarır; bu yüzden `serde_json`'u ayrıca eklemeniz gerekmez (isterseniz ekleyebilirsiniz).

## 🔑 API Bilgileri — Reseller ID ve API Key

Kütüphane DomainNameAPI REST API'si ile konuşur ve yapıcıya (constructor) verilen tam olarak iki kimlik bilgisine ihtiyaç duyar:

| Parametre | Değer |
|---|---|
| Birinci | Reseller ID — `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` biçiminde UUID |
| İkinci | API Key |

> 💡 **Reseller ID** ve **API Key** bilgilerinizi DomainNameAPI panelinizde **API Ayarları** bölümünde bulabilirsiniz.
> ⚠️ Bunlar **API kimlik bilgileridir** — panel giriş bilgileriniz burada **çalışmaz**.

```rust
use domainnameapi::DomainNameApi;

let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";
let api_key = "your-api-key";

// Live environment
let mut dna = DomainNameApi::new(reseller_id, api_key, false);

// Test (OTE) environment
let mut dna = DomainNameApi::new(reseller_id, api_key, true);
```

İki kimlik bilgisinin de başındaki ve sonundaki boşluklar temizlenir. Paneller onay kutularını çoğu zaman metin olarak sakladığı için üçüncü argüman metin de kabul eder: `"true"`, `"1"`, `"on"` ve `"yes"` (büyük/küçük harf duyarsız) test ortamını seçer; diğer her değer (`"false"` ve `"off"` dahil) canlı ortamı seçer.

## Hızlı başlangıç

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

[examples/](examples) klasöründeki her örnek `cargo run --example <ad>` ile çalışır, örneğin `cargo run --example get_details`.

## API'nin yapısı

- **Tek bir istemci tipi**: `DomainNameApi`. Metotlar `&mut self` alır (istemci, tanılama için son isteği hatırlar).
- **Dönüş değerleri `serde_json::Value`'dur** ve php-dna'nın döndürdüğü yapıyla birebir aynıdır — tipli modeller değildir. Anahtar adları, metin bayrakları (`"true"` / `"false"`), biçimlendirilmiş tutar metinleri (`"11.9900"`) ve zarf (envelope) yapıları aynıdır:
  - başarı: `{"result": "OK", "data": {...}}`
  - hata: `{"result": "ERROR", "error": {"Code": "...", "Message": "...", "Details": "..."}}`
  - birkaç metot php-dna'dan aynen alınmış başka bir zarf kullanır: `get_current_balance` bir `OperationResult` nesnesi döndürür, `check_availability` başarıda çıplak bir liste döndürür, `get_reseller_details` alanlarını en üst seviyede döndürür.
- **Hatalar fırlatılmaz, döndürülür.** Gateway hataları, HTTP hataları ve ağ arızalarının hepsi hata zarfı olarak döner; hiçbir şey panic oluşturmaz.
- **Yapılandırılmış girdiler** (kişiler, nameserver listeleri, filtreler, TLD öznitelikleri, kayıt içerikleri) `json!` ile oluşturulan `&Value` değerleridir. Kişi haritaları php-dna'nın anahtarlarını kullanır (`FirstName`, `EMail`, `AddressLine1`, `PhoneCountryCode`, ...); hem düz hem iç içe (`Address.Line1`, `Phone.Phone.Number`) kişi biçimleri kabul edilir. Nameserver listeleri liste ya da harita olabilir (değerler kullanılır).
- **Rust'ta varsayılan argüman yoktur.** Her metot php-dna'nın tam argüman listesini aynı sırayla alır. php-dna'nın varsayılanları sabit olarak dışa aktarılır: `DEFAULT_TTL` (3600), `DEFAULT_REASON` ("Owner request"), `DEFAULT_FORWARD_TYPE` ("Standard"), `DEFAULT_CURRENCY` ("USD"), `DEFAULT_TLD_COUNT` (20), `DEFAULT_COMMAND` ("create"). Dört isteğe bağlı argümanı olan `register_with_contact_info` için ayrıca, `Default` değeri php-dna'nın varsayılanları olan bir `RegisterOptions` yapısı alan `register_with_contact_info_opts` de vardır.

## Kullanım

Tüm örnekler `use domainnameapi::{json, DomainNameApi};` satırını ve `let mut dna = DomainNameApi::new(...)` ile oluşturulmuş bir istemciyi varsayar.

#### Domain Kaydı

Not: .tr alan adları için ek parametreler gereklidir. Ek öznitelikler argümanı, .tr gibi ekstra bilgi gerektiren alan adları için kullanılır.

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

Bir `.tr` alan adı için TRABIS özniteliklerini gönderin. Kütüphane yalnızca gateway'in kabul ettiklerini gönderir (`TRABISDOMAINCATEGORY`, `TRABISCITIZENID`, `TRABISTAXOFFICE`, `TRABISTAXNUMBER`), `1`/`0` değerlerini `Owner`/`Corporate` olarak eşler ve eski `TRABISCITIZIENID` yazımını düzeltir:

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
<summary>Domain Kaydı için Örnek Çıktı</summary>

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

#### Domain Yenileme

```rust
let result = dna.renew("example.com", 1);
```

<details>
<summary>Domain Yenileme için Örnek Çıktı</summary>

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

#### Domain Transferi

```rust
let result = dna.transfer("example.com", "Xy9#mK2$", 3);

// php-dna's facade sends no contacts with a transfer; to send them:
let result = dna.transfer_with_contacts("example.com", "Xy9#mK2$", 3, &contacts);
```

<details>
<summary>Domain Transferi için Örnek Çıktı</summary>

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

#### Transfer Yönetimi

```rust
// Can the domain be transferred with this auth code?
let check = dna.check_transfer("example.com", "Xy9#mK2$");

// Cancel an incoming transfer, approve or reject an outgoing one
let result = dna.cancel_transfer("example.com");
let result = dna.approve_transfer("example.com");
let result = dna.reject_transfer("example.com");
```

<details>
<summary>Transfer Yönetimi için Örnek Çıktı</summary>

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

`check_transfer`, kayıt otoritesi alan adını transfer edilemez olarak bildirdiğinde `"result": "ERROR"` ve bir `error` (`TRANSFER_NOT_AVAILABLE`) döndürür; `data` her iki durumda da doludur. `cancel_transfer`, `approve_transfer` ve `reject_transfer` şunu döndürür:

```json
{"result": "OK", "data": {"DomainName": "example.com", "Status": "Cancelled"}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Domain Listesi

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

`PageNumber` 0'dan başlar. Gateway'in kendi `MaxResultCount`, `SkipCount` ve `Sorting` parametreleri ile `DomainName` gibi filtreler de gönderilebilir.

<details>
<summary>Domain Listesi için Örnek Çıktı</summary>

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

#### TLD Listesi

```rust
let result = dna.get_tld_list(100);
```

<details>
<summary>TLD Listesi için Örnek Çıktı</summary>

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

#### Domain Müsaitlik Kontrolü

```rust
let result = dna.check_availability(["hello", "world123x0"], ["com", "net"], 1, "create");
```

<details>
<summary>Domain Müsaitlik Kontrolü için Örnek Çıktı</summary>

Başarıda sonuç çıplak bir listedir (`result` anahtarı yoktur); hata durumunda hata zarfıdır.

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

#### Domain Detayları

```rust
let result = dna.get_details("example.com");

// Re-read the domain from the registry (same result shape):
let result = dna.sync_from_registry("example.com");
```

<details>
<summary>Domain Detayları için Örnek Çıktı</summary>

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

#### Nameserver Değiştirme

```rust
let result = dna.modify_name_server("example.com", &json!(["ns1.example.com", "ns2.example.com"]));
```

<details>
<summary>Nameserver Değiştirme için Örnek Çıktı</summary>

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

#### Domain Kilidini Açma (Kilitleme)

```rust
let result = dna.enable_theft_protection_lock("example.com");
```

<details>
<summary>Domain Kilitleme için Örnek Çıktı</summary>

```json
{"result": "OK", "data": {"LockStatus": true}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Domain Kilidini Kaldırma

```rust
let result = dna.disable_theft_protection_lock("example.com");
```

<details>
<summary>Domain Kilidini Kaldırma için Örnek Çıktı</summary>

```json
{"result": "OK", "data": {"LockStatus": false}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Child Nameserver Ekleme

```rust
let result = dna.add_child_name_server("example.com", "ns1.example.com", "1.2.3.4");
```

<details>
<summary>Child Nameserver Ekleme için Örnek Çıktı</summary>

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

#### Child Nameserver Silme

```rust
let result = dna.delete_child_name_server("example.com", "ns1.example.com");
```

<details>
<summary>Child Nameserver Silme için Örnek Çıktı</summary>

```json
{"result": "OK", "data": {"NameServer": "ns1.example.com"}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Child Nameserver Güncelleme

```rust
let result = dna.modify_child_name_server("example.com", "ns1.example.com", "1.2.3.4");
```

<details>
<summary>Child Nameserver Güncelleme için Örnek Çıktı</summary>

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

#### Domain Gizliliğini Değiştirme

```rust
let result = dna.modify_privacy_protection_status("example.com", true, "Owner request");
```

<details>
<summary>Domain Gizliliğini Değiştirme için Örnek Çıktı</summary>

```json
{"data": {"PrivacyProtectionStatus": true}, "result": "OK"}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Domain Kişilerini Alma

```rust
let result = dna.get_contacts("example.com");
```

<details>
<summary>Domain Kişilerini Alma için Örnek Çıktı</summary>

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

#### Domain Kişilerini Kaydetme

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
<summary>Domain Kişilerini Kaydetme için Örnek Çıktı</summary>

```json
{"result": "OK", "data": {"contacts": []}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### DNS Zone Kayıtları

Gateway kayıtları **kümeler** halinde saklar (bir veya daha fazla değer tutan bir ad + tip). Bir kümeyi yazmak tüm değerlerini değiştirir, herhangi bir değeri silmek ise kümenin tamamını kaldırır. `*_value` metotları oku-birleştir-yaz işlemini sizin yerinize yapar; böylece bir panel kayıtları tek tek ele alabilir. Adlar göreli (`www`), apex için `@` / boş ya da tam nitelikli olabilir; gateway'in beklediği göreli etikete indirgenir. Desteklenen tipler: A, AAAA, ALIAS, CAA, CNAME, DS, MX, NAPTR, PTR, SRV, TLSA, TXT. `apply` `true` olduğunda (php-dna varsayılanı) değişiklikler hemen uygulanır; aksi halde `apply_resource_records` çağırın. Gateway ani istek patlamalarını sınırladığı için zone çağrıları arasında en az bir saniye bırakılır.

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
<summary>DNS Zone Kayıtları için Örnek Çıktı</summary>

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

Zaten mevcut olan (silmede ise mevcut olmayan) bir değer hiçbir şeyi değiştirmez:

```json
{"result": "OK", "data": {"Name": "", "Type": "MX", "Contents": ["10 mx1.example.com."], "Changed": false}}
```

Desteklenmeyen bir tip, hiçbir istek yapılmadan reddedilir:

```json
{"result": "ERROR", "error": {"Code": "ADD_ZONE", "Message": "Record type \"NS\" is not supported by the DNS zone API. Supported types: A, AAAA, ALIAS, CAA, CNAME, DS, MX, NAPTR, PTR, SRV, TLSA, TXT.", "Details": "..."}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### URL Yönlendirme

```rust
let current = dna.get_forwarding("example.com");
let result = dna.set_forwarding("example.com", "https://www.example.org", "Standard"); // or "Frame" (masked)
let result = dna.delete_forwarding("example.com");
```

<details>
<summary>URL Yönlendirme için Örnek Çıktı</summary>

```json
{"result": "OK", "data": {"Enabled": true, "RedirectAddress": "https://www.example.org/landing", "ForwardType": "Frame"}}
```

Yönlendirme tanımlı değilse `get_forwarding` şunu döndürür: `{"result": "OK", "data": {"Enabled": false, "RedirectAddress": "", "ForwardType": ""}}`. `set_forwarding` şunu döndürür: `{"result": "OK", "data": {"RedirectAddress": "...", "ForwardType": "Standard"}}`.

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Bayi Bilgilerini Alma

```rust
let result = dna.get_reseller_details();
let balance = dna.get_current_balance("USD"); // "USD", "TRY", "EUR" or "GBP"
```

<details>
<summary>Bayi Bilgileri için Örnek Çıktı</summary>

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

Hata durumunda `OperationResult` `"FAILED"` olur, `ErrorCode` hata kodunu tutar (ör. `"API_429"`) ve `Balance` `"0.00"` olur.

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Yardımcı araçlar (ağ kullanmaz)

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
<summary>validate_contact için Örnek Çıktı</summary>

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

## Hata yönetimi

Her ağ metodu bir değer döndürür; `result` alanını kontrol edin:

```rust
let out = dna.get_details("missing.com");
if out["result"] == "ERROR" {
    println!("{}", out["error"]["Code"]);    // "API_404"
    println!("{}", out["error"]["Message"]); // "The domain name you requested could not be found. ..."
    println!("{}", out["error"]["Details"]);
}
```

- HTTP hataları `API_<durum>` kodunu kullanır (`API_400`, `API_404`, `API_429`, `API_500`, ...). Boş bir hata gövdesi `HTTP 429 - Too Many Requests` gibi okunabilir bir mesaja dönüştürülür.
- 301/302 yanıtı hatalı kimlik bilgisi demektir: `Invalid API credentials`.
- Gateway doğrulama hataları alanın adını belirterek `Details` içinde düzleştirilir, ör. `Contacts Phone - The field Phone must be a string with a maximum length of 16.`
- Ağ arızaları hata olarak döner, asla boş bir başarı olarak değil. Zaman aşımı mesajı `Could not reach the DomainNameAPI service: the connection timed out. (...)` şeklindedir; diğer arızalar `Curl error during request: ...` olarak okunur. Zarf kodu `API_<curl hata numarası>`'dır (zaman aşımı için `API_28`, DNS için `API_6`, reddedilen bağlantı için `API_7`) ve `get_parsed_response_data()["Code"]` `CURL_<n>` değerini tutar.
- Yerel doğrulama (desteklenmeyen kayıt tipi, eksik yönlendirme adresi, kontrol edilecek alan adı olmaması) herhangi bir istekten önce açıklayıcı bir kodla başarısız olur (`ADD_ZONE`, `SET_FORWARD`, `AVAILABILITY`, ...).

### Tanılama

```rust
dna.get_request_data();        // {"url", "method", "payload"} of the last request
dna.get_response_data();       // decoded last response, or {"raw_response": "<body>"}
dna.get_response_headers();    // {"url", "content_type", "http_code", "total_time", "headers"}
dna.get_parsed_response_data();// parsed body on success, {"Code", "Message", "Details"} on failure
dna.get_last_function();       // e.g. "request:GET domains/info"
dna.get_service_url();
dna.set_service_url("https://ote.domainresellerapi.com/api/v1");
```

### Özel transport

HTTP katmanı `Transport` trait'idir. Varsayılan `UreqTransport`'tur (rustls ile engelleyici `ureq`, toplam 30 sn ve bağlantı için 10 sn zaman aşımı, yönlendirmeler takip edilmez). Proxy'ler, özel istemciler veya testler için değiştirebilirsiniz:

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

## Test modu

`https://ote.domainresellerapi.com/api/v1` adresindeki OTE test ortamını kullanmak için yapıcının üçüncü argümanı olarak `true` (veya `"true"`, `"1"`, `"on"`, `"yes"`) verin. Canlı çağrılar `https://api.domainresellerapi.com/api/v1` adresine gider.

## Testler

Test paketi ağ bağlantısı veya kimlik bilgisi gerektirmez:

```bash
cargo test
```

HTTP katmanı, hata eşleme, telemetri maskeleme ve yardımcı araçlar için birim testleri ile `tests/conformance/` altındaki 126 php-dna senaryosunun tamamını yeniden oynatan ve ağa giden her isteğin ve her dönüş değerinin php-dna ile aynı olduğunu doğrulayan bir uyumluluk testi içerir.

## Hata raporlama (telemetri)

Bir API çağrısı başarısız olduğunda kütüphane, DomainNameAPI'nin kendi hata izleme sistemine (Sentry) bir hata raporu gönderebilir. Başarılı çağrıların küçük ve rastgele bir bölümü (%4, ayrıca 1 saniyeden yavaş her çağrı) performans kaydı olarak gönderilir. Bu, DomainNameAPI'nin sorunları bulup düzeltmesine yardımcı olur. Hata raporlama **varsayılan olarak açıktır**.

> ℹ️ 1.0.0 sürümünde Rust kütüphanesi için Sentry projesi henüz oluşturulmadığından yerleşik DSN boştur ve hiçbir şey gönderilmez. Aşağıdaki davranış, DSN yapılandırıldığında (veya `set_telemetry_dsn` ile kendiniz bir DSN ayarladığınızda) geçerlidir.

**Gönderilenler:**

- hata kodu ve mesajı
- çağrılan uç nokta yolu ve HTTP metodu
- çağrı süresi
- uygulama adı (`DomainNameAPI-Rust`) ve kütüphane sürümü
- bayi ID'si ve alan adı
- sunucunun genel IP adresi (arka planda `api.ipify.org` üzerinden sorgulanır)
- işletim sistemi kullanıcı adı
- alan adının kişi bilgileri dahil API isteği ve yanıtı; yalnızca kimlik bilgileri maskelenir — EPP/auth kodları, API anahtarları, parolalar, gizli anahtarlar ve token'lar `***` ile değiştirilir

**Asla gönderilmeyenler:** okunabilir biçimde EPP/auth kodları, API anahtarları, parolalar veya token'lar.

Raporlar kısa zaman aşımlarıyla bir arka plan iş parçacığından gönderilir; çağrınızı asla bekletmez veya başarısız kılmaz. "Alan adı bulunamadı" veya "yetersiz bakiye" gibi beklenen hatalar raporlanmaz.

**Neden ve ne kadar süre:** veriler yalnızca kullanım ölçümü ve sorunların erken teşhisi için kullanılır ve DomainNameAPI'nin kendi Sentry sunucusunda 30 gün saklanır, ardından otomatik olarak silinir.

**Nasıl kapatılır veya azaltılır:**

```rust
let mut dna = DomainNameApi::new(reseller_id, api_key, false);
dna.set_error_reporting_enabled(false); // nothing is sent

// or keep it on but send only the error code and message, endpoint,
// timing, versions and reseller ID (no request/response data, domain, IP or user name)
dna.set_telemetry_minimal(true);
```

> 🔒 **Güvenlik:** TLS sertifika doğrulaması açıktır. CA paketi eski olan bir sunucuda `dna.set_ssl_verification(false);` ile kapatılabilir — bu önerilmez; CA paketini güncellemek daha doğru çözümdür.

## Dönüş ve Hata Kodları ile Açıklamaları

| Kod   | Açıklama                                               | Detay                                                             |
|-------|--------------------------------------------------------|-------------------------------------------------------------------|
| 1000  | Command completed successfully                       | İşlem başarılı.                                                  |
| 1001  | Command completed successfully; action pending.     | İşlem başarılı. Fakat işlem şu an tamamlanmak için kuyruğa alındı. |
| 2003  | Required parameter missing                          | Parametre eksik hatası. Örneğin; Kontak bilgisinde telefon girişi yapılmaması. |
| 2105  | Object is not eligible for renewal                  | Domain durumu yenilemeye müsait değil, güncelleme işlemlerine kilitlenmiştir. Durum durumu "clientupdateprohibited" olmamalı. Diğer durum durumlarından kaynaklanabilir. |
| 2200  | Authentication error                               | Yetki hatası, güvenlik kodu hatalı veya domain başka bir kayıt firmasında bulunuyor. |
| 2302  | Object exists                                      | Domain adı veya name server bilgisi veritabanında mevcut. Kayıt edilemez. |
| 2303  | Object does not exist                              | Domain adı veya name server bilgisi veritabanında mevcut değil. Yeni kayıt oluşturulmalı. |
| 2304  | Object status prohibits operation                  | Domain durumu güncellemeye müsait değildir, güncelleme işlemlerine kilitlenmiştir. Durum durumu "clientupdateprohibited" olmamalı. Diğer durum durumlarından kaynaklanabilir. |

| KOD | DETAY                                                                                                                               |
|-----|-------------------------------------------------------------------------------------------------------------------------------------|
| 101 | Sistem Hatası Detayları ({0}) İçin!                                                                                                |
| 102 | Çoklu Hatalar Detayları ({0}) İçin!                                                                                                |
| 103 | Bilinmeyen Hatalar Detayları ({0}) İçin!                                                                                           |
| 200 | API komutu bulunamadı ({0})!                                                                                                       |
| 210 | API servisi bulunamadı ({0})!                                                                                                      |
| 211 | API servis sağlayıcısı ayarlanmamış!                                                                                               |
| 300 | Bayi bulunamadı!                                                                                                                   |
| 301 | Mevcut IP adresiniz erişim için yetkilendirilmemiş. Yetkili IP adresinden bağlandığınızdan emin olun ve tekrar deneyin            |
| 310 | TLD desteklenmiyor!                                                                                                                |
| 320 | API bulunamadı!                                                                                                                    |
| 321 | Para birimi desteklenmiyor!                                                                                                        |
| 330 | Gerekli parametre(ler) ayarlanmadı ({0}).                                                                                          |
| –   | Tüm iletişim nesnelerini tam olarak gönderdiğinizden emin olun                                                                     |
| 340 | Fiyat tanımı bulunamadı ({0}[{1}]-{2}{3}).                                                                                         |
| 350 | Yetersiz bayi bakiyesi. (Bayi Id : {0} - Mevcut Bakiye : {1} {2}).                                                                 |
| 350 | Muhasebe para birimi eşleşmiyor veya bakiye yetersiz.                                                                              |
| 360 | Alan ({0}) için geçersiz API isteği.                                                                                               |
| 360 | API kotası aşıldı!                                                                                                                 |
| 361 | Azaltma hatası!                                                                                                                    |
| 362 | Premium domain şu anda kayıt için kullanılamıyor.                                                                                  |
| 363 | Domain otomatik yenileme döneminde olduğu için işlem iptal edildi                                                                  |
| 364 | Bu domain şu anda kayıt defteri ile ilgili bir sorun nedeniyle işlem için kullanılamıyor                                          |

| KOD | DETAY                                                                                       |
|-----|---------------------------------------------------------------------------------------------|
| 400 | Geçersiz iletişim ({0}).                                                                   |
| 401 | İletişim bilgisi senkronize edilemiyor.                                                    |
| 402 | Kayıt defterinden iletişim bilgilerine erişim yok.                                         |
| 403 | SİSTEMDE EKSİK BİLGİ VAR. VARSAYILAN BİLGİLERİNİZİ GİRİN VEYA DESTEK EKIBI İLE İLETİŞİME GEÇİN. |
| 404 | DOMAIN İLETİŞİMİ GÜNCELLENEMİYOR. KAYIT DEFTERİNDEN İZİN.                                  |
| 410 | İletişim bulunamadı.                                                                       |
| 410 | İletişim bulunamadı ({0}).                                                                 |
| 420 | İletişim {0} için geçersiz Api komutu.                                                     |
| 430 | İletişim api'si bulunamadı.                                                                |
| 440 | İletişim senkronize edilmemiş.                                                             |
| 450 | Çok fazla domain iletişimi.. !                                                             |
| 451 | İletişim güncellemesi ile ilerlenemedi.                                                    |

| KOD | DETAY                                                                                                                               |
|-----|-------------------------------------------------------------------------------------------------------------------------------------|
| 500 | Geçersiz domain id.                                                                                                                 |
| 500 | Geçersiz domain id ({0}).                                                                                                           |
| 501 | Domain senkronize edilemedi({0})                                                                                                   |
| 502 | İç transfer başarısız                                                                                                              |
| 503 | Domain kaydı kullanılamıyor.                                                                                                       |
| 504 | Domain bilgisi eşleşmiyor .. Önce :{0}                                                                                             |
| 505 | IP Adresini girin.                                                                                                                 |
| 506 | Domain Transferi Başlatıldı Ancak İletişim Bilgisi Okunamadı ..                                                                    |
| 507 | Yetkilendirme hatası.                                                                                                              |
| 510 | Domain bulunamadı.                                                                                                                 |
| 511 | Süresi dolmuş domainler bulunamıyor.                                                                                               |
| 512 | Domain yenilenemez.                                                                                                                |
| 513 | Domain güncellenebilir durumda değil. Güncellenebilmesi için aktif olmalı                                                          |
| 514 | Geri Alım Süresi Bekleniyor.                                                                                                       |
| 520 | Domain "{0}" için geçersiz Api komutu.                                                                                             |
| 530 | Geçersiz domain süresi. Süre {0} ile {1} yıl arasında olmalı. İstenen süre {2} yıl.                                               |
| 540 | Domain mevcut tarihten itibaren {0} yılın ötesine uzatılamaz.                                                                      |
| 550 | Geçersiz domain adı. Domain {0} ile {1} karakter uzunluğunda olmalı. İstenen domain adı {2} karakter.                             |
| 560 | Geçersiz name server sayısı. Domain {0} ile {1} name server'a sahip olmalı. İstenen name server sayısı {2}.                       |
| 561 | Gelen istekte name server bilgisi bulunamadı                                                                                       |
| 570 | Idn tld "{0}" için desteklenmiyor.                                                                                                 |
| 571 | Süre geçersiz.                                                                                                                     |
| 572 | Komut geçersiz.                                                                                                                    |
| 573 | Domain adları bulunamadı.                                                                                                          |
| 574 | TLD adları bulunamadı.                                                                                                             |
| 575 | Domain güncellenebilir durumda değil. Name server'ların güncellenmesi için aktif olmalı                                            |
| 576 | Domain son kullanma tarihinden önceki son 1 ay içinde yenilenebilir                                                                |
| 580 | Transfer tld "{0}" için desteklenmiyor.                                                                                            |
| 581 | Alt name server bulunamadı                                                                                                         |
| 582 | Transfer başka bayi tarafından başlatıldı.                                                                                         |
| 583 | Transfer başlatılmadı. Lütfen destek ekibi ile iletişime geçin.                                                                    |
| 584 | Nesne durumu işlemi yasaklıyor                                                                                                     |
| 590 | Bu transfer için auth kodu gerekli.                                                                                                |
| 591 | Auth kodu geçerli değil.                                                                                                           |
| 592 | Domain üzerinde transfer kilidi mevcut.                                                                                            |
| 593 | Domain transfer edilemiyor, Durum Bilgisi ve Transfer Kilidi gerekli kriterlere uymalı. (Durum Bilgisi: #ok)                      |
| 594 | Domain name server adresi çözümlenemedi ({0})                                                                                      |
| 595 | İletişim bilgisi okunamıyor (whois.registrar.tld) Gizlilik koruma durumunun açık olduğundan emin olun                             |
| 596 | İletişim bilgisi doğrulanamadı                                                                                                     |
| 597 | Tld Bulunamadı                                                                                                                     |
| 598 | Bilinmeyen hata oluştu                                                                                                             |
| 599 | Domain Yönlendirmesi Bulunamadı                                                                                                    |

## Lisans

Apache License 2.0 — bkz. [LICENSE](LICENSE).
