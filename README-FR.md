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

# rust-dna — client DomainNameAPI pour Rust

Client Rust pour l'API REST du bureau d'enregistrement de domaines DomainNameAPI. C'est un portage fidèle de la bibliothèque PHP de référence [php-dna](https://github.com/domainreseller/php-dna) : les mêmes méthodes, les mêmes requêtes sur le réseau et les mêmes valeurs de retour. Chaque portage est vérifié avec les 134 fixtures de conformité de php-dna.

## 📦 Téléchargement — utilisez toujours les Releases !

⬇️ **Obtenez la dernière version testée ici : https://github.com/domainreseller/rust-dna/releases/latest**

> ⚠️ N'utilisez **pas** le bouton vert **Code → Download ZIP** — il télécharge la branche de développement brute. Les paquets de release sont versionnés, testés et prêts pour la production.

## Guide d'installation et d'intégration

### Prérequis minimum

- Rust 1.83 ou plus récent (édition 2021)
- Aucun runtime asynchrone : l'API est bloquante
- Dépendances d'exécution : `ureq` (HTTP, TLS rustls), `rustls` et `serde_json`. OpenSSL n'est pas nécessaire.

### Installation

**A) Dépendance Git (figée sur un tag de release)**

```toml
[dependencies]
domainnameapi = { git = "https://github.com/domainreseller/rust-dna", tag = "v1.0.1" }
```

**B) Release téléchargée**

Décompressez l'archive de la release à côté de votre projet et indiquez-la à Cargo :

```toml
[dependencies]
domainnameapi = { path = "../rust-dna" }
```

La crate réexporte `serde_json::Value` et la macro `json!`, vous n'avez donc pas besoin d'ajouter `serde_json` vous-même (vous pouvez le faire si vous préférez).

## 🔑 Identifiants API — Reseller ID et API Key

La bibliothèque communique avec l'API REST de DomainNameAPI et a besoin d'exactement deux identifiants, passés au constructeur :

| Paramètre | Valeur |
|---|---|
| Premier | Reseller ID — UUID du type `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` |
| Second | API Key |

> 💡 Vous trouverez votre **Reseller ID** et votre **API Key** dans votre panneau DomainNameAPI, sous **Paramètres API**.
> ⚠️ Ce sont des **identifiants API** — vos identifiants de connexion au panneau ne fonctionneront **pas** ici.

```rust
use domainnameapi::DomainNameApi;

let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";
let api_key = "your-api-key";

// Live environment
let mut dna = DomainNameApi::new(reseller_id, api_key, false);

// Test (OTE) environment
let mut dna = DomainNameApi::new(reseller_id, api_key, true);
```

Les deux identifiants sont débarrassés des espaces de début et de fin. Le troisième argument accepte aussi des chaînes, car les panneaux enregistrent souvent les cases à cocher sous forme de texte : `"true"`, `"1"`, `"on"` et `"yes"` (sans tenir compte de la casse) sélectionnent l'environnement de test, toute autre valeur (y compris `"false"` et `"off"`) sélectionne la production.

## Démarrage rapide

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

Chaque exemple du dossier [examples/](examples) s'exécute avec `cargo run --example <nom>`, par exemple `cargo run --example get_details`.

## Organisation de l'API

- **Un seul type de client**, `DomainNameApi`. Les méthodes prennent `&mut self` (le client mémorise la dernière requête pour le diagnostic).
- **Les valeurs de retour sont des `serde_json::Value`** ayant exactement la structure renvoyée par php-dna — pas des modèles typés. Les noms de clés, les indicateurs sous forme de chaîne (`"true"` / `"false"`), les montants formatés (`"11.9900"`) et les enveloppes sont identiques :
  - succès : `{"result": "OK", "data": {...}}`
  - échec : `{"result": "ERROR", "error": {"Code": "...", "Message": "...", "Details": "..."}}`
  - quelques méthodes utilisent une autre enveloppe, reprise de php-dna : `get_current_balance` renvoie un objet `OperationResult`, `check_availability` renvoie une simple liste en cas de succès, `get_reseller_details` renvoie ses champs au premier niveau.
