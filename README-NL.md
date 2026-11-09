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

# rust-dna — DomainNameAPI-client voor Rust

Rust-client voor de REST-API van domeinregistrar DomainNameAPI. Het is een getrouwe port van de PHP-referentiebibliotheek [php-dna](https://github.com/domainreseller/php-dna): dezelfde methoden, dezelfde verzoeken over de lijn en dezelfde retourwaarden. Elke port wordt gecontroleerd met de 126 conformiteitsfixtures van php-dna.

## 📦 Downloaden — gebruik altijd de Releases!

⬇️ **Download de nieuwste geteste versie hier: https://github.com/domainreseller/rust-dna/releases/latest**

> ⚠️ Gebruik **niet** de groene knop **Code → Download ZIP** — die downloadt de ruwe ontwikkelbranch. Releasepakketten zijn geversioneerd, getest en klaar voor productie.

## Installatie- en integratiegids

### Minimale vereisten

- Rust 1.83 of nieuwer (editie 2021)
- Geen async-runtime: de API is blokkerend
- Runtime-afhankelijkheden: `ureq` (HTTP, rustls-TLS), `rustls` en `serde_json`. OpenSSL is niet nodig.

### Installatie

**A) Git-afhankelijkheid (vastgezet op een releasetag)**

```toml
[dependencies]
domainnameapi = { git = "https://github.com/domainreseller/rust-dna", tag = "v1.0.0" }
```

**B) Gedownloade release**

Pak het releasearchief uit naast uw project en verwijs Cargo ernaar:

```toml
[dependencies]
domainnameapi = { path = "../rust-dna" }
```

De crate exporteert `serde_json::Value` en de macro `json!` opnieuw, zodat u `serde_json` niet zelf hoeft toe te voegen (dat mag wel als u wilt).

## 🔑 API-gegevens — Reseller ID en API Key

De bibliotheek praat met de DomainNameAPI REST-API en heeft precies twee inloggegevens nodig, die aan de constructor worden meegegeven:

| Parameter | Waarde |
|---|---|
| Eerste | Reseller ID — UUID zoals `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` |
| Tweede | API Key |

> 💡 U vindt uw **Reseller ID** en **API Key** in uw DomainNameAPI-paneel onder **API-instellingen**.
> ⚠️ Dit zijn **API-gegevens** — uw paneel-inloggegevens werken hier **niet**.

```rust
use domainnameapi::DomainNameApi;

let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";
let api_key = "your-api-key";

// Live environment
let mut dna = DomainNameApi::new(reseller_id, api_key, false);

// Test (OTE) environment
let mut dna = DomainNameApi::new(reseller_id, api_key, true);
```

Van beide gegevens worden spaties aan begin en eind verwijderd. Het derde argument accepteert ook tekst, omdat panelen selectievakjes vaak als tekst opslaan: `"true"`, `"1"`, `"on"` en `"yes"` (hoofdletterongevoelig) kiezen de testomgeving, al het andere (inclusief `"false"` en `"off"`) kiest de live-omgeving.

## Snel aan de slag

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

Elk voorbeeld in [examples/](examples) draait met `cargo run --example <naam>`, bijvoorbeeld `cargo run --example get_details`.

## Opbouw van de API

- **Eén clienttype**, `DomainNameApi`. Methoden nemen `&mut self` (de client onthoudt het laatste verzoek voor diagnose).
- **Retourwaarden zijn `serde_json::Value`** met precies de structuur die php-dna teruggeeft — geen getypeerde modellen. Sleutelnamen, tekstvlaggen (`"true"` / `"false"`), opgemaakte bedragen (`"11.9900"`) en de enveloppen zijn identiek:
  - succes: `{"result": "OK", "data": {...}}`
  - fout: `{"result": "ERROR", "error": {"Code": "...", "Message": "...", "Details": "..."}}`
  - enkele methoden gebruiken een andere envelop, overgenomen van php-dna: `get_current_balance` geeft een `OperationResult`-object terug, `check_availability` geeft bij succes een kale lijst terug, `get_reseller_details` geeft zijn velden op het hoogste niveau terug.
