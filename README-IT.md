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

# rust-dna — client DomainNameAPI per Rust

Client Rust per l'API REST del registrar di domini DomainNameAPI. È un porting fedele della libreria PHP di riferimento [php-dna](https://github.com/domainreseller/php-dna): gli stessi metodi, le stesse richieste in rete e gli stessi valori di ritorno. Ogni porting viene verificato con le 134 fixture di conformità di php-dna.

## 📦 Download — usa sempre le Releases!

⬇️ **Scarica l'ultima versione testata qui: https://github.com/domainreseller/rust-dna/releases/latest**

> ⚠️ **Non** usare il pulsante verde **Code → Download ZIP** — scarica il ramo di sviluppo grezzo. I pacchetti di release sono versionati, testati e pronti per la produzione.

## Guida all'installazione e all'integrazione

### Requisiti minimi

- Rust 1.83 o successivo (edizione 2021)
- Nessun runtime asincrono: l'API è bloccante
- Dipendenze di runtime: `ureq` (HTTP, TLS rustls), `rustls` e `serde_json`. Non serve OpenSSL.

### Installazione

**A) Dipendenza Git (fissata a un tag di release)**

```toml
[dependencies]
domainnameapi = { git = "https://github.com/domainreseller/rust-dna", tag = "v1.0.1" }
```

**B) Release scaricata**

Estrai l'archivio della release accanto al tuo progetto e indica a Cargo dove si trova:

```toml
[dependencies]
domainnameapi = { path = "../rust-dna" }
```

La crate riesporta `serde_json::Value` e la macro `json!`, quindi non è necessario aggiungere `serde_json` (puoi farlo se preferisci).

## 🔑 Credenziali API — Reseller ID e API Key

La libreria comunica con l'API REST di DomainNameAPI e richiede esattamente due credenziali, passate al costruttore:

| Parametro | Valore |
|---|---|
| Primo | Reseller ID — UUID come `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` |
| Secondo | API Key |

> 💡 Trovi il tuo **Reseller ID** e la tua **API Key** nel pannello DomainNameAPI, sotto **Impostazioni API**.
> ⚠️ Queste sono **credenziali API** — i dati di accesso al pannello **non** funzionano qui.

```rust
use domainnameapi::DomainNameApi;

let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";
let api_key = "your-api-key";

// Live environment
let mut dna = DomainNameApi::new(reseller_id, api_key, false);

// Test (OTE) environment
let mut dna = DomainNameApi::new(reseller_id, api_key, true);
```

Entrambe le credenziali vengono ripulite dagli spazi iniziali e finali. Il terzo argomento accetta anche stringhe, perché i pannelli spesso salvano le caselle di controllo come testo: `"true"`, `"1"`, `"on"` e `"yes"` (senza distinzione tra maiuscole e minuscole) selezionano l'ambiente di test, qualsiasi altro valore (compresi `"false"` e `"off"`) seleziona l'ambiente live.

## Avvio rapido

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

Ogni esempio in [examples/](examples) si esegue con `cargo run --example <nome>`, ad esempio `cargo run --example get_details`.

## Struttura dell'API

