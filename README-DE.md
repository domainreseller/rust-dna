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

# rust-dna — DomainNameAPI-Client für Rust

Rust-Client für die REST-API des Domain-Registrars DomainNameAPI. Er ist eine originalgetreue Portierung der PHP-Referenzbibliothek [php-dna](https://github.com/domainreseller/php-dna): dieselben Methoden, dieselben Anfragen auf der Leitung und dieselben Rückgabewerte. Jede Portierung wird gegen die 134 Konformitäts-Fixtures von php-dna geprüft.

## 📦 Download — immer die Releases verwenden!

⬇️ **Die neueste getestete Version gibt es hier: https://github.com/domainreseller/rust-dna/releases/latest**

> ⚠️ Verwenden Sie **nicht** die grüne Schaltfläche **Code → Download ZIP** — damit laden Sie den rohen Entwicklungszweig herunter. Release-Pakete sind versioniert, getestet und produktionsreif.

## Installations- und Integrationsanleitung

### Mindestanforderungen

- Rust 1.83 oder neuer (Edition 2021)
- Keine Async-Laufzeitumgebung: die API ist blockierend
- Laufzeitabhängigkeiten: `ureq` (HTTP, rustls-TLS), `rustls` und `serde_json`. OpenSSL wird nicht benötigt.

### Installation

**A) Git-Abhängigkeit (auf ein Release-Tag festgelegt)**

```toml
[dependencies]
domainnameapi = { git = "https://github.com/domainreseller/rust-dna", tag = "v1.0.0" }
```

**B) Heruntergeladenes Release**

Entpacken Sie das Release-Archiv neben Ihrem Projekt und verweisen Sie Cargo darauf:

```toml
[dependencies]
domainnameapi = { path = "../rust-dna" }
```

Das Crate exportiert `serde_json::Value` und das Makro `json!` erneut, daher müssen Sie `serde_json` nicht selbst hinzufügen (Sie können es aber, wenn Sie möchten).

## 🔑 API-Zugangsdaten — Reseller-ID & API-Key

Die Bibliothek spricht mit der DomainNameAPI-REST-API und benötigt genau zwei Zugangsdaten, die dem Konstruktor übergeben werden:

| Parameter | Wert |
|---|---|
| Erster | Reseller-ID — UUID wie `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` |
| Zweiter | API-Key |

> 💡 Ihre **Reseller-ID** und Ihren **API-Key** finden Sie in Ihrem DomainNameAPI-Panel unter **API-Einstellungen**.
> ⚠️ Dies sind **API-Zugangsdaten** — Ihre Panel-Anmeldedaten funktionieren hier **nicht**.

```rust
use domainnameapi::DomainNameApi;

let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";
let api_key = "your-api-key";

// Live environment
let mut dna = DomainNameApi::new(reseller_id, api_key, false);

// Test (OTE) environment
let mut dna = DomainNameApi::new(reseller_id, api_key, true);
```

Beide Zugangsdaten werden von Leerzeichen am Anfang und Ende befreit. Das dritte Argument akzeptiert auch Zeichenketten, da Panels Kontrollkästchen oft als Text speichern: `"true"`, `"1"`, `"on"` und `"yes"` (ohne Beachtung der Groß-/Kleinschreibung) wählen die Testumgebung, alles andere (einschließlich `"false"` und `"off"`) wählt die Live-Umgebung.

## Schnellstart

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

Jedes Beispiel in [examples/](examples) lässt sich mit `cargo run --example <name>` ausführen, z. B. `cargo run --example get_details`.

## Aufbau der API

- **Ein einziger Client-Typ**, `DomainNameApi`. Methoden nehmen `&mut self` (der Client merkt sich die letzte Anfrage für die Diagnose).
- **Rückgabewerte sind `serde_json::Value`** mit genau der Struktur, die php-dna zurückgibt — keine typisierten Modelle. Schlüsselnamen, Zeichenketten-Flags (`"true"` / `"false"`), formatierte Geldbeträge (`"11.9900"`) und die Umschläge sind identisch:
  - Erfolg: `{"result": "OK", "data": {...}}`
  - Fehler: `{"result": "ERROR", "error": {"Code": "...", "Message": "...", "Details": "..."}}`
  - einige Methoden verwenden einen anderen, aus php-dna übernommenen Umschlag: `get_current_balance` liefert ein `OperationResult`-Objekt, `check_availability` liefert bei Erfolg eine reine Liste, `get_reseller_details` liefert seine Felder auf oberster Ebene.