- **Fouten worden teruggegeven, nooit gegooid.** Gatewayfouten, HTTP-fouten en netwerkstoringen komen allemaal terug als foutenvelop; niets veroorzaakt een panic.
- **Gestructureerde invoer** (contacten, nameserverlijsten, filters, TLD-attributen, recordinhoud) is een `&Value`, opgebouwd met `json!`. Contactmaps gebruiken de sleutels van php-dna (`FirstName`, `EMail`, `AddressLine1`, `PhoneCountryCode`, ...); zowel de platte als de geneste vorm (`Address.Line1`, `Phone.Phone.Number`) wordt geaccepteerd. Nameserverlijsten mogen een lijst of een map zijn (de waarden worden gebruikt).
- **Geen standaardargumenten in Rust.** Elke methode neemt de volledige argumentenlijst van php-dna in dezelfde volgorde. De standaardwaarden van php-dna worden als constanten geëxporteerd: `DEFAULT_TTL` (3600), `DEFAULT_REASON` ("Owner request"), `DEFAULT_FORWARD_TYPE` ("Standard"), `DEFAULT_CURRENCY` ("USD"), `DEFAULT_TLD_COUNT` (20), `DEFAULT_COMMAND` ("create"). Voor `register_with_contact_info`, dat vier optionele argumenten heeft, bestaat ook `register_with_contact_info_opts`, dat een `RegisterOptions`-struct aanneemt waarvan `Default` de standaardwaarden van php-dna zijn.

## Gebruik

Alle fragmenten gaan uit van `use domainnameapi::{json, DomainNameApi};` en een client `let mut dna = DomainNameApi::new(...)`.

#### Domeinregistratie

Let op: voor .tr-domeinen zijn extra parameters nodig. Het argument met extra attributen wordt gebruikt voor domeinnamen die aanvullende informatie vereisen, zoals .tr.

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

Geef voor een `.tr`-domein de TRABIS-attributen mee. De bibliotheek stuurt alleen wat de gateway accepteert (`TRABISDOMAINCATEGORY`, `TRABISCITIZENID`, `TRABISTAXOFFICE`, `TRABISTAXNUMBER`), zet `1`/`0` om naar `Owner`/`Corporate` en corrigeert de oude spelling `TRABISCITIZIENID`:

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
<summary>Voorbeelduitvoer voor domeinregistratie</summary>

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

#### Domeinverlenging

```rust
let result = dna.renew("example.com", 1);
```

<details>
<summary>Voorbeelduitvoer voor domeinverlenging</summary>

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

#### Domeinoverdracht

```rust
let result = dna.transfer("example.com", "Xy9#mK2$", 3);

// php-dna's facade sends no contacts with a transfer; to send them:
let result = dna.transfer_with_contacts("example.com", "Xy9#mK2$", 3, &contacts);
```

<details>
<summary>Voorbeelduitvoer voor domeinoverdracht</summary>

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

#### Overdrachtsbeheer

```rust
// Can the domain be transferred with this auth code?
let check = dna.check_transfer("example.com", "Xy9#mK2$");

// Cancel an incoming transfer, approve or reject an outgoing one
let result = dna.cancel_transfer("example.com");
let result = dna.approve_transfer("example.com");
let result = dna.reject_transfer("example.com");
```

<details>
<summary>Voorbeelduitvoer voor overdrachtsbeheer</summary>

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

`check_transfer` geeft `"result": "ERROR"` met een `error` (`TRANSFER_NOT_AVAILABLE`) terug wanneer het register meldt dat het domein niet overdraagbaar is; `data` is in beide gevallen gevuld. `cancel_transfer`, `approve_transfer` en `reject_transfer` geven terug:

```json
{"result": "OK", "data": {"DomainName": "example.com", "Status": "Cancelled"}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Domeinlijst

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

`PageNumber` begint bij 0. De eigen gatewayparameters `MaxResultCount`, `SkipCount` en `Sorting`, en filters zoals `DomainName`, kunnen ook worden meegegeven.

<details>
<summary>Voorbeelduitvoer voor domeinlijst</summary>

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

#### TLD-lijst

```rust
let result = dna.get_tld_list(100);
```

<details>
<summary>Voorbeelduitvoer voor TLD-lijst</summary>

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

#### Beschikbaarheidscontrole van domeinen

```rust
let result = dna.check_availability(["hello", "world123x0"], ["com", "net"], 1, "create");
```

<details>
<summary>Voorbeelduitvoer voor beschikbaarheidscontrole</summary>

Bij succes is het resultaat een kale lijst (zonder `result`-sleutel); bij een fout is het de foutenvelop.

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

#### Domeindetails

```rust
let result = dna.get_details("example.com");