- **Un solo tipo di client**, `DomainNameApi`. I metodi prendono `&mut self` (il client ricorda l'ultima richiesta per la diagnostica).
- **I valori di ritorno sono `serde_json::Value`** con esattamente la struttura restituita da php-dna — non modelli tipizzati. Nomi delle chiavi, flag come stringhe (`"true"` / `"false"`), importi formattati (`"11.9900"`) e buste sono identici:
  - successo: `{"result": "OK", "data": {...}}`
  - errore: `{"result": "ERROR", "error": {"Code": "...", "Message": "...", "Details": "..."}}`
  - alcuni metodi usano una busta diversa, ripresa da php-dna: `get_current_balance` restituisce un oggetto `OperationResult`, `check_availability` restituisce una semplice lista in caso di successo, `get_reseller_details` restituisce i suoi campi al livello superiore.
- **Gli errori vengono restituiti, mai lanciati.** Errori del gateway, errori HTTP e guasti di rete tornano tutti come busta di errore; nulla causa un panic. Registrazione, trasferimento e rinnovo restituiscono un errore anche quando il gateway risponde HTTP 200 con `success: false`.
- **Gli input strutturati** (contatti, liste di nameserver, filtri, attributi TLD, contenuti dei record) sono `&Value` costruiti con `json!`. Le mappe dei contatti usano le chiavi di php-dna (`FirstName`, `EMail`, `AddressLine1`, `PhoneCountryCode`, ...); sono accettate sia la forma piatta sia quella annidata (`Address.Line1`, `Phone.Phone.Number`). Le liste di nameserver possono essere una lista o una mappa (vengono usati i valori).
- **Niente argomenti predefiniti in Rust.** Ogni metodo prende l'elenco completo degli argomenti di php-dna nello stesso ordine. I valori predefiniti di php-dna sono esportati come costanti: `DEFAULT_TTL` (3600), `DEFAULT_REASON` ("Owner request"), `DEFAULT_FORWARD_TYPE` ("Standard"), `DEFAULT_CURRENCY` ("USD"), `DEFAULT_TLD_COUNT` (20), `DEFAULT_COMMAND` ("create"). `register_with_contact_info`, che ha quattro argomenti opzionali, dispone anche di `register_with_contact_info_opts`, che accetta una struttura `RegisterOptions` il cui `Default` corrisponde ai valori predefiniti di php-dna.

## Utilizzo

Tutti gli esempi presuppongono `use domainnameapi::{json, DomainNameApi};` e un client `let mut dna = DomainNameApi::new(...)`.

#### Registrazione dominio

Nota: per i domini .tr sono richiesti parametri aggiuntivi. L'argomento degli attributi aggiuntivi serve per i nomi di dominio che richiedono informazioni extra, come .tr.

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

Per un dominio `.tr` passa gli attributi TRABIS. La libreria invia solo ciò che il gateway accetta (`TRABISDOMAINCATEGORY`, `TRABISCITIZENID`, `TRABISTAXOFFICE`, `TRABISTAXNUMBER`), converte `1`/`0` in `Owner`/`Corporate` e corregge la vecchia grafia `TRABISCITIZIENID`:

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
<summary>Output di esempio per la registrazione dominio</summary>

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

#### Rinnovo dominio

```rust
let result = dna.renew("example.com", 1);
```

<details>
<summary>Output di esempio per il rinnovo dominio</summary>

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

#### Trasferimento dominio

I trasferimenti non inviano mai i dati dei contatti: il registrar acquisisce i contatti dal registrar precedente, quindi ogni richiesta di trasferimento contiene `contacts: []`.

```rust
let result = dna.transfer("example.com", "Xy9#mK2$", 3);
```

<details>
<summary>Output di esempio per il trasferimento dominio</summary>

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

#### Gestione dei trasferimenti

```rust
// Can the domain be transferred with this auth code?
let check = dna.check_transfer("example.com", "Xy9#mK2$");

// Cancel an incoming transfer, approve or reject an outgoing one
let result = dna.cancel_transfer("example.com");
let result = dna.approve_transfer("example.com");
let result = dna.reject_transfer("example.com");
```

<details>
<summary>Output di esempio per la gestione dei trasferimenti</summary>

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

`check_transfer` restituisce `"result": "ERROR"` con un `error` (`TRANSFER_NOT_AVAILABLE`) quando il registro segnala che il dominio non è trasferibile; `data` è compilato in entrambi i casi. `cancel_transfer`, `approve_transfer` e `reject_transfer` restituiscono:

```json
{"result": "OK", "data": {"DomainName": "example.com", "Status": "Cancelled"}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Elenco domini

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

`PageNumber` parte da 0. Si possono passare anche i parametri propri del gateway `MaxResultCount`, `SkipCount` e `Sorting`, e filtri come `DomainName`.

<details>
<summary>Output di esempio per l'elenco domini</summary>

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

#### Elenco TLD

```rust
let result = dna.get_tld_list(100);
```

<details>
<summary>Output di esempio per l'elenco TLD</summary>

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

#### Verifica disponibilità dominio

```rust
let result = dna.check_availability(["hello", "world123x0"], ["com", "net"], 1, "create");
```

<details>
<summary>Output di esempio per la verifica disponibilità</summary>

In caso di successo il risultato è una semplice lista (senza chiave `result`); in caso di errore è la busta di errore.

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

#### Dettagli dominio

```rust
let result = dna.get_details("example.com");

// Re-read the domain from the registry (same result shape):
let result = dna.sync_from_registry("example.com");
```

<details>
<summary>Output di esempio per i dettagli dominio</summary>

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

#### Modifica nameserver

```rust
let result = dna.modify_name_server("example.com", &json!(["ns1.example.com", "ns2.example.com"]));
```

<details>
<summary>Output di esempio per la modifica nameserver</summary>

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

#### Attiva blocco dominio

```rust
let result = dna.enable_theft_protection_lock("example.com");
```

<details>
<summary>Output di esempio per l'attivazione del blocco</summary>

```json
{"result": "OK", "data": {"LockStatus": true}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Disattiva blocco dominio

```rust
let result = dna.disable_theft_protection_lock("example.com");
```

<details>
<summary>Output di esempio per la disattivazione del blocco</summary>

```json
{"result": "OK", "data": {"LockStatus": false}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Aggiungi child nameserver

```rust
let result = dna.add_child_name_server("example.com", "ns1.example.com", "1.2.3.4");
```

<details>
<summary>Output di esempio per l'aggiunta di un child nameserver</summary>

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

#### Elimina child nameserver

```rust
let result = dna.delete_child_name_server("example.com", "ns1.example.com");
```

<details>
<summary>Output di esempio per l'eliminazione di un child nameserver</summary>

```json
{"result": "OK", "data": {"NameServer": "ns1.example.com"}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Aggiorna child nameserver

```rust
let result = dna.modify_child_name_server("example.com", "ns1.example.com", "1.2.3.4");
```

<details>
<summary>Output di esempio per l'aggiornamento di un child nameserver</summary>

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

#### Modifica privacy dominio

`status` è un `bool` oppure un valore come lo salvano i pannelli: `"1"`, `"on"`, `"true"`, `"yes"` e `1` significano attivo, qualsiasi altro valore disattivo. L'argomento del motivo (reason) è mantenuto solo per compatibilità e non viene inviato, perché il gateway non ha questo campo.

```rust
let result = dna.modify_privacy_protection_status("example.com", true, "Owner request");
```

<details>
<summary>Output di esempio per la modifica della privacy</summary>

```json
{"data": {"PrivacyProtectionStatus": true}, "result": "OK"}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Ottieni contatti dominio

```rust
let result = dna.get_contacts("example.com");
```

<details>
<summary>Output di esempio per l'ottenimento dei contatti</summary>

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

#### Salva contatti dominio

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
<summary>Output di esempio per il salvataggio dei contatti</summary>

```json
{"result": "OK", "data": {"contacts": []}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Record della zona DNS

Il gateway memorizza i record come **insiemi** (un nome + tipo con uno o più valori). Scrivere un insieme sostituisce tutti i suoi valori, ed eliminare un qualsiasi valore rimuove l'intero insieme. I metodi `*_value` eseguono per te la lettura-unione-scrittura, così un pannello può gestire i record uno alla volta. I nomi possono essere relativi (`www`), `@` / vuoti per l'apex, oppure completamente qualificati; vengono ridotti all'etichetta relativa che il gateway si aspetta. Tipi supportati: A, AAAA, ALIAS, CAA, CNAME, DS, MX, NAPTR, PTR, SRV, TLSA, TXT. Le modifiche vengono applicate subito quando `apply` è `true` (predefinito di php-dna); altrimenti chiama `apply_resource_records`. Le chiamate di zona sono distanziate di almeno un secondo perché il gateway limita le raffiche di richieste. Se l'applicazione automatica fallisce, il metodo restituisce l'errore dell'applicazione (la modifica resta in attesa). I risultati riportano il tipo in maiuscolo e il TTL effettivamente inviato (sotto 1 diventa 3600, sopra 86400 diventa 86400).

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
<summary>Output di esempio per i record della zona DNS</summary>

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

Un valore già presente (o assente, in caso di rimozione) non cambia nulla:

```json
{"result": "OK", "data": {"Name": "", "Type": "MX", "Contents": ["10 mx1.example.com."], "Changed": false}}
```

Un tipo non supportato viene rifiutato prima di qualsiasi richiesta:

```json
{"result": "ERROR", "error": {"Code": "ADD_ZONE", "Message": "Record type \"NS\" is not supported by the DNS zone API. Supported types: A, AAAA, ALIAS, CAA, CNAME, DS, MX, NAPTR, PTR, SRV, TLSA, TXT.", "Details": "..."}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Inoltro URL

```rust
let current = dna.get_forwarding("example.com");
let result = dna.set_forwarding("example.com", "https://www.example.org", "Standard"); // or "Frame" (masked)
let result = dna.delete_forwarding("example.com");
```

<details>
<summary>Output di esempio per l'inoltro URL</summary>

```json
{"result": "OK", "data": {"Enabled": true, "RedirectAddress": "https://www.example.org/landing", "ForwardType": "Frame"}}
```

Se non è configurato alcun inoltro (il gateway risponde `Redirect information not found`), `get_forwarding` restituisce `{"result": "OK", "data": {"Enabled": false, "RedirectAddress": "", "ForwardType": ""}}`; qualsiasi altro errore, come un dominio inesistente, viene restituito come errore. `set_forwarding` restituisce `{"result": "OK", "data": {"RedirectAddress": "...", "ForwardType": "Standard"}}`. `delete_forwarding` restituisce `{"result": "OK", "data": {"DomainName": "example.com", "Removed": true}}`, oppure `"Removed": false` se non era impostato alcun inoltro.

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Ottieni dettagli rivenditore

```rust
let result = dna.get_reseller_details();
let balance = dna.get_current_balance("USD"); // "USD" or "TRY"
```

<details>
<summary>Output di esempio per i dettagli del rivenditore</summary>

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

L'account riporta i saldi in USD e TRY; qualsiasi altra valuta (EUR, GBP, ...) fallisce con `ErrorCode` `"BALANCE"`. In caso di errore `OperationResult` è `"FAILED"`, `ErrorCode` contiene il codice (ad es. `"API_429"`) e `Balance` è `"0.00"`.

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Utilità (senza rete)

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
<summary>Output di esempio per validate_contact</summary>

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

## Gestione degli errori

Ogni metodo di rete restituisce un valore; controlla `result`:

```rust
let out = dna.get_details("missing.com");
if out["result"] == "ERROR" {
    println!("{}", out["error"]["Code"]);        // "API_404"
    println!("{}", out["error"]["GatewayCode"]); // "Dna.DomainService:Domain:10007"
    println!("{}", out["error"]["Message"]);     // "The domain name you requested could not be found. ..."
    println!("{}", out["error"]["Details"]);
}
```

- Gli errori HTTP usano il codice `API_<stato>` (`API_400`, `API_404`, `API_429`, `API_500`, ...). Un corpo di errore vuoto diventa un messaggio leggibile come `HTTP 429 - Too Many Requests`. Quando il gateway ha inviato un proprio codice di errore, la busta lo riporta anche come `GatewayCode` (ad es. `"Dna.DomainService:Domain:10007"`).
- Una risposta 301/302 indica credenziali errate: `Invalid API credentials`.
- Gli errori di validazione del gateway vengono riassunti in `Details` indicando il campo, ad es. `Contacts Phone - The field Phone must be a string with a maximum length of 16.`
- I guasti di rete sono errori, mai un successo vuoto. Un timeout produce `Could not reach the DomainNameAPI service: the connection timed out. (...)`; gli altri guasti producono `Curl error during request: ...`. Il loro codice è `CURL_<numero errore curl>` (`CURL_28` per un timeout, `CURL_6` per il DNS, `CURL_7` per una connessione rifiutata); `API_<n>` è riservato agli stati HTTP.
- La validazione locale (tipo di record non supportato, indirizzo di inoltro mancante, nessun dominio da verificare) fallisce prima di qualsiasi richiesta con un codice descrittivo (`ADD_ZONE`, `SET_FORWARD`, `AVAILABILITY`, ...).

### Diagnostica

```rust
dna.get_request_data();        // {"url", "method", "payload"} of the last request
dna.get_response_data();       // decoded last response, or {"raw_response": "<body>"}
dna.get_response_headers();    // {"url", "content_type", "http_code", "total_time", "headers"}
dna.get_parsed_response_data();// parsed body on success, {"Code", "Message", "Details"} on failure
dna.get_last_function();       // e.g. "request:GET domains/info"
dna.get_service_url();
dna.set_service_url("https://ote.domainresellerapi.com/api/v1");
```

### Transport personalizzato

Il livello HTTP è il trait `Transport`. Quello predefinito è `UreqTransport` (`ureq` bloccante con rustls, timeout di 30 s totali e 10 s di connessione, redirect non seguiti). Sostituiscilo per proxy, client personalizzati o test:

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

## Modalità test

Passa `true` (oppure `"true"`, `"1"`, `"on"`, `"yes"`) come terzo argomento del costruttore per usare l'ambiente di test OTE su `https://ote.domainresellerapi.com/api/v1`. Le chiamate live vanno a `https://api.domainresellerapi.com/api/v1`.

## Test

La suite di test non richiede né rete né credenziali:

```bash
cargo test
```

Contiene test unitari per il livello HTTP, la mappatura degli errori, il mascheramento della telemetria e le utilità, oltre a un test di conformità che riproduce tutte le 134 fixture di php-dna da `tests/conformance/` e verifica che ogni richiesta inviata e ogni valore di ritorno corrispondano a php-dna.

## Segnalazione errori (telemetria)

Quando una chiamata API fallisce, la libreria può inviare una segnalazione di errore al sistema di tracciamento errori di DomainNameAPI (Sentry). Una piccola quota casuale delle chiamate riuscite (4%, più ogni chiamata più lenta di 1 secondo) viene inviata come record di prestazioni. Questo aiuta DomainNameAPI a trovare e risolvere i problemi. La segnalazione degli errori è **attiva per impostazione predefinita**.

**Cosa viene inviato:**

- codice e messaggio di errore
- percorso dell'endpoint chiamato e metodo HTTP
- durata della chiamata
- nome dell'applicazione (`DomainNameAPI-Rust`) e versione della libreria
- ID rivenditore e nome di dominio
- l'indirizzo IP pubblico del server (ricavato in background tramite `api.ipify.org`)
- il nome utente del sistema operativo
- la richiesta e la risposta API, inclusi i dati dei contatti del dominio; vengono mascherate solo le credenziali — codici EPP/auth, chiavi API, password, segreti e token sono sostituiti con `***`

**Cosa non viene mai inviato:** codici EPP/auth, chiavi API, password o token in forma leggibile.

Le segnalazioni vengono inviate da un thread in background con timeout brevi; non bloccano mai la tua chiamata né la fanno fallire. Errori attesi come «dominio non trovato» o «saldo insufficiente» non vengono segnalati.

**Perché e per quanto tempo:** i dati sono usati solo per misurare l'utilizzo e diagnosticare tempestivamente i problemi, e vengono conservati per 30 giorni sul server Sentry di DomainNameAPI, dopodiché vengono eliminati automaticamente.

**Come disattivarla o ridurla:**

```rust
let mut dna = DomainNameApi::new(reseller_id, api_key, false);
dna.set_error_reporting_enabled(false); // nothing is sent

// or keep it on but send only the error code and message, endpoint,
// timing, versions and reseller ID (no request/response data, domain, IP or user name)
dna.set_telemetry_minimal(true);
```

> 🔒 **Sicurezza:** la verifica dei certificati TLS è attiva. Su un host con un bundle CA obsoleto può essere disattivata con `dna.set_ssl_verification(false);` — non è consigliato; aggiornare il bundle CA è la soluzione migliore.

## Codici di Ritorno ed Errore con Descrizioni

| Codice | Descrizione | Dettaglio |
|--------|-------------|-----------|
| 1000 | Comando completato con successo | Operazione riuscita. |
| 1001 | Comando completato con successo; azione in sospeso | Operazione riuscita. Tuttavia, l'operazione è attualmente in coda per il completamento. |
| 2003 | Parametro richiesto mancante | Errore parametro mancante. Ad esempio: nessuna voce telefonica nelle informazioni di contatto. |
| 2105 | L'oggetto non è idoneo per il rinnovo | Lo stato del dominio non è adatto per il rinnovo, bloccato per operazioni di aggiornamento. Lo stato non dovrebbe essere "clientupdateprohibited". Potrebbe essere dovuto ad altre condizioni di stato. |
| 2200 | Errore di autenticazione | Errore di autorizzazione, codice di sicurezza errato o dominio registrato con un altro registrar. |
| 2302 | L'oggetto esiste | Il nome di dominio o le informazioni del name server esistono già nel database. Non può essere registrato. |
| 2303 | L'oggetto non esiste | Il nome di dominio o le informazioni del name server non esistono nel database. È richiesta una nuova registrazione. |
| 2304 | Lo stato dell'oggetto vieta l'operazione | Lo stato del dominio non è adatto per gli aggiornamenti, bloccato per operazioni di aggiornamento. Lo stato non dovrebbe essere "clientupdateprohibited". Potrebbe essere dovuto ad altre condizioni di stato. |

| CODICE | DETTAGLIO |
|--------|-----------|
| 101 | Errore di sistema dettagliato per ({0})! |
| 102 | Errori multipli dettagliati per ({0})! |
| 103 | Errori sconosciuti dettagliati per ({0})! |
| 200 | Comando API non trovato ({0})! |
| 210 | Servizio API non trovato ({0})! |
| 211 | Provider di servizio API non impostato! |
| 300 | Rivenditore non trovato! |
| 301 | Il tuo indirizzo IP attuale non è autorizzato ad accedere. Assicurati di connetterti da un indirizzo IP autorizzato e riprova |
| 310 | TLD non supportato! |
| 320 | API non trovata! |
| 321 | Valuta non supportata! |
| 330 | Parametro/i richiesto/i non impostato/i ({0}). |
| – | Assicurati di inviare tutti gli oggetti di contatto completi |
| 340 | Definizione prezzo non trovata ({0}[{1}]-{2}{3}). |
| 350 | Saldo rivenditore insufficiente. (Id Rivenditore : {0} - Saldo Attuale : {1} {2}). |
| 350 | La valuta contabile non corrisponde o il saldo non è sufficiente. |
| 360 | Richiesta API non valida per il campo ({0}). |
| 360 | Quota API superata! |
| 361 | Errore di throttling! |
| 362 | Il dominio premium non è disponibile per la registrazione al momento. |
| 363 | Operazione annullata perché il dominio è nel periodo di rinnovo automatico |
| 364 | Questo dominio è attualmente non disponibile per la transazione a causa di un problema con il registro |

| CODICE | DETTAGLIO |
|--------|-----------|
| 400 | Contatto non valido ({0}). |
| 401 | Le informazioni di contatto non possono essere sincronizzate. |
| 402 | Nessun accesso alle informazioni di contatto dal registro. |
| 403 | IL SISTEMA HA INFORMAZIONI MANCANTI. INSERISCI LE TUE INFORMAZIONI PREDEFINITE O CONTATTA IL TEAM DI SUPPORTO. |
| 404 | IL CONTATTO DEL DOMINIO NON PUÒ ESSERE AGGIORNATO. PERMESSO DAL REGISTRO. |
| 410 | Contatto non trovato. |
| 410 | Contatto non trovato ({0}). |
| 420 | Comando Api non valido per il contatto {0}. |
| 430 | API contatto non trovata. |
| 440 | Il contatto non è sincronizzato. |
| 450 | Troppi contatti di dominio.. ! |
| 451 | Impossibile procedere con l'aggiornamento del contatto. |

| CODICE | DETTAGLIO |
|--------|-----------|
| 500 | ID dominio non valido. |
| 500 | ID dominio non valido ({0}). |
| 501 | Il dominio non può essere sincronizzato({0}) |
| 502 | Trasferimento interno fallito |
| 503 | La registrazione del dominio non è disponibile. |
| 504 | Le informazioni del dominio non corrispondono .. Prima :{0} |
| 505 | Inserisci l'indirizzo IP. |
| 506 | Il trasferimento del dominio è stato avviato ma le informazioni di contatto non possono essere lette .. |
| 507 | Errore di autorizzazione. |
| 510 | Dominio non trovato. |
| 511 | I domini scaduti non possono essere trovati. |
| 512 | Il dominio non è rinnovabile. |
| 513 | Il dominio non è in stato aggiornabile. Deve essere attivo per essere aggiornato |
| 514 | Periodo di redenzione previsto. |
| 520 | Comando Api non valido per il dominio "{0}". |
| 530 | Periodo di dominio non valido. Il periodo deve essere da {0} a {1} anni. Il periodo richiesto è {2} anni. |
| 540 | Il dominio non può essere esteso oltre {0} anni dalla data attuale. |
| 550 | Nome di dominio non valido. Il dominio deve essere lungo da {0} a {1} caratteri. Il nome di dominio richiesto è di {2} caratteri. |
| 560 | Numero di name server non valido. Il dominio deve avere da {0} a {1} name server. Il numero di name server richiesto è {2}. |
| 561 | Nessuna informazione name server trovata nella richiesta in arrivo |
| 570 | Idn non supportato per tld "{0}". |
| 571 | Periodo non valido. |
| 572 | Comando non valido. |
| 573 | Nomi di dominio non trovati. |
| 574 | Nomi TLD non trovati. |
| 575 | Il dominio non è in stato aggiornabile. Deve essere attivo per aggiornare i name server |
| 576 | Il dominio può essere rinnovato nell'ultimo mese prima della data di scadenza |
| 580 | Trasferimento non supportato per tld "{0}". |
| 581 | Child name server non trovato |
| 582 | Trasferimento avviato da altro rivenditore. |
| 583 | Trasferimento non inizializzato. Contatta il team di supporto. |
| 584 | Lo stato dell'oggetto vieta l'operazione |
| 590 | Il codice di autorizzazione è richiesto per questo trasferimento. |
| 591 | Il codice di autorizzazione non è valido. |
| 592 | Esiste un blocco di trasferimento sul dominio. |
| 593 | Il dominio non può essere trasferito, le informazioni di stato e il blocco di trasferimento devono rispettare i criteri richiesti. (Informazioni di stato: #ok) |
| 594 | L'indirizzo del name server di dominio non può essere risolto ({0}) |
| 595 | Le informazioni di contatto non possono essere lette (whois.registrar.tld) Assicurati che lo stato di protezione della privacy sia aperto |
| 596 | Le informazioni di contatto non possono essere verificate |
| 597 | Tld non trovato |
| 598 | Si è verificato un errore sconosciuto |
| 599 | Inoltro di dominio non trovato |

## Licenza

Apache License 2.0 — vedi [LICENSE](LICENSE).