- **Fehler werden zurückgegeben, nie geworfen.** Gateway-Fehler, HTTP-Fehler und Netzwerkausfälle kommen alle als Fehlerumschlag zurück; nichts löst einen Panic aus. Registrierung, Transfer und Verlängerung liefern auch dann einen Fehler, wenn das Gateway mit HTTP 200 und `success: false` antwortet.
- **Strukturierte Eingaben** (Kontakte, Nameserver-Listen, Filter, TLD-Attribute, Record-Inhalte) sind `&Value`, erstellt mit `json!`. Kontakt-Maps verwenden die Schlüssel von php-dna (`FirstName`, `EMail`, `AddressLine1`, `PhoneCountryCode`, ...); sowohl die flache als auch die verschachtelte Form (`Address.Line1`, `Phone.Phone.Number`) wird akzeptiert. Nameserver-Listen können eine Liste oder eine Map sein (es werden die Werte verwendet).
- **Keine Standardargumente in Rust.** Jede Methode nimmt die vollständige Argumentliste von php-dna in derselben Reihenfolge. Die Standardwerte von php-dna werden als Konstanten exportiert: `DEFAULT_TTL` (3600), `DEFAULT_REASON` ("Owner request"), `DEFAULT_FORWARD_TYPE` ("Standard"), `DEFAULT_CURRENCY` ("USD"), `DEFAULT_TLD_COUNT` (20), `DEFAULT_COMMAND` ("create"). Für `register_with_contact_info` mit vier optionalen Argumenten gibt es zusätzlich `register_with_contact_info_opts`, das eine `RegisterOptions`-Struktur entgegennimmt, deren `Default` den Standardwerten von php-dna entspricht.

## Verwendung

Alle Beispiele setzen `use domainnameapi::{json, DomainNameApi};` und einen Client `let mut dna = DomainNameApi::new(...)` voraus.

#### Domain-Registrierung

Hinweis: Für .tr-Domains sind zusätzliche Parameter erforderlich. Das Argument für zusätzliche Attribute wird für Domainnamen verwendet, die weitere Angaben benötigen, wie .tr.

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

Übergeben Sie für eine `.tr`-Domain die TRABIS-Attribute. Die Bibliothek sendet nur, was das Gateway akzeptiert (`TRABISDOMAINCATEGORY`, `TRABISCITIZENID`, `TRABISTAXOFFICE`, `TRABISTAXNUMBER`), bildet `1`/`0` auf `Owner`/`Corporate` ab und korrigiert die alte Schreibweise `TRABISCITIZIENID`:

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
<summary>Beispielausgabe für Domain-Registrierung</summary>

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

#### Domain-Verlängerung

```rust
let result = dna.renew("example.com", 1);
```

<details>
<summary>Beispielausgabe für Domain-Verlängerung</summary>

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

#### Domain-Transfer

Transfers senden nie Kontaktdaten: Der Registrar übernimmt die Kontakte vom bisherigen Registrar, daher enthält jede Transferanfrage `contacts: []`.

```rust
let result = dna.transfer("example.com", "Xy9#mK2$", 3);
```

<details>
<summary>Beispielausgabe für Domain-Transfer</summary>

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

#### Transferverwaltung

```rust
// Can the domain be transferred with this auth code?
let check = dna.check_transfer("example.com", "Xy9#mK2$");

// Cancel an incoming transfer, approve or reject an outgoing one
let result = dna.cancel_transfer("example.com");
let result = dna.approve_transfer("example.com");
let result = dna.reject_transfer("example.com");
```

<details>
<summary>Beispielausgabe für Transferverwaltung</summary>

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