// Re-read the domain from the registry (same result shape):
let result = dna.sync_from_registry("example.com");
```

<details>
<summary>Voorbeelduitvoer voor domeindetails</summary>

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

#### Nameservers wijzigen

```rust
let result = dna.modify_name_server("example.com", &json!(["ns1.example.com", "ns2.example.com"]));
```

<details>
<summary>Voorbeelduitvoer voor nameservers wijzigen</summary>

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

#### Domeinvergrendeling inschakelen

```rust
let result = dna.enable_theft_protection_lock("example.com");
```

<details>
<summary>Voorbeelduitvoer voor vergrendeling inschakelen</summary>

```json
{"result": "OK", "data": {"LockStatus": true}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Domeinvergrendeling uitschakelen

```rust
let result = dna.disable_theft_protection_lock("example.com");
```

<details>
<summary>Voorbeelduitvoer voor vergrendeling uitschakelen</summary>

```json
{"result": "OK", "data": {"LockStatus": false}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Child-nameserver toevoegen

```rust
let result = dna.add_child_name_server("example.com", "ns1.example.com", "1.2.3.4");
```

<details>
<summary>Voorbeelduitvoer voor child-nameserver toevoegen</summary>

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

#### Child-nameserver verwijderen

```rust
let result = dna.delete_child_name_server("example.com", "ns1.example.com");
```

<details>
<summary>Voorbeelduitvoer voor child-nameserver verwijderen</summary>

```json
{"result": "OK", "data": {"NameServer": "ns1.example.com"}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Child-nameserver bijwerken

```rust
let result = dna.modify_child_name_server("example.com", "ns1.example.com", "1.2.3.4");
```

<details>
<summary>Voorbeelduitvoer voor child-nameserver bijwerken</summary>

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

#### Domeinprivacy wijzigen

```rust
let result = dna.modify_privacy_protection_status("example.com", true, "Owner request");
```

<details>
<summary>Voorbeelduitvoer voor domeinprivacy wijzigen</summary>

```json
{"data": {"PrivacyProtectionStatus": true}, "result": "OK"}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Domeincontacten ophalen

```rust
let result = dna.get_contacts("example.com");
```

<details>
<summary>Voorbeelduitvoer voor domeincontacten ophalen</summary>

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

#### Domeincontacten opslaan

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
<summary>Voorbeelduitvoer voor domeincontacten opslaan</summary>

```json
{"result": "OK", "data": {"contacts": []}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### DNS-zonerecords

De gateway slaat records op als **sets** (één naam + type met één of meer waarden). Een set schrijven vervangt al zijn waarden, en het verwijderen van een willekeurige waarde verwijdert de hele set. De `*_value`-methoden doen het lezen-samenvoegen-schrijven voor u, zodat een paneel records één voor één kan behandelen. Namen mogen relatief zijn (`www`), `@` / leeg voor de apex, of volledig gekwalificeerd; ze worden teruggebracht tot het relatieve label dat de gateway verwacht. Ondersteunde types: A, AAAA, ALIAS, CAA, CNAME, DS, MX, NAPTR, PTR, SRV, TLSA, TXT. Wijzigingen worden direct toegepast wanneer `apply` `true` is (standaard van php-dna); roep anders `apply_resource_records` aan. Zone-aanroepen liggen minstens één seconde uit elkaar, omdat de gateway pieken in verzoeken beperkt.

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
<summary>Voorbeelduitvoer voor DNS-zonerecords</summary>

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

Een waarde die al aanwezig is (of bij verwijderen ontbreekt) verandert niets:

```json
{"result": "OK", "data": {"Name": "", "Type": "MX", "Contents": ["10 mx1.example.com."], "Changed": false}}
```

Een niet-ondersteund type wordt geweigerd vóór enig verzoek:

```json
{"result": "ERROR", "error": {"Code": "ADD_ZONE", "Message": "Record type \"NS\" is not supported by the DNS zone API. Supported types: A, AAAA, ALIAS, CAA, CNAME, DS, MX, NAPTR, PTR, SRV, TLSA, TXT.", "Details": "..."}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### URL-doorverwijzing

```rust
let current = dna.get_forwarding("example.com");
let result = dna.set_forwarding("example.com", "https://www.example.org", "Standard"); // or "Frame" (masked)
let result = dna.delete_forwarding("example.com");
```

<details>
<summary>Voorbeelduitvoer voor URL-doorverwijzing</summary>

```json
{"result": "OK", "data": {"Enabled": true, "RedirectAddress": "https://www.example.org/landing", "ForwardType": "Frame"}}
```

Als er geen doorverwijzing is ingesteld, geeft `get_forwarding` `{"result": "OK", "data": {"Enabled": false, "RedirectAddress": "", "ForwardType": ""}}` terug. `set_forwarding` geeft `{"result": "OK", "data": {"RedirectAddress": "...", "ForwardType": "Standard"}}` terug.

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Resellergegevens ophalen

```rust
let result = dna.get_reseller_details();
let balance = dna.get_current_balance("USD"); // "USD", "TRY", "EUR" or "GBP"
```

<details>
<summary>Voorbeelduitvoer voor resellergegevens ophalen</summary>

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

Bij een fout is `OperationResult` `"FAILED"`, bevat `ErrorCode` de code (bijv. `"API_429"`) en is `Balance` `"0.00"`.

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Hulpfuncties (zonder netwerk)

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
<summary>Voorbeelduitvoer voor validate_contact</summary>

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

## Foutafhandeling

Elke netwerkmethode geeft een waarde terug; controleer `result`:

```rust
let out = dna.get_details("missing.com");
if out["result"] == "ERROR" {
    println!("{}", out["error"]["Code"]);    // "API_404"
    println!("{}", out["error"]["Message"]); // "The domain name you requested could not be found. ..."
    println!("{}", out["error"]["Details"]);
}
```

- HTTP-fouten gebruiken de code `API_<status>` (`API_400`, `API_404`, `API_429`, `API_500`, ...). Een lege foutbody wordt een leesbare melding zoals `HTTP 429 - Too Many Requests`.
- Een 301/302-antwoord betekent onjuiste inloggegevens: `Invalid API credentials`.
- Validatiefouten van de gateway worden samengevoegd in `Details`, met de naam van het veld, bijv. `Contacts Phone - The field Phone must be a string with a maximum length of 16.`
- Netwerkstoringen zijn fouten, nooit een leeg succes. Een time-out luidt `Could not reach the DomainNameAPI service: the connection timed out. (...)`; andere storingen luiden `Curl error during request: ...`. De envelopcode is `API_<curl-foutnummer>` (`API_28` bij een time-out, `API_6` bij DNS, `API_7` bij een geweigerde verbinding), en `get_parsed_response_data()["Code"]` bevat `CURL_<n>`.
- Lokale validatie (niet-ondersteund recordtype, ontbrekend doorverwijzingsadres, geen domeinen om te controleren) faalt vóór enig verzoek met een beschrijvende code (`ADD_ZONE`, `SET_FORWARD`, `AVAILABILITY`, ...).

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

### Eigen transport

De HTTP-laag is de trait `Transport`. Standaard is dat `UreqTransport` (blokkerende `ureq` met rustls, time-outs van 30 s totaal en 10 s voor de verbinding, redirects worden niet gevolgd). Vervang deze voor proxy's, eigen clients of tests:

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

Geef `true` (of `"true"`, `"1"`, `"on"`, `"yes"`) mee als derde constructorargument om de OTE-testomgeving op `https://ote.domainresellerapi.com/api/v1` te gebruiken. Live-aanroepen gaan naar `https://api.domainresellerapi.com/api/v1`.

## Tests

De testsuite heeft geen netwerk en geen inloggegevens nodig:

```bash
cargo test
```

Ze bevat unittests voor de HTTP-laag, de foutvertaling, het maskeren van telemetrie en de hulpfuncties, plus een conformiteitstest die alle 126 php-dna-fixtures uit `tests/conformance/` afspeelt en controleert dat elk verzonden verzoek en elke retourwaarde overeenkomt met php-dna.

## Foutrapportage (telemetrie)

Als een API-aanroep mislukt, kan de bibliotheek een foutrapport sturen naar de eigen foutregistratie van DomainNameAPI (Sentry). Een klein willekeurig deel van de geslaagde aanroepen (4%, plus elke aanroep die langer dan 1 seconde duurt) wordt als prestatiegegeven verstuurd. Dit helpt DomainNameAPI problemen te vinden en op te lossen. Foutrapportage staat **standaard aan**.

> ℹ️ In versie 1.0.0 bestaat het Sentry-project voor de Rust-bibliotheek nog niet, dus de ingebouwde DSN is leeg en er wordt niets verstuurd. Het gedrag hieronder geldt zodra die is ingesteld (of als u zelf een DSN instelt met `set_telemetry_dsn`).

**Wat wordt verstuurd:**

- foutcode en foutmelding
- pad van het aangeroepen endpoint en de HTTP-methode
- duur van de aanroep
- applicatienaam (`DomainNameAPI-Rust`) en bibliotheekversie
- reseller-ID en domeinnaam
- het openbare IP-adres van de server (op de achtergrond opgevraagd via `api.ipify.org`)
- de gebruikersnaam van het besturingssysteem
- het API-verzoek en -antwoord, inclusief de contactgegevens van het domein; alleen inloggegevens worden gemaskeerd — EPP-/authcodes, API-sleutels, wachtwoorden, geheimen en tokens worden vervangen door `***`

**Wat nooit wordt verstuurd:** EPP-/authcodes, API-sleutels, wachtwoorden of tokens in leesbare vorm.

Rapporten worden vanuit een achtergrondthread met korte time-outs verstuurd; ze blokkeren uw aanroep nooit en laten die nooit mislukken. Verwachte fouten zoals "domein niet gevonden" of "onvoldoende saldo" worden niet gerapporteerd.

**Waarom en hoe lang:** de gegevens worden alleen gebruikt voor gebruiksmeting en vroege diagnose van problemen, en worden 30 dagen bewaard op de eigen Sentry-server van DomainNameAPI, waarna ze automatisch worden verwijderd.

**Uitschakelen of beperken:**

```rust
let mut dna = DomainNameApi::new(reseller_id, api_key, false);
dna.set_error_reporting_enabled(false); // nothing is sent

// or keep it on but send only the error code and message, endpoint,
// timing, versions and reseller ID (no request/response data, domain, IP or user name)
dna.set_telemetry_minimal(true);
```

> 🔒 **Beveiliging:** controle van TLS-certificaten staat aan. Op een host met een verouderde CA-bundel kan die worden uitgeschakeld met `dna.set_ssl_verification(false);` — dit wordt afgeraden; de CA-bundel bijwerken is de betere oplossing.

## Return en Foutcodes met Beschrijvingen

| Code | Beschrijving | Detail |
|------|-------------|--------|
| 1000 | Commando succesvol voltooid | Operatie succesvol. |
| 1001 | Commando succesvol voltooid; actie in behandeling | Operatie succesvol. De operatie is echter momenteel in de wachtrij geplaatst voor voltooiing. |
| 2003 | Vereiste parameter ontbreekt | Parameter ontbreekt error. Bijvoorbeeld: geen telefoon invoer in contactinformatie. |
| 2105 | Object komt niet in aanmerking voor verlenging | Domeinstatus is niet geschikt voor verlenging, vergrendeld voor update operaties. Status mag niet "clientupdateprohibited" zijn. Kan te wijten zijn aan andere statusvoorwaarden. |
| 2200 | Authenticatiefout | Autorisatiefout, beveiligingscode is onjuist of domein is geregistreerd bij een andere registrar. |
| 2302 | Object bestaat | Domeinnaam of nameserver informatie bestaat al in database. Kan niet geregistreerd worden. |
| 2303 | Object bestaat niet | Domeinnaam of nameserver informatie bestaat niet in database. Nieuwe registratie vereist. |
| 2304 | Objectstatus verbiedt operatie | Domeinstatus is niet geschikt voor updates, vergrendeld voor update operaties. Status mag niet "clientupdateprohibited" zijn. Kan te wijten zijn aan andere statusvoorwaarden. |

| CODE | DETAIL |
|------|--------|
| 101 | Systeemfout gedetailleerd voor ({0})! |
| 102 | Meerdere fouten gedetailleerd voor ({0})! |
| 103 | Onbekende fouten gedetailleerd voor ({0})! |
| 200 | API commando niet gevonden ({0})! |
| 210 | API service niet gevonden ({0})! |
| 211 | API service provider niet ingesteld! |
| 300 | Reseller niet gevonden! |
| 301 | Uw huidige IP-adres is niet geautoriseerd voor toegang. Zorg ervoor dat u verbinding maakt vanaf een geautoriseerd IP-adres en probeer opnieuw |
| 310 | TLD wordt niet ondersteund! |
| 320 | API niet gevonden! |
| 321 | Valuta wordt niet ondersteund! |
| 330 | Vereiste parameter(s) niet ingesteld ({0}). |
| – | Zorg ervoor dat u alle contactobjecten volledig verzendt |
| 340 | Prijsdefinitie niet gevonden ({0}[{1}]-{2}{3}). |
| 350 | Onvoldoende reseller saldo. (Reseller Id : {0} - Huidig Saldo : {1} {2}). |
| 350 | Boekhoudvaluta komt niet overeen of het saldo is niet voldoende. |
| 360 | Ongeldige API aanvraag voor veld ({0}). |
| 360 | API quota overschreden! |
| 361 | Throttled error! |
| 362 | Premium domein is momenteel niet beschikbaar voor registratie. |
| 363 | Operatie geannuleerd omdat domein in auto-vernieuwingsperiode is |
| 364 | Dit domein is momenteel niet beschikbaar voor transactie vanwege een probleem met het register |

| CODE | DETAIL |
|------|--------|
| 400 | Ongeldig contact ({0}). |
| 401 | Contactinfo kan niet synchroniseren. |
| 402 | Geen toegang to contactinformatie van register. |
| 403 | HET SYSTEEM HEEFT ONTBREKENDE INFORMATIE. VOER UW STANDAARDINFORMATIE IN OF NEEM CONTACT OP MET HET ONDERSTEUNINGSTEAM. |
| 404 | DOMEINCONTACT KAN NIET WORDEN BIJGEWERKT. TOESTEMMING VAN HET REGISTER. |
| 410 | Contact niet gevonden. |
| 410 | Contact niet gevonden ({0}). |
| 420 | Ongeldig Api commando voor contact {0}. |
| 430 | Contact api niet gevonden. |
| 440 | Contact is niet gesynchroniseerd. |
| 450 | Te veel domeincontacten.. ! |
| 451 | Kon niet doorgaan met contact update. |

| CODE | DETAIL |
|------|--------|
| 500 | Ongeldig domein id. |
| 500 | Ongeldig domein id ({0}). |
| 501 | Domein kon niet worden gesynchroniseerd({0}) |
| 502 | Interne transfer mislukt |
| 503 | Domeinregistratie is niet beschikbaar. |
| 504 | Domeininformatie komt niet overeen .. Voor :{0} |
| 505 | Voer het IP-adres in. |
| 506 | Domeintransfer is gestart maar contactinfo kon niet worden gelezen .. |
| 507 | Autorisatiefout. |
| 510 | Domein niet gevonden. |
| 511 | Verlopen domeinen kunnen niet worden gevonden. |
| 512 | Domein is niet verlengbaar. |
| 513 | Domein is niet in updatebare status. Het moet actief zijn om te worden bijgewerkt |
| 514 | Redemption Period verwacht. |
| 520 | Ongeldig Api commando voor domein "{0}". |
| 530 | Ongeldige domeinperiode. Periode moet {0} tot {1} jaar zijn. Gevraagde periode is {2} jaar. |
| 540 | Domein kan niet worden verlengd tot meer dan {0} jaar vanaf huidige datum. |
| 550 | Ongeldige domeinnaam. Domein moet {0} tot {1} karakters lang zijn. Gevraagde domeinnaam is {2} karakters. |
| 560 | Ongeldig nameserver aantal. Domein moet {0} tot {1} nameservers hebben. Gevraagd nameserver aantal is {2}. |
| 561 | Geen nameserver informatie gevonden in de binnenkomende aanvraag |
| 570 | Idn niet ondersteund voor tld "{0}". |
| 571 | Periode ongeldig. |
| 572 | Commando ongeldig. |
| 573 | Domeinnamen niet gevonden. |
| 574 | TLD namen niet gevonden. |
| 575 | Domein is niet in updatebare status. Het moet actief zijn voor nameservers om te worden bijgewerkt |
| 576 | Domein kan worden vernieuwd in de laatste 1 maand voor de vervaldatum |
| 580 | Transfer niet ondersteund voor tld "{0}". |
| 581 | Child nameserver niet gevonden |
| 582 | Transfer gestart door andere reseller. |
| 583 | Transfer niet geïnitialiseerd. Neem contact op met het ondersteuningsteam. |
| 584 | Objectstatus verbiedt operatie |
| 590 | Auth code is vereist voor deze transfer. |
| 591 | Auth code is niet geldig. |
| 592 | Transfer lock bestaat op domein. |
| 593 | Domein kan niet worden getransfereerd, Statusinformatie en Transfer Lock moeten voldoen aan de vereiste criteria. (Statusinformatie: #ok) |
| 594 | Domein nameserver adres kon niet worden opgelost ({0}) |
| 595 | Contactinformatie kan niet worden gelezen (whois.registrar.tld) Zorg ervoor dat privacy bescherming status open is |
| 596 | Contactinformatie kon niet worden geverifieerd |
| 597 | Tld niet gevonden |
| 598 | Onbekende fout opgetreden |
| 599 | Domein Forward niet gevonden |

## Licentie

Apache License 2.0 — zie [LICENSE](LICENSE).