- **Les erreurs sont renvoyées, jamais levées.** Les erreurs de la passerelle, les erreurs HTTP et les pannes réseau reviennent toutes sous forme d'enveloppe d'erreur ; rien ne provoque de panic. L'enregistrement, le transfert et le renouvellement renvoient aussi une erreur lorsque la passerelle répond HTTP 200 avec `success: false`.
- **Les entrées structurées** (contacts, listes de serveurs de noms, filtres, attributs de TLD, contenus d'enregistrements) sont des `&Value` construites avec `json!`. Les maps de contact utilisent les clés de php-dna (`FirstName`, `EMail`, `AddressLine1`, `PhoneCountryCode`, ...) ; les formes plate et imbriquée (`Address.Line1`, `Phone.Phone.Number`) sont acceptées. Les listes de serveurs de noms peuvent être une liste ou une map (les valeurs sont utilisées).
- **Pas d'arguments par défaut en Rust.** Chaque méthode prend la liste complète des arguments de php-dna, dans le même ordre. Les valeurs par défaut de php-dna sont exportées comme constantes : `DEFAULT_TTL` (3600), `DEFAULT_REASON` ("Owner request"), `DEFAULT_FORWARD_TYPE` ("Standard"), `DEFAULT_CURRENCY` ("USD"), `DEFAULT_TLD_COUNT` (20), `DEFAULT_COMMAND` ("create"). `register_with_contact_info`, qui a quatre arguments optionnels, dispose aussi de `register_with_contact_info_opts`, qui prend une structure `RegisterOptions` dont le `Default` correspond aux valeurs par défaut de php-dna.

## Utilisation

Tous les extraits supposent `use domainnameapi::{json, DomainNameApi};` et un client `let mut dna = DomainNameApi::new(...)`.

#### Enregistrement de domaine

Remarque : des paramètres supplémentaires sont requis pour les domaines .tr. L'argument des attributs supplémentaires sert aux noms de domaine qui exigent des informations en plus, comme .tr.

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

Pour un domaine `.tr`, transmettez les attributs TRABIS. La bibliothèque n'envoie que ce que la passerelle accepte (`TRABISDOMAINCATEGORY`, `TRABISCITIZENID`, `TRABISTAXOFFICE`, `TRABISTAXNUMBER`), convertit `1`/`0` en `Owner`/`Corporate` et corrige l'ancienne orthographe `TRABISCITIZIENID` :

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
<summary>Exemple de sortie pour l'enregistrement de domaine</summary>

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

#### Renouvellement de domaine

```rust
let result = dna.renew("example.com", 1);
```

<details>
<summary>Exemple de sortie pour le renouvellement de domaine</summary>

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

#### Transfert de domaine

Les transferts n'envoient jamais de coordonnées de contact : le bureau d'enregistrement reprend les contacts de l'ancien bureau d'enregistrement, chaque requête de transfert contient donc `contacts: []`.

```rust
let result = dna.transfer("example.com", "Xy9#mK2$", 3);
```

<details>
<summary>Exemple de sortie pour le transfert de domaine</summary>

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

#### Gestion des transferts

```rust
// Can the domain be transferred with this auth code?
let check = dna.check_transfer("example.com", "Xy9#mK2$");

// Cancel an incoming transfer, approve or reject an outgoing one
let result = dna.cancel_transfer("example.com");
let result = dna.approve_transfer("example.com");
let result = dna.reject_transfer("example.com");
```

<details>
<summary>Exemple de sortie pour la gestion des transferts</summary>

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

`check_transfer` renvoie `"result": "ERROR"` avec une `error` (`TRANSFER_NOT_AVAILABLE`) lorsque le registre indique que le domaine n'est pas transférable ; `data` est rempli dans les deux cas. `cancel_transfer`, `approve_transfer` et `reject_transfer` renvoient :

```json
{"result": "OK", "data": {"DomainName": "example.com", "Status": "Cancelled"}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Liste des domaines

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

`PageNumber` commence à 0. Les paramètres propres à la passerelle `MaxResultCount`, `SkipCount` et `Sorting`, ainsi que des filtres comme `DomainName`, peuvent aussi être transmis.

<details>
<summary>Exemple de sortie pour la liste des domaines</summary>

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

#### Liste des TLD

```rust
let result = dna.get_tld_list(100);
```

<details>
<summary>Exemple de sortie pour la liste des TLD</summary>

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

#### Vérification de disponibilité

```rust
let result = dna.check_availability(["hello", "world123x0"], ["com", "net"], 1, "create");
```

<details>
<summary>Exemple de sortie pour la vérification de disponibilité</summary>

En cas de succès, le résultat est une simple liste (sans clé `result`) ; en cas d'échec, c'est l'enveloppe d'erreur.

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

#### Détails du domaine

```rust
let result = dna.get_details("example.com");

// Re-read the domain from the registry (same result shape):
let result = dna.sync_from_registry("example.com");
```

<details>
<summary>Exemple de sortie pour les détails du domaine</summary>

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

#### Modification des serveurs de noms

```rust
let result = dna.modify_name_server("example.com", &json!(["ns1.example.com", "ns2.example.com"]));
```

<details>
<summary>Exemple de sortie pour la modification des serveurs de noms</summary>

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

#### Activer le verrouillage du domaine

```rust
let result = dna.enable_theft_protection_lock("example.com");
```

<details>
<summary>Exemple de sortie pour l'activation du verrouillage</summary>

```json
{"result": "OK", "data": {"LockStatus": true}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Désactiver le verrouillage du domaine

```rust
let result = dna.disable_theft_protection_lock("example.com");
```

<details>
<summary>Exemple de sortie pour la désactivation du verrouillage</summary>

```json
{"result": "OK", "data": {"LockStatus": false}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Ajouter un serveur de noms enfant

```rust
let result = dna.add_child_name_server("example.com", "ns1.example.com", "1.2.3.4");
```

<details>
<summary>Exemple de sortie pour l'ajout d'un serveur de noms enfant</summary>

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

#### Supprimer un serveur de noms enfant

```rust
let result = dna.delete_child_name_server("example.com", "ns1.example.com");
```

<details>
<summary>Exemple de sortie pour la suppression d'un serveur de noms enfant</summary>

```json
{"result": "OK", "data": {"NameServer": "ns1.example.com"}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Mettre à jour un serveur de noms enfant

```rust
let result = dna.modify_child_name_server("example.com", "ns1.example.com", "1.2.3.4");
```

<details>
<summary>Exemple de sortie pour la mise à jour d'un serveur de noms enfant</summary>

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

#### Modifier la confidentialité du domaine

`status` est un `bool` ou une valeur telle que les panneaux la stockent : `"1"`, `"on"`, `"true"`, `"yes"` et `1` signifient activé, toute autre valeur désactivé. L'argument de motif (reason) n'est conservé que pour la compatibilité et n'est pas envoyé, car la passerelle n'a pas ce champ.

```rust
let result = dna.modify_privacy_protection_status("example.com", true, "Owner request");
```

<details>
<summary>Exemple de sortie pour la modification de la confidentialité</summary>

```json
{"data": {"PrivacyProtectionStatus": true}, "result": "OK"}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Obtenir les contacts du domaine

```rust
let result = dna.get_contacts("example.com");
```

<details>
<summary>Exemple de sortie pour l'obtention des contacts</summary>

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

#### Enregistrer les contacts du domaine

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
<summary>Exemple de sortie pour l'enregistrement des contacts</summary>

```json
{"result": "OK", "data": {"contacts": []}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Enregistrements de zone DNS

La passerelle stocke les enregistrements sous forme d'**ensembles** (un nom + un type contenant une ou plusieurs valeurs). Écrire un ensemble remplace toutes ses valeurs, et supprimer n'importe quelle valeur supprime l'ensemble entier. Les méthodes `*_value` effectuent pour vous la lecture-fusion-écriture, afin qu'un panneau puisse traiter les enregistrements un par un. Les noms peuvent être relatifs (`www`), `@` / vides pour l'apex, ou pleinement qualifiés ; ils sont réduits au libellé relatif attendu par la passerelle. Types pris en charge : A, AAAA, ALIAS, CAA, CNAME, DS, MX, NAPTR, PTR, SRV, TLSA, TXT. Les modifications sont appliquées immédiatement lorsque `apply` vaut `true` (valeur par défaut de php-dna) ; sinon appelez `apply_resource_records`. Les appels de zone sont espacés d'au moins une seconde car la passerelle limite les rafales. Si l'application automatique échoue, la méthode renvoie l'erreur d'application (la modification reste en attente). Les résultats indiquent le type en majuscules et le TTL réellement envoyé (en dessous de 1 il devient 3600, au-dessus de 86400 il devient 86400).

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
<summary>Exemple de sortie pour les enregistrements de zone DNS</summary>

`get_resource_records` :

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

`set_resource_record_set` / `add_resource_record_value` :

```json
{"result": "OK", "data": {"Name": "", "Type": "MX", "TTL": 3600, "Contents": ["10 mx1.example.com.", "20 mx2.example.com.", "30 mx3.example.com"]}}
```

Une valeur déjà présente (ou absente, pour une suppression) ne change rien :

```json
{"result": "OK", "data": {"Name": "", "Type": "MX", "Contents": ["10 mx1.example.com."], "Changed": false}}
```

Un type non pris en charge est refusé avant toute requête :

```json
{"result": "ERROR", "error": {"Code": "ADD_ZONE", "Message": "Record type \"NS\" is not supported by the DNS zone API. Supported types: A, AAAA, ALIAS, CAA, CNAME, DS, MX, NAPTR, PTR, SRV, TLSA, TXT.", "Details": "..."}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Redirection d'URL

```rust
let current = dna.get_forwarding("example.com");
let result = dna.set_forwarding("example.com", "https://www.example.org", "Standard"); // or "Frame" (masked)
let result = dna.delete_forwarding("example.com");
```

<details>
<summary>Exemple de sortie pour la redirection d'URL</summary>

```json
{"result": "OK", "data": {"Enabled": true, "RedirectAddress": "https://www.example.org/landing", "ForwardType": "Frame"}}
```

Si aucune redirection n'est configurée (la passerelle répond `Redirect information not found`), `get_forwarding` renvoie `{"result": "OK", "data": {"Enabled": false, "RedirectAddress": "", "ForwardType": ""}}` ; tout autre échec, comme un domaine inexistant, est renvoyé comme une erreur. `set_forwarding` renvoie `{"result": "OK", "data": {"RedirectAddress": "...", "ForwardType": "Standard"}}`. `delete_forwarding` renvoie `{"result": "OK", "data": {"DomainName": "example.com", "Removed": true}}`, ou `"Removed": false` si aucune redirection n'était configurée.

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Obtenir les détails du revendeur

```rust
let result = dna.get_reseller_details();
let balance = dna.get_current_balance("USD"); // "USD" or "TRY"
```

<details>
<summary>Exemple de sortie pour les détails du revendeur</summary>

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

`get_current_balance` :

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

Le compte indique les soldes en USD et en TRY ; toute autre devise (EUR, GBP, ...) échoue avec `ErrorCode` `"BALANCE"`. En cas d'échec, `OperationResult` vaut `"FAILED"`, `ErrorCode` contient le code (par ex. `"API_429"`) et `Balance` vaut `"0.00"`.

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### Utilitaires (sans réseau)

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
<summary>Exemple de sortie pour validate_contact</summary>

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

## Gestion des erreurs

Chaque méthode réseau renvoie une valeur ; vérifiez `result` :

```rust
let out = dna.get_details("missing.com");
if out["result"] == "ERROR" {
    println!("{}", out["error"]["Code"]);        // "API_404"
    println!("{}", out["error"]["GatewayCode"]); // "Dna.DomainService:Domain:10007"
    println!("{}", out["error"]["Message"]);     // "The domain name you requested could not be found. ..."
    println!("{}", out["error"]["Details"]);
}
```

- Les erreurs HTTP utilisent le code `API_<statut>` (`API_400`, `API_404`, `API_429`, `API_500`, ...). Un corps d'erreur vide devient un message lisible tel que `HTTP 429 - Too Many Requests`. Lorsque la passerelle a envoyé son propre code d'erreur, l'enveloppe le contient aussi dans `GatewayCode` (par ex. `"Dna.DomainService:Domain:10007"`).
- Une réponse 301/302 signifie des identifiants erronés : `Invalid API credentials`.
- Les erreurs de validation de la passerelle sont regroupées dans `Details` en nommant le champ, par ex. `Contacts Phone - The field Phone must be a string with a maximum length of 16.`
- Les pannes réseau sont des erreurs, jamais un succès vide. Un dépassement de délai donne `Could not reach the DomainNameAPI service: the connection timed out. (...)` ; les autres pannes donnent `Curl error during request: ...`. Leur code est `CURL_<numéro d'erreur curl>` (`CURL_28` pour un délai dépassé, `CURL_6` pour le DNS, `CURL_7` pour une connexion refusée) ; `API_<n>` est réservé aux statuts HTTP.
- La validation locale (type d'enregistrement non pris en charge, adresse de redirection manquante, aucun domaine à vérifier) échoue avant toute requête avec un code explicite (`ADD_ZONE`, `SET_FORWARD`, `AVAILABILITY`, ...).

### Diagnostic

```rust
dna.get_request_data();        // {"url", "method", "payload"} of the last request
dna.get_response_data();       // decoded last response, or {"raw_response": "<body>"}
dna.get_response_headers();    // {"url", "content_type", "http_code", "total_time", "headers"}
dna.get_parsed_response_data();// parsed body on success, {"Code", "Message", "Details"} on failure
dna.get_last_function();       // e.g. "request:GET domains/info"
dna.get_service_url();
dna.set_service_url("https://ote.domainresellerapi.com/api/v1");
```

### Transport personnalisé

La couche HTTP est le trait `Transport`. Par défaut, c'est `UreqTransport` (`ureq` bloquant avec rustls, délais de 30 s au total et de 10 s pour la connexion, redirections non suivies). Remplacez-le pour des proxys, des clients personnalisés ou des tests :

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

## Mode test

Passez `true` (ou `"true"`, `"1"`, `"on"`, `"yes"`) comme troisième argument du constructeur pour utiliser l'environnement de test OTE à `https://ote.domainresellerapi.com/api/v1`. Les appels en production vont vers `https://api.domainresellerapi.com/api/v1`.

## Tests

La suite de tests ne nécessite ni réseau ni identifiants :

```bash
cargo test
```

Elle contient des tests unitaires pour la couche HTTP, la correspondance des erreurs, le masquage de la télémétrie et les utilitaires, ainsi qu'un test de conformité qui rejoue les 134 fixtures de php-dna depuis `tests/conformance/` et vérifie que chaque requête envoyée et chaque valeur de retour correspondent à php-dna.

## Rapports d'erreurs (télémétrie)

Lorsqu'un appel API échoue, la bibliothèque peut envoyer un rapport d'erreur au système de suivi d'erreurs de DomainNameAPI (Sentry). Une petite part aléatoire des appels réussis (4 %, plus chaque appel de plus d'une seconde) est envoyée comme mesure de performance. Cela aide DomainNameAPI à trouver et corriger les problèmes. Les rapports d'erreurs sont **activés par défaut**.

**Ce qui est envoyé :**

- le code et le message d'erreur
- le chemin du point de terminaison appelé et la méthode HTTP
- la durée de l'appel
- le nom de l'application (`DomainNameAPI-Rust`) et la version de la bibliothèque
- l'ID revendeur et le nom de domaine
- l'adresse IP publique du serveur (obtenue en arrière-plan via `api.ipify.org`)
- le nom d'utilisateur du système d'exploitation
- la requête et la réponse de l'API, y compris les coordonnées des contacts du domaine ; seuls les identifiants sont masqués — les codes EPP/auth, clés API, mots de passe, secrets et jetons sont remplacés par `***`

**Ce qui n'est jamais envoyé :** codes EPP/auth, clés API, mots de passe ou jetons sous forme lisible.

Les rapports sont envoyés depuis un thread en arrière-plan avec des délais courts ; ils ne bloquent jamais votre appel et ne le font jamais échouer. Les erreurs attendues comme « domaine introuvable » ou « solde insuffisant » ne sont pas signalées.

**Pourquoi et pour combien de temps :** les données servent uniquement à mesurer l'utilisation et à diagnostiquer tôt les problèmes ; elles sont conservées 30 jours sur le serveur Sentry propre à DomainNameAPI, puis supprimées automatiquement.

**Comment le désactiver ou le réduire :**

```rust
let mut dna = DomainNameApi::new(reseller_id, api_key, false);
dna.set_error_reporting_enabled(false); // nothing is sent

// or keep it on but send only the error code and message, endpoint,
// timing, versions and reseller ID (no request/response data, domain, IP or user name)
dna.set_telemetry_minimal(true);
```

> 🔒 **Sécurité :** la vérification des certificats TLS est activée. Sur un hôte dont le bundle CA est obsolète, elle peut être désactivée avec `dna.set_ssl_verification(false);` — ce n'est pas recommandé ; mettre à jour le bundle CA est la meilleure solution.

## Codes de Retour et d'Erreur avec Descriptions

| Code | Description | Détail |
|------|-------------|--------|
| 1000 | Commande complétée avec succès | Opération réussie. |
| 1001 | Commande complétée avec succès ; action en attente | Opération réussie. Cependant, l'opération est actuellement en file d'attente pour completion. |
| 2003 | Paramètre requis manquant | Erreur de paramètre manquant. Par exemple : aucune entrée de téléphone dans les informations de contact. |
| 2105 | L'objet n'est pas éligible pour le renouvellement | Le statut du domaine n'est pas adapté pour le renouvellement, verrouillé pour les opérations de mise à jour. Le statut ne doit pas être "clientupdateprohibited". Peut être dû à d'autres conditions de statut. |
| 2200 | Erreur d'authentification | Erreur d'autorisation, code de sécurité incorrect ou domaine enregistré avec un autre registraire. |
| 2302 | L'objet existe | Le nom de domaine ou les informations de serveur de noms existent déjà dans la base de données. Ne peut pas être enregistré. |
| 2303 | L'objet n'existe pas | Le nom de domaine ou les informations de serveur de noms n'existent pas dans la base de données. Nouvel enregistrement requis. |
| 2304 | Le statut de l'objet interdit l'opération | Le statut du domaine n'est pas adapté pour les mises à jour, verrouillé pour les opérations de mise à jour. Le statut ne doit pas être "clientupdateprohibited". Peut être dû à d'autres conditions de statut. |

| CODE | DÉTAIL |
|------|--------|
| 101 | Erreur système détaillée pour ({0})! |
| 102 | Erreurs multiples détaillées pour ({0})! |
| 103 | Erreurs inconnues détaillées pour ({0})! |
| 200 | Commande API introuvable ({0})! |
| 210 | Service API introuvable ({0})! |
| 211 | Fournisseur de service API non défini! |
| 300 | Revendeur introuvable! |
| 301 | Votre adresse IP actuelle n'est pas autorisée à accéder. Veuillez vous assurer que vous vous connectez depuis une adresse IP autorisée et réessayez |
| 310 | TLD non supporté! |
| 320 | API introuvable! |
| 321 | Devise non supportée! |
| 330 | Paramètre(s) requis non défini(s) ({0}). |
| – | Assurez-vous d'envoyer tous les objets de contact complets |
| 340 | Définition de prix introuvable ({0}[{1}]-{2}{3}). |
| 350 | Solde de revendeur insuffisant. (Id Revendeur : {0} - Solde Actuel : {1} {2}). |
| 350 | La devise comptable ne correspond pas ou le solde est insuffisant. |
| 360 | Requête API invalide pour le champ ({0}). |
| 360 | Quota API dépassé! |
| 361 | Erreur de limitation! |
| 362 | Le domaine premium n'est pas disponible pour l'enregistrement pour le moment. |
| 363 | Opération annulée car le domaine est en période de renouvellement automatique |
| 364 | Ce domaine est actuellement indisponible pour une transaction en raison d'un problème avec le registre |

| CODE | DÉTAIL |
|------|--------|
| 400 | Contact invalide ({0}). |
| 401 | Les informations de contact ne peuvent pas être synchronisées. |
| 402 | Aucun accès aux informations de contact depuis le registre. |
| 403 | LE SYSTÈME A DES INFORMATIONS MANQUANTES. ENTREZ VOS INFORMATIONS PAR DÉFAUT OU CONTACTEZ L'ÉQUIPE DE SUPPORT. |
| 404 | LE CONTACT DE DOMAINE NE PEUT PAS ÊTRE MIS À JOUR. PERMISSION DU REGISTRE. |
| 410 | Contact introuvable. |
| 410 | Contact introuvable ({0}). |
| 420 | Commande Api invalide pour le contact {0}. |
| 430 | API de contact introuvable. |
| 440 | Le contact n'est pas synchronisé. |
| 450 | Trop de contacts de domaine.. ! |
| 451 | Échec de la poursuite de la mise à jour du contact. |

| CODE | DÉTAIL |
|------|--------|
| 500 | ID de domaine invalide. |
| 500 | ID de domaine invalide ({0}). |
| 501 | Le domaine n'a pas pu être synchronisé({0}) |
| 502 | Transfert interne échoué |
| 503 | L'enregistrement de domaine n'est pas disponible. |
| 504 | Les informations de domaine ne correspondent pas .. Avant :{0} |
| 505 | Entrez l'adresse IP. |
| 506 | Le transfert de domaine a été initié mais les informations de contact n'ont pas pu être lues .. |
| 507 | Erreur d'autorisation. |
| 510 | Domaine introuvable. |
| 511 | Les domaines expirés ne peuvent pas être trouvés. |
| 512 | Le domaine n'est pas renouvelable. |
| 513 | Le domaine n'est pas dans un état modifiable. Il doit être actif pour être mis à jour |
| 514 | Période de rachat attendue. |
| 520 | Commande Api invalide pour le domaine "{0}". |
| 530 | Période de domaine invalide. La période doit être de {0} à {1} ans. La période demandée est de {2} ans. |
| 540 | Le domaine ne peut pas être étendu au-delà de {0} ans à partir de la date actuelle. |
| 550 | Nom de domaine invalide. Le domaine doit avoir une longueur de {0} à {1} caractères. Le nom de domaine demandé fait {2} caractères. |
| 560 | Nombre de serveurs de noms invalide. Le domaine doit avoir {0} à {1} serveurs de noms. Le nombre de serveurs de noms demandé est {2}. |
| 561 | Aucune information de serveur de noms trouvée dans la requête entrante |
| 570 | Idn non supporté pour le tld "{0}". |
| 571 | Période invalide. |
| 572 | Commande invalide. |
| 573 | Noms de domaine introuvables. |
| 574 | Noms TLD introuvables. |
| 575 | Le domaine n'est pas dans un état modifiable. Il doit être actif pour que les serveurs de noms soient mis à jour |
| 576 | Le domaine peut être renouvelé dans le dernier mois avant la date d'expiration |
| 580 | Transfert non supporté pour le tld "{0}". |
| 581 | Serveur de noms enfant introuvable |
| 582 | Transfert initié par un autre revendeur. |
| 583 | Transfert non initialisé. Veuillez contacter l'équipe de support. |
| 584 | Le statut de l'objet interdit l'opération |
| 590 | Le code d'autorisation est requis pour ce transfert. |
| 591 | Le code d'autorisation n'est pas valide. |
| 592 | Un verrou de transfert existe sur le domaine. |
| 593 | Le domaine ne peut pas être transféré, les informations de statut et le verrou de transfert doivent respecter les critères requis. (Informations de statut: #ok) |
| 594 | L'adresse du serveur de noms de domaine n'a pas pu être résolue ({0}) |
| 595 | Les informations de contact ne peuvent pas être lues (whois.registrar.tld) Veuillez vous assurer que le statut de protection de la vie privée est ouvert |
| 596 | Les informations de contact n'ont pas pu être vérifiées |
| 597 | Tld introuvable |
| 598 | Erreur inconnue survenue |
| 599 | Redirection de domaine introuvable |

## Licence

Apache License 2.0 — voir [LICENSE](LICENSE).
