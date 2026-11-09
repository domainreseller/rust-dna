//! Register a .tr domain with TRABIS attributes.
//!
//! Run with `cargo run --example register_with_contact_info_tr`.

use domainnameapi::{json, DomainNameApi, RegisterOptions};

fn main() {
    // Credentials: your Reseller ID (UUID) + API Key from Panel -> API Settings.
    // Panel login credentials will NOT work.
    let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"; // Reseller ID (UUID)
    let api_key = "your-api-key"; // API Key

    let mut dna = DomainNameApi::new(reseller_id, api_key, false);

    let contact = json!({
        "FirstName": "Ahmet",
        "LastName": "Yilmaz",
        "Company": "",
        "EMail": "ahmet.yilmaz@example.com",
        "AddressLine1": "Ornek Sokak No: 123",
        "AddressLine2": "Daire 4",
        "AddressLine3": "",
        "City": "Istanbul",
        "Country": "TR",
        "Fax": "2121234567",
        "FaxCountryCode": "90",
        "Phone": "5551234567",
        "PhoneCountryCode": 90,
        "Type": "Contact",
        "ZipCode": "34000",
        "State": "Istanbul"
    });
    let contacts = json!({
        "Administrative": contact,
        "Billing": contact,
        "Technical": contact,
        "Registrant": contact
    });
    // Individual owner
    let individual = json!({
        "TRABISDOMAINCATEGORY": 1,
        "TRABISCITIZIENID": "11111111111",
        "TRABISNAMESURNAME": "Ahmet Yilmaz",
        "TRABISCOUNTRYID": "215",
        "TRABISCITYID": "34",
        "TRABISCOUNTRYNAME": "Türkiye",
        "TRABISCITYNAME": "Istanbul"
    });
    // Company owner
    let _commercial = json!({
        "TRABISDOMAINCATEGORY": 0,
        "TRABISORGANIZATION": "Example Ltd. Sti.",
        "TRABISTAXOFFICE": "Istanbul VD",
        "TRABISTAXNUMBER": "1234567890",
        "TRABISCOUNTRYID": "215",
        "TRABISCITYID": "34",
        "TRABISCOUNTRYNAME": "Türkiye",
        "TRABISCITYNAME": "Istanbul"
    });

    // The library keeps only the attributes the gateway accepts and maps
    // legacy values (1 -> Owner, 0 -> Corporate, TRABISCITIZIENID typo).
    let result = dna.register_with_contact_info_opts(
        "example.com.tr",
        1,
        &contacts,
        &RegisterOptions {
            name_servers: json!(["ns1.example.com", "ns2.example.com"]),
            additional_attributes: individual,
            ..RegisterOptions::default()
        },
    );

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