`check_transfer` liefert `"result": "ERROR"` mit einem `error` (`TRANSFER_NOT_AVAILABLE`), wenn die Registry die Domain als nicht transferierbar meldet; `data` ist in beiden Fällen gefüllt. `cancel_transfer`, `approve_transfer` und `reject_transfer` liefern:

```json
{"result": "OK", "data": {"DomainName": "example.com", "Status": "Cancelled"}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Domain-Liste

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

`PageNumber` beginnt bei 0. Die gateway-eigenen Parameter `MaxResultCount`, `SkipCount` und `Sorting` sowie Filter wie `DomainName` können ebenfalls übergeben werden.

<details>
<summary>Beispielausgabe für Domain-Liste</summary>

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

#### TLD-Liste

```rust
let result = dna.get_tld_list(100);
```

<details>
<summary>Beispielausgabe für TLD-Liste</summary>

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

#### Domain-Verfügbarkeitsprüfung

```rust
let result = dna.check_availability(["hello", "world123x0"], ["com", "net"], 1, "create");
```

<details>
<summary>Beispielausgabe für Domain-Verfügbarkeitsprüfung</summary>

Bei Erfolg ist das Ergebnis eine reine Liste (ohne `result`-Schlüssel); bei einem Fehler ist es der Fehlerumschlag.

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

#### Domain-Details

```rust
let result = dna.get_details("example.com");

// Re-read the domain from the registry (same result shape):
let result = dna.sync_from_registry("example.com");
```

<details>
<summary>Beispielausgabe für Domain-Details</summary>

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

#### Nameserver ändern

```rust
let result = dna.modify_name_server("example.com", &json!(["ns1.example.com", "ns2.example.com"]));
```

<details>
<summary>Beispielausgabe für Nameserver ändern</summary>

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

#### Domain-Sperre aktivieren

```rust
let result = dna.enable_theft_protection_lock("example.com");
```

<details>
<summary>Beispielausgabe für Domain-Sperre aktivieren</summary>

```json
{"result": "OK", "data": {"LockStatus": true}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Domain-Sperre deaktivieren

```rust
let result = dna.disable_theft_protection_lock("example.com");
```

<details>
<summary>Beispielausgabe für Domain-Sperre deaktivieren</summary>

```json
{"result": "OK", "data": {"LockStatus": false}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Child-Nameserver hinzufügen

```rust
let result = dna.add_child_name_server("example.com", "ns1.example.com", "1.2.3.4");
```

<details>
<summary>Beispielausgabe für Child-Nameserver hinzufügen</summary>

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

#### Child-Nameserver löschen

```rust
let result = dna.delete_child_name_server("example.com", "ns1.example.com");
```

<details>
<summary>Beispielausgabe für Child-Nameserver löschen</summary>

```json
{"result": "OK", "data": {"NameServer": "ns1.example.com"}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Child-Nameserver aktualisieren

```rust
let result = dna.modify_child_name_server("example.com", "ns1.example.com", "1.2.3.4");
```

<details>
<summary>Beispielausgabe für Child-Nameserver aktualisieren</summary>

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

#### Domain-Datenschutz ändern

`status` ist ein `bool` oder ein Wert, wie Panels ihn speichern: `"1"`, `"on"`, `"true"`, `"yes"` und `1` bedeuten an, alles andere aus. Das Argument für den Grund (reason) bleibt nur aus Kompatibilitätsgründen erhalten und wird nicht gesendet, da das Gateway kein solches Feld kennt.

```rust
let result = dna.modify_privacy_protection_status("example.com", true, "Owner request");
```

<details>
<summary>Beispielausgabe für Domain-Datenschutz ändern</summary>

```json
{"data": {"PrivacyProtectionStatus": true}, "result": "OK"}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Domain-Kontakte abrufen

```rust
let result = dna.get_contacts("example.com");
```

<details>
<summary>Beispielausgabe für Domain-Kontakte abrufen</summary>

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

#### Domain-Kontakte speichern

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
<summary>Beispielausgabe für Domain-Kontakte speichern</summary>

```json
{"result": "OK", "data": {"contacts": []}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### DNS-Zonen-Records

Das Gateway speichert Records als **Sets** (ein Name + Typ mit einem oder mehreren Werten). Das Schreiben eines Sets ersetzt alle seine Werte, und das Löschen eines beliebigen Werts entfernt das gesamte Set. Die `*_value`-Methoden übernehmen das Lesen-Zusammenführen-Schreiben für Sie, sodass ein Panel Records einzeln behandeln kann. Namen können relativ (`www`), `@` / leer für den Apex oder vollqualifiziert sein; sie werden auf das relative Label reduziert, das das Gateway erwartet. Unterstützte Typen: A, AAAA, ALIAS, CAA, CNAME, DS, MX, NAPTR, PTR, SRV, TLSA, TXT. Änderungen werden sofort angewendet, wenn `apply` `true` ist (Standard von php-dna); andernfalls rufen Sie `apply_resource_records` auf. Zonen-Aufrufe liegen mindestens eine Sekunde auseinander, weil das Gateway Anfragespitzen drosselt. Schlägt das automatische Anwenden fehl, liefert die Methode den Fehler des Anwendens (die Änderung bleibt vorgemerkt). Ergebnisse melden den großgeschriebenen Typ und die tatsächlich gesendete TTL (unter 1 wird zu 3600, über 86400 wird zu 86400).

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
<summary>Beispielausgabe für DNS-Zonen-Records</summary>

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

Ein Wert, der bereits vorhanden ist (bzw. beim Entfernen fehlt), ändert nichts:

```json
{"result": "OK", "data": {"Name": "", "Type": "MX", "Contents": ["10 mx1.example.com."], "Changed": false}}
```

Ein nicht unterstützter Typ wird vor jeder Anfrage abgelehnt:

```json
{"result": "ERROR", "error": {"Code": "ADD_ZONE", "Message": "Record type \"NS\" is not supported by the DNS zone API. Supported types: A, AAAA, ALIAS, CAA, CNAME, DS, MX, NAPTR, PTR, SRV, TLSA, TXT.", "Details": "..."}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### URL-Weiterleitung

```rust
let current = dna.get_forwarding("example.com");
let result = dna.set_forwarding("example.com", "https://www.example.org", "Standard"); // or "Frame" (masked)
let result = dna.delete_forwarding("example.com");
```

<details>
<summary>Beispielausgabe für URL-Weiterleitung</summary>

```json
{"result": "OK", "data": {"Enabled": true, "RedirectAddress": "https://www.example.org/landing", "ForwardType": "Frame"}}
```

Ist keine Weiterleitung eingerichtet (das Gateway antwortet `Redirect information not found`), liefert `get_forwarding` `{"result": "OK", "data": {"Enabled": false, "RedirectAddress": "", "ForwardType": ""}}`; jeder andere Fehler, etwa eine nicht vorhandene Domain, wird als Fehler zurückgegeben. `set_forwarding` liefert `{"result": "OK", "data": {"RedirectAddress": "...", "ForwardType": "Standard"}}`. `delete_forwarding` liefert `{"result": "OK", "data": {"DomainName": "example.com", "Removed": true}}` bzw. `"Removed": false`, wenn keine Weiterleitung eingerichtet war.

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Reseller-Details abrufen

```rust
let result = dna.get_reseller_details();
let balance = dna.get_current_balance("USD"); // "USD" or "TRY"
```

<details>
<summary>Beispielausgabe für Reseller-Details abrufen</summary>

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

Das Konto meldet USD- und TRY-Guthaben; jede andere Währung (EUR, GBP, ...) schlägt mit `ErrorCode` `"BALANCE"` fehl. Bei einem Fehler ist `OperationResult` `"FAILED"`, `ErrorCode` enthält den Code (z. B. `"API_429"`) und `Balance` ist `"0.00"`.

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Hilfsfunktionen (ohne Netzwerk)

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
<summary>Beispielausgabe für validate_contact</summary>

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

## Fehlerbehandlung

Jede Netzwerkmethode liefert einen Wert; prüfen Sie `result`:

```rust
let out = dna.get_details("missing.com");
if out["result"] == "ERROR" {
    println!("{}", out["error"]["Code"]);        // "API_404"
    println!("{}", out["error"]["GatewayCode"]); // "Dna.DomainService:Domain:10007"
    println!("{}", out["error"]["Message"]);     // "The domain name you requested could not be found. ..."
    println!("{}", out["error"]["Details"]);
}
```

- HTTP-Fehler verwenden den Code `API_<Status>` (`API_400`, `API_404`, `API_429`, `API_500`, ...). Ein leerer Fehler-Body wird zu einer lesbaren Meldung wie `HTTP 429 - Too Many Requests`. Hat das Gateway einen eigenen Fehlercode gesendet, enthält der Umschlag ihn zusätzlich als `GatewayCode` (z. B. `"Dna.DomainService:Domain:10007"`).
- Eine 301/302-Antwort bedeutet falsche Zugangsdaten: `Invalid API credentials`.
- Validierungsfehler des Gateways werden unter Nennung des Feldes in `Details` zusammengefasst, z. B. `Contacts Phone - The field Phone must be a string with a maximum length of 16.`
- Netzwerkausfälle sind Fehler, niemals ein leerer Erfolg. Ein Timeout lautet `Could not reach the DomainNameAPI service: the connection timed out. (...)`; andere Ausfälle lauten `Curl error during request: ...`. Ihr Code ist `CURL_<curl-Fehlernummer>` (`CURL_28` bei Timeout, `CURL_6` bei DNS, `CURL_7` bei abgelehnter Verbindung); `API_<n>` ist HTTP-Statuscodes vorbehalten.
- Lokale Validierung (nicht unterstützter Record-Typ, fehlende Weiterleitungsadresse, keine zu prüfenden Domains) schlägt vor jeder Anfrage mit einem aussagekräftigen Code fehl (`ADD_ZONE`, `SET_FORWARD`, `AVAILABILITY`, ...).

### Diagnose

```rust
dna.get_request_data();        // {"url", "method", "payload"} of the last request
dna.get_response_data();       // decoded last response, or {"raw_response": "<body>"}
dna.get_response_headers();    // {"url", "content_type", "http_code", "total_time", "headers"}
dna.get_parsed_response_data();// parsed body on success, {"Code", "Message", "Details"} on failure
dna.get_last_function();       // e.g. "request:GET domains/info"
dna.get_service_url();
dna.set_service_url("https://ote.domainresellerapi.com/api/v1");
```

### Eigener Transport

Die HTTP-Schicht ist der Trait `Transport`. Standard ist `UreqTransport` (blockierendes `ureq` mit rustls, 30 s Gesamt- und 10 s Verbindungs-Timeout, Weiterleitungen werden nicht verfolgt). Ersetzen Sie ihn für Proxys, eigene Clients oder Tests:

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

## Testmodus

Übergeben Sie `true` (oder `"true"`, `"1"`, `"on"`, `"yes"`) als drittes Konstruktorargument, um die OTE-Testumgebung unter `https://ote.domainresellerapi.com/api/v1` zu verwenden. Live-Aufrufe gehen an `https://api.domainresellerapi.com/api/v1`.

## Tests

Die Testsuite benötigt weder Netzwerk noch Zugangsdaten:

```bash
cargo test
```

Sie enthält Unit-Tests für die HTTP-Schicht, die Fehlerabbildung, die Telemetrie-Maskierung und die Hilfsfunktionen sowie einen Konformitätstest, der alle 134 php-dna-Fixtures aus `tests/conformance/` abspielt und prüft, dass jede Anfrage auf der Leitung und jeder Rückgabewert mit php-dna übereinstimmt.

## Fehlerberichte (Telemetrie)

Wenn ein API-Aufruf fehlschlägt, kann die Bibliothek einen Fehlerbericht an das eigene Fehlertracking von DomainNameAPI (Sentry) senden. Ein kleiner zufälliger Anteil erfolgreicher Aufrufe (4 %, dazu jeder Aufruf, der länger als 1 Sekunde dauert) wird als Performance-Datensatz gesendet. Das hilft DomainNameAPI, Probleme zu finden und zu beheben. Die Fehlerberichterstattung ist **standardmäßig aktiviert**.

> ℹ️ In Version 1.0.0 existiert das Sentry-Projekt für die Rust-Bibliothek noch nicht, daher ist der eingebaute DSN leer und es wird nichts gesendet. Das folgende Verhalten gilt, sobald er konfiguriert ist (oder wenn Sie selbst mit `set_telemetry_dsn` einen DSN setzen).

**Was gesendet wird:**

- Fehlercode und Fehlermeldung
- aufgerufener Endpunktpfad und HTTP-Methode
- Dauer des Aufrufs
- Anwendungsname (`DomainNameAPI-Rust`) und Bibliotheksversion
- Reseller-ID und Domainname
- die öffentliche IP-Adresse des Servers (im Hintergrund über `api.ipify.org` ermittelt)
- der Benutzername des Betriebssystems
- die API-Anfrage und -Antwort einschließlich der Kontaktdaten der Domain; nur Zugangsdaten werden maskiert — EPP-/Auth-Codes, API-Keys, Passwörter, Secrets und Tokens werden durch `***` ersetzt

**Was nie gesendet wird:** EPP-/Auth-Codes, API-Keys, Passwörter oder Tokens in lesbarer Form.

Berichte werden aus einem Hintergrund-Thread mit kurzen Timeouts gesendet; sie blockieren Ihren Aufruf nie und lassen ihn nie fehlschlagen. Erwartete Fehler wie „Domain nicht gefunden“ oder „unzureichendes Guthaben“ werden nicht gemeldet.

**Warum und wie lange:** Die Daten werden nur zur Nutzungsmessung und frühzeitigen Fehlerdiagnose verwendet und 30 Tage auf dem eigenen Sentry-Server von DomainNameAPI gespeichert; danach werden sie automatisch gelöscht.

**So schalten Sie es ab oder reduzieren es:**

```rust
let mut dna = DomainNameApi::new(reseller_id, api_key, false);
dna.set_error_reporting_enabled(false); // nothing is sent

// or keep it on but send only the error code and message, endpoint,
// timing, versions and reseller ID (no request/response data, domain, IP or user name)
dna.set_telemetry_minimal(true);
```

> 🔒 **Sicherheit:** Die Prüfung der TLS-Zertifikate ist aktiviert. Auf einem Host mit veraltetem CA-Bundle kann sie mit `dna.set_ssl_verification(false);` abgeschaltet werden — das wird nicht empfohlen; das CA-Bundle zu aktualisieren ist die bessere Lösung.

## Rückgabe- und Fehlercodes mit Erklärungen

| Code  | Beschreibung                                          | Details                                                           |
|-------|------------------------------------------------------|-------------------------------------------------------------------|
| 1000  | Befehl erfolgreich abgeschlossen                     | Operation erfolgreich.                                             |
| 1001  | Befehl erfolgreich abgeschlossen; Aktion ausstehend  | Operation erfolgreich. Der Vorgang wurde jedoch in die Warteschlange eingereiht. |
| 2003  | Erforderlicher Parameter fehlt                        | Parameterfehler. Zum Beispiel: Keine Telefonnummer in Kontaktinformationen angegeben. |
| 2105  | Objekt ist nicht für Verlängerung geeignet           | Domain-Status erlaubt keine Verlängerung, ist für Updates gesperrt. Status darf nicht "clientupdateprohibited" sein. Kann auch an anderen Statusbedingungen liegen. |
| 2200  | Authentifizierungsfehler                             | Berechtigungsfehler, Sicherheitscode falsch oder Domain bei anderem Registrar. |
| 2302  | Objekt existiert bereits                             | Domain-Name oder Nameserver-Information existiert bereits in der Datenbank. Kann nicht registriert werden. |
| 2303  | Objekt existiert nicht                               | Domain-Name oder Nameserver-Information existiert nicht in der Datenbank. Muss neu erstellt werden. |
| 2304  | Objektstatus verhindert Operation                    | Domain-Status erlaubt keine Aktualisierung, ist für Updates gesperrt. Status darf nicht "clientupdateprohibited" sein. Kann auch an anderen Statusbedingungen liegen. |

| CODE | DETAIL                                                                                                                               |
|------|--------------------------------------------------------------------------------------------------------------------------------------|
| 101  | Systemfehler detailliert für ({0})!                                                                                                  |
| 102  | Mehrere Fehler detailliert für ({0})!                                                                                                |
| 103  | Unbekannte Fehler detailliert für ({0})!                                                                                             |
| 200  | API-Befehl nicht gefunden ({0})!                                                                                                     |
| 210  | API-Service nicht gefunden ({0})!                                                                                                    |
| 211  | API-Service-Provider nicht gesetzt!                                                                                                  |
| 300  | Reseller nicht gefunden!                                                                                                             |
| 301  | Ihre aktuelle IP-Adresse ist nicht für den Zugriff autorisiert. Stellen Sie sicher, dass Sie sich von einer autorisierten IP-Adresse verbinden und versuchen Sie es erneut |
| 310  | TLD wird nicht unterstützt!                                                                                                          |
| 320  | API nicht gefunden!                                                                                                                  |
| 321  | Währung wird nicht unterstützt!                                                                                                      |
| 330  | Erforderliche Parameter nicht gesetzt ({0}).                                                                                         |
| –    | Stellen Sie sicher, dass Sie alle Kontaktobjekte vollständig senden                                                                  |
| 340  | Preisdefinition nicht gefunden ({0}[{1}]-{2}{3}).                                                                                    |
| 350  | Unzureichendes Reseller-Guthaben. (Reseller Id : {0} - Aktuelles Guthaben : {1} {2}).                                                |
| 350  | Buchhaltungswährung stimmt nicht überein oder das Guthaben ist nicht ausreichend.                                                    |
| 360  | Ungültige API-Anfrage für Feld ({0}).                                                                                                |
| 360  | API-Quota überschritten!                                                                                                             |
| 361  | Drosselungsfehler!                                                                                                                   |
| 362  | Premium-Domain ist derzeit nicht für die Registrierung verfügbar.                                                                    |
| 363  | Operation abgebrochen, da sich die Domain in der automatischen Verlängerungsperiode befindet                                        |
| 364  | Diese Domain ist derzeit aufgrund eines Problems mit der Registry nicht für Transaktionen verfügbar                                  |

| CODE | DETAIL                                                                                      |
|------|---------------------------------------------------------------------------------------------|
| 400  | Ungültiger Kontakt ({0}).                                                                   |
| 401  | Kontaktinfo kann nicht synchronisiert werden.                                               |
| 402  | Kein Zugriff auf Kontaktinformationen von der Registry.                                     |
| 403  | DAS SYSTEM HAT FEHLENDE INFORMATIONEN. GEBEN SIE IHRE STANDARDINFORMATIONEN EIN ODER KONTAKTIEREN SIE DAS SUPPORT-TEAM. |
| 404  | DOMAIN-KONTAKT KANN NICHT AKTUALISIERT WERDEN. BERECHTIGUNG VON DER REGISTRY.               |
| 410  | Kontakt nicht gefunden.                                                                     |
| 410  | Kontakt nicht gefunden ({0}).                                                               |
| 420  | Ungültiger Api-Befehl für Kontakt {0}.                                                      |
| 430  | Kontakt-API nicht gefunden.                                                                 |
| 440  | Kontakt ist nicht synchronisiert.                                                           |
| 450  | Zu viele Domain-Kontakte.. !                                                                |
| 451  | Kontaktaktualisierung konnte nicht fortgesetzt werden.                                      |

| CODE | DETAIL                                                                                                                               |
|------|--------------------------------------------------------------------------------------------------------------------------------------|
| 500  | Ungültige Domain-ID.                                                                                                                 |
| 500  | Ungültige Domain-ID ({0}).                                                                                                           |
| 501  | Domain konnte nicht synchronisiert werden({0})                                                                                       |
| 502  | Interner Transfer fehlgeschlagen                                                                                                     |
| 503  | Domain-Registrierung ist nicht verfügbar.                                                                                            |
| 504  | Domain-Informationen stimmen nicht überein .. Vorher :{0}                                                                            |
| 505  | Geben Sie die IP-Adresse ein.                                                                                                        |
| 506  | Domain-Transfer wurde gestartet, aber Kontaktinfo konnte nicht gelesen werden ..                                                     |
| 507  | Autorisierungsfehler.                                                                                                                |
| 510  | Domain nicht gefunden.                                                                                                               |
| 511  | Abgelaufene Domains können nicht gefunden werden.                                                                                    |
| 512  | Domain ist nicht verlängerbar.                                                                                                       |
| 513  | Domain ist nicht in aktualisierbarem Status. Sie muss aktiv sein, um aktualisiert zu werden                                         |
| 514  | Rücknahmeperiode erwartet.                                                                                                           |
| 520  | Ungültiger Api-Befehl für Domain "{0}".                                                                                              |
| 530  | Ungültige Domain-Periode. Periode muss {0} bis {1} Jahre betragen. Angeforderte Periode ist {2} Jahre.                              |
| 540  | Domain kann nicht über {0} Jahre vom aktuellen Datum hinaus verlängert werden.                                                       |
| 550  | Ungültiger Domain-Name. Domain muss {0} bis {1} Zeichen lang sein. Angeforderter Domain-Name ist {2} Zeichen.                       |
| 560  | Ungültige Nameserver-Anzahl. Domain muss {0} bis {1} Nameserver haben. Angeforderte Nameserver-Anzahl ist {2}.                      |
| 561  | Keine Nameserver-Informationen in der eingehenden Anfrage gefunden                                                                   |
| 570  | Idn nicht unterstützt für TLD "{0}".                                                                                                 |
| 571  | Periode ungültig.                                                                                                                    |
| 572  | Befehl ungültig.                                                                                                                     |
| 573  | Domain-Namen nicht gefunden.                                                                                                         |
| 574  | TLD-Namen nicht gefunden.                                                                                                            |
| 575  | Domain ist nicht in aktualisierbarem Status. Sie muss aktiv sein, damit Nameserver aktualisiert werden können                       |
| 576  | Domain kann in dem letzten Monat vor dem Ablaufdatum verlängert werden                                                               |
| 580  | Transfer nicht unterstützt für TLD "{0}".                                                                                            |
| 581  | Child-Nameserver nicht gefunden                                                                                                      |
| 582  | Transfer von anderem Reseller gestartet.                                                                                             |
| 583  | Transfer nicht initialisiert. Bitte kontaktieren Sie das Support-Team.                                                              |
| 584  | Objektstatus verbietet Operation                                                                                                     |
| 590  | Auth-Code ist für diesen Transfer erforderlich.                                                                                      |
| 591  | Auth-Code ist nicht gültig.                                                                                                          |
| 592  | Transfer-Sperre existiert auf Domain.                                                                                                |
| 593  | Domain kann nicht transferiert werden, Statusinformationen und Transfer-Sperre müssen die erforderlichen Kriterien erfüllen. (Statusinformationen: #ok) |
| 594  | Domain-Nameserver-Adresse konnte nicht aufgelöst werden ({0})                                                                        |
| 595  | Kontaktinformationen können nicht gelesen werden (whois.registrar.tld) Stellen Sie sicher, dass der Datenschutzstatus offen ist     |
| 596  | Kontaktinformationen konnten nicht verifiziert werden                                                                                |
| 597  | TLD nicht gefunden                                                                                                                   |
| 598  | Unbekannter Fehler aufgetreten                                                                                                       |
| 599  | Domain-Weiterleitung nicht gefunden                                                                                                  |

## Lizenz

Apache License 2.0 — siehe [LICENSE](LICENSE).
