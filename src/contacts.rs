//! Contact and TLD-attribute helpers (php-dna `parseContact`,
//! `parseContactInfo`, `validateContact`, `normalizePhone`,
//! `normalizeTrAttributes`, `stringifyAttributes`).

use serde_json::{json, Map, Value};

use crate::php::{self, get, get_path, obj};

/// Default values `validate_contact` fills in.
const DEFAULTS: [(&str, &str); 8] = [
    ("FirstName", "Isimyok"),
    ("LastName", "Isimyok"),
    ("AddressLine1", "Addres yok"),
    ("City", "ISTANBUL"),
    ("Country", "TR"),
    ("ZipCode", "34000"),
    ("Phone", "5555555555"),
    ("PhoneCountryCode", "90"),
];

fn is_blank(v: Option<&Value>) -> bool {
    php::trim(&v.map(php::strval).unwrap_or_default()).is_empty()
}

/// php-dna `validateContact()`.
pub(crate) fn validate_contact(input: &Value) -> Value {
    let mut c: Map<String, Value> = match input {
        Value::Object(map) => map.clone(),
        Value::Array(list) => list
            .iter()
            .enumerate()
            .map(|(i, v)| (i.to_string(), v.clone()))
            .collect(),
        _ => Map::new(),
    };
    for (key, value) in DEFAULTS {
        if c.get(key).map(Value::is_null).unwrap_or(true) {
            c.insert(key.to_string(), json!(value));
        }
    }
    // No State default: a missing state falls back to the city below.
    if c.get("State").map(Value::is_null).unwrap_or(true) {
        c.insert("State".into(), json!(""));
    }

    let first = sanitize_person_name(&php::strval(&c["FirstName"]));
    let last = sanitize_person_name(&php::strval(&c["LastName"]));
    c.insert("FirstName".into(), json!(first));
    c.insert("LastName".into(), json!(last));

    if is_blank(c.get("FirstName")) {
        c.insert("FirstName".into(), json!("Isimyok"));
    }
    if is_blank(c.get("LastName")) {
        let first = c["FirstName"].clone();
        c.insert("LastName".into(), first);
    }
    if is_blank(c.get("AddressLine1")) {
        c.insert("AddressLine1".into(), json!("Addres yok"));
    }
    if is_blank(c.get("City")) {
        c.insert("City".into(), json!("ISTANBUL"));
    }
    if is_blank(c.get("State")) {
        let city = c["City"].clone();
        c.insert("State".into(), city);
    }
    let country = normalize_country_code(&php::strval(&c["Country"]));
    c.insert("Country".into(), json!(country));
    if is_blank(c.get("Country")) {
        c.insert("Country".into(), json!("TR"));
    }
    if is_blank(c.get("ZipCode")) {
        c.insert("ZipCode".into(), json!("34000"));
    }

    let (phone, phone_cc) = normalize_phone(
        &php::strval(&c["Phone"]),
        &php::strval(&c["PhoneCountryCode"]),
    );
    c.insert("Phone".into(), json!(phone));
    c.insert("PhoneCountryCode".into(), json!(phone_cc));
    if is_blank(c.get("PhoneCountryCode")) {
        c.insert("PhoneCountryCode".into(), json!("90"));
    }
    if is_blank(c.get("Phone")) {
        c.insert("Phone".into(), json!("5555555555"));
    }

    let fax_in = c
        .get("Fax")
        .filter(|v| !v.is_null())
        .map(php::strval)
        .unwrap_or_default();
    let fax_cc_in = c
        .get("FaxCountryCode")
        .filter(|v| !v.is_null())
        .map(php::strval)
        .unwrap_or_default();
    let (fax, fax_cc) = normalize_phone(&fax_in, &fax_cc_in);
    let fax_cc = if fax.is_empty() {
        String::new()
    } else {
        fax_cc
    };
    c.insert("Fax".into(), json!(fax));
    c.insert("FaxCountryCode".into(), json!(fax_cc));

    Value::Object(c)
}

/// Keep letters (any alphabet), whitespace, apostrophes and hyphens;
/// collapse whitespace; trim.
pub(crate) fn sanitize_person_name(name: &str) -> String {
    let kept: String = name
        .chars()
        .filter(|c| c.is_alphabetic() || c.is_whitespace() || *c == '\'' || *c == '-')
        .collect();
    let mut out = String::with_capacity(kept.len());
    let mut in_space = false;
    for ch in kept.chars() {
        if ch.is_whitespace() {
            if !in_space {
                out.push(' ');
            }
            in_space = true;
        } else {
            out.push(ch);
            in_space = false;
        }
    }
    php::trim(&out).to_string()
}

/// Country name or code → 2-letter ISO code where known.
pub(crate) fn normalize_country_code(country: &str) -> String {
    let country = php::trim(country);
    if country.is_empty() {
        return String::new();
    }
    if country.len() == 2 {
        return php::upper(country);
    }
    let code = match php::lower(country).as_str() {
        "turkey" | "türkiye" | "turkiye" => "TR",
        "united states" | "united states of america" | "usa" => "US",
        "united kingdom" | "great britain" | "england" => "GB",
        "germany" | "deutschland" => "DE",
        "france" => "FR",
        "spain" => "ES",
        "italy" => "IT",
        "netherlands" => "NL",
        "india" => "IN",
        "china" => "CN",
        "russia" => "RU",
        "canada" => "CA",
        "australia" => "AU",
        "brazil" => "BR",
        _ => return country.to_string(),
    };
    code.to_string()
}

/// Split a free-form phone number into `(subscriber number, country code)`.
pub(crate) fn normalize_phone(number: &str, country_code: &str) -> (String, String) {
    let lead = number.trim_start_matches(|c: char| c.is_ascii_whitespace() || c == '\x0b');
    let has_intl_prefix = lead.starts_with('+') || lead.starts_with("00");
    let mut digits: String = number.chars().filter(char::is_ascii_digit).collect();
    let mut cc: String = country_code.chars().filter(char::is_ascii_digit).collect();

    if digits.is_empty() {
        return (String::new(), cc);
    }
    if has_intl_prefix {
        if digits.starts_with("00") {
            digits = digits[2..].to_string();
        }
        if !cc.is_empty() && digits.starts_with(&cc) && digits.len() > cc.len() {
            digits = digits[cc.len()..].to_string();
        } else {
            // The number carries its own country code ("+1 202...") and it
            // differs from the given one (often a default 90): the number wins.
            let own = calling_code_prefix(&digits);
            if !own.is_empty() && digits.len() > own.len() {
                digits = digits[own.len()..].to_string();
                cc = own;
            }
        }
    } else if !cc.is_empty() && digits.starts_with(&cc) && digits.len() > 10 {
        digits = digits[cc.len()..].to_string();
    }
    if cc.is_empty() {
        if digits.len() == 12 && digits.starts_with("90") {
            digits = digits[2..].to_string();
        }
        cc = "90".to_string();
    }
    if digits.len() > 1 && digits.starts_with('0') {
        digits = digits.trim_start_matches('0').to_string();
    }
    (digits, cc)
}

/// Country calling code at the start of an international number (digits
/// only). E.164 codes are prefix-free: zones 1 and 7 use one digit, the
/// listed two-digit codes use two, everything else uses three.
pub(crate) fn calling_code_prefix(digits: &str) -> String {
    const TWO_DIGIT: [&str; 44] = [
        "20", "27", "30", "31", "32", "33", "34", "36", "39", "40", "41", "43", "44", "45", "46",
        "47", "48", "49", "51", "52", "53", "54", "55", "56", "57", "58", "60", "61", "62", "63",
        "64", "65", "66", "81", "82", "84", "86", "90", "91", "92", "93", "94", "95", "98",
    ];
    if digits.is_empty() || digits.starts_with('0') {
        return String::new();
    }
    if digits.starts_with('1') || digits.starts_with('7') {
        return digits[..1].to_string();
    }
    if digits.len() >= 2 && TWO_DIGIT.contains(&&digits[..2]) {
        return digits[..2].to_string();
    }
    if digits.len() >= 3 {
        digits[..3].to_string()
    } else {
        String::new()
    }
}

/// php-dna `parseContact()`: module contact (flat or nested) → gateway
/// `ContactLiteDto`.
pub(crate) fn parse_contact(contact: &Value, kind: &str) -> Value {
    let api_type = match kind {
        "Administrative" => "Admin",
        "Technical" => "Tech",
        "Billing" => "Billing",
        "Registrant" => "Registrant",
        other => other,
    };
    let pick = |flat: &str, nested: &[&str]| -> Value {
        get(contact, flat)
            .or_else(|| get_path(contact, nested))
            .cloned()
            .unwrap_or(json!(""))
    };

    let mut address = pick("AddressLine1", &["Address", "Line1"]);
    let line2 = pick("AddressLine2", &["Address", "Line2"]);
    if php::truthy(&line2) {
        address = json!(format!("{} {}", php::strval(&address), php::strval(&line2)));
    }

    let mut phone = pick("Phone", &["Phone", "Phone", "Number"]);
    let mut phone_cc = pick("PhoneCountryCode", &["Phone", "Phone", "CountryCode"]);
    let mut fax = pick("Fax", &["Phone", "Fax", "Number"]);
    let mut fax_cc = pick("FaxCountryCode", &["Phone", "Fax", "CountryCode"]);
    if php::is_array(&phone) {
        phone_cc = get_path(&phone, &["Phone", "CountryCode"])
            .cloned()
            .unwrap_or(json!(""));
        phone = get_path(&phone, &["Phone", "Number"])
            .cloned()
            .unwrap_or(json!(""));
    }
    if php::is_array(&fax) {
        fax_cc = get_path(&fax, &["Fax", "CountryCode"])
            .cloned()
            .unwrap_or(json!(""));
        fax = get_path(&fax, &["Fax", "Number"])
            .cloned()
            .unwrap_or(json!(""));
    }

    let validated = validate_contact(&obj([
        (
            "FirstName",
            get(contact, "FirstName").cloned().unwrap_or(json!("")),
        ),
        (
            "LastName",
            get(contact, "LastName").cloned().unwrap_or(json!("")),
        ),
        ("AddressLine1", address),
        ("City", pick("City", &["Address", "City"])),
        ("State", pick("State", &["Address", "State"])),
        ("Country", pick("Country", &["Address", "Country"])),
        ("ZipCode", pick("ZipCode", &["Address", "ZipCode"])),
        ("Phone", phone),
        ("PhoneCountryCode", phone_cc),
        ("Fax", fax),
        ("FaxCountryCode", fax_cc),
    ]));

    obj([
        ("contactType", json!(api_type)),
        ("firstName", validated["FirstName"].clone()),
        ("lastName", validated["LastName"].clone()),
        (
            "companyName",
            get(contact, "Company").cloned().unwrap_or(json!("")),
        ),
        ("eMail", get(contact, "EMail").cloned().unwrap_or(json!(""))),
        ("address", validated["AddressLine1"].clone()),
        ("city", validated["City"].clone()),
        ("state", validated["State"].clone()),
        ("country", validated["Country"].clone()),
        ("postalCode", validated["ZipCode"].clone()),
        ("phoneCountryCode", validated["PhoneCountryCode"].clone()),
        ("phone", validated["Phone"].clone()),
        ("faxCountryCode", validated["FaxCountryCode"].clone()),
        ("fax", validated["Fax"].clone()),
        (
            "isHidden",
            json!(php::truthy(
                get(contact, "IsHidden").unwrap_or(&json!(false))
            )),
        ),
    ])
}

/// php-dna `parseContactInfo()`: gateway contact → module contact.
pub(crate) fn parse_contact_info(data: &Value) -> Value {
    if !php::truthy(data) {
        return json!([]);
    }
    let first = |keys: &[&str], default: &str| -> Value {
        keys.iter()
            .find_map(|k| get(data, k))
            .cloned()
            .unwrap_or(json!(default))
    };
    obj([
        ("ID", first(&["handle", "id"], "")),
        ("Status", first(&["status"], "Active")),
        ("AuthCode", first(&["authCode"], "")),
        ("FirstName", first(&["firstName"], "")),
        ("LastName", first(&["lastName"], "")),
        (
            "Company",
            first(&["companyName", "organizationName", "company"], ""),
        ),
        ("EMail", first(&["eMail", "emailAddress", "email"], "")),
        ("Type", first(&["type"], "")),
        (
            "Address",
            obj([
                ("Line1", first(&["address", "addressLine1"], "")),
                ("Line2", first(&["addressLine2"], "")),
                ("Line3", first(&["addressLine3"], "")),
                ("State", first(&["state", "stateOrProvince"], "")),
                ("City", first(&["city"], "")),
                ("Country", first(&["country", "countryCode"], "")),
                ("ZipCode", first(&["postalCode", "zipCode"], "")),
            ]),
        ),
        (
            "Phone",
            obj([
                (
                    "Phone",
                    obj([
                        ("Number", first(&["phone", "phoneNumber"], "")),
                        ("CountryCode", first(&["phoneCountryCode"], "")),
                    ]),
                ),
                (
                    "Fax",
                    obj([
                        ("Number", first(&["fax", "faxNumber"], "")),
                        ("CountryCode", first(&["faxCountryCode"], "")),
                    ]),
                ),
            ]),
        ),
        (
            "Additional",
            get(data, "additionalAttributes")
                .cloned()
                .unwrap_or(json!([])),
        ),
    ])
}

/// php-dna `normalizeTrAttributes()`: keep the four keys the gateway accepts,
/// fix the `TRABISCITIZIENID` typo, map the domain category to
/// `Owner` / `Corporate`.
pub(crate) fn normalize_tr_attributes(attrs: &Value) -> Value {
    if !php::is_array(attrs) {
        return json!([]);
    }
    if !php::truthy(attrs) {
        return attrs.clone();
    }
    let mut normalized = Map::new();
    for (key, value) in php::entries(attrs) {
        let target = match key.as_str() {
            "TRABISDOMAINCATEGORY" => "TRABISDOMAINCATEGORY",
            "TRABISCITIZENID" | "TRABISCITIZIENID" => "TRABISCITIZENID",
            "TRABISTAXOFFICE" => "TRABISTAXOFFICE",
            "TRABISTAXNUMBER" => "TRABISTAXNUMBER",
            _ => continue,
        };
        if key == "TRABISCITIZIENID" && normalized.contains_key("TRABISCITIZENID") {
            continue;
        }
        normalized.insert(target.to_string(), value.clone());
    }
    if let Some(cat) = normalized
        .get("TRABISDOMAINCATEGORY")
        .filter(|v| !v.is_null())
    {
        let c = php::lower(php::trim(&php::strval(cat)));
        if matches!(c.as_str(), "1" | "owner" | "personal" | "individual") {
            normalized.insert("TRABISDOMAINCATEGORY".into(), json!("Owner"));
        } else if matches!(c.as_str(), "0" | "corporate" | "company") {
            normalized.insert("TRABISDOMAINCATEGORY".into(), json!("Corporate"));
        }
    }
    Value::Object(normalized)
}

/// php-dna `stringifyAttributes()`: every value becomes a string.
pub(crate) fn stringify_attributes(attrs: &Value) -> Map<String, Value> {
    let mut out = Map::new();
    for (key, value) in php::entries(attrs) {
        let s = match value {
            Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            Value::Null => String::new(),
            Value::Number(_) | Value::String(_) => php::strval(value),
            other => php::json_encode(other),
        };
        out.insert(key, json!(s));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phones() {
        assert_eq!(
            normalize_phone("+90 (532) 374 33 88", ""),
            ("5323743388".into(), "90".into())
        );
        assert_eq!(
            normalize_phone("0532 374 33 88", "90"),
            ("5323743388".into(), "90".into())
        );
        assert_eq!(
            normalize_phone("905323743388", "90"),
            ("5323743388".into(), "90".into())
        );
        assert_eq!(normalize_phone("", "1"), ("".into(), "1".into()));
        assert_eq!(
            normalize_phone("+1 (202) 555-0100", "90"),
            ("2025550100".into(), "1".into())
        );
        assert_eq!(
            normalize_phone("+44 20 7123 4567", "90"),
            ("2071234567".into(), "44".into())
        );
        assert_eq!(calling_code_prefix("358401234567"), "358");
        assert_eq!(calling_code_prefix("0"), "");
        assert_eq!(
            normalize_phone("0090 212 123 45 67", ""),
            ("2121234567".into(), "90".into())
        );
    }

    #[test]
    fn names() {
        assert_eq!(sanitize_person_name("  Jane 3rd "), "Jane rd");
        assert_eq!(sanitize_person_name("Çağrı O'Neil-Öz"), "Çağrı O'Neil-Öz");
        assert_eq!(sanitize_person_name("1234"), "");
    }

    #[test]
    fn countries() {
        assert_eq!(normalize_country_code("tr"), "TR");
        assert_eq!(normalize_country_code("Türkiye"), "TR");
        assert_eq!(normalize_country_code("Narnia"), "Narnia");
        assert_eq!(normalize_country_code(" "), "");
    }

    #[test]
    fn tr_attributes() {
        let out = normalize_tr_attributes(&json!({
            "TRABISDOMAINCATEGORY": 1, "TRABISCITIZIENID": "111", "TRABISNAMESURNAME": "x"
        }));
        assert_eq!(
            out,
            json!({"TRABISDOMAINCATEGORY": "Owner", "TRABISCITIZENID": "111"})
        );
        let out = normalize_tr_attributes(
            &json!({"TRABISCITIZENID": "a", "TRABISCITIZIENID": "b", "TRABISDOMAINCATEGORY": "company"}),
        );
        assert_eq!(out["TRABISCITIZENID"], "a");
        assert_eq!(out["TRABISDOMAINCATEGORY"], "Corporate");
    }

    #[test]
    fn stringify() {
        let out = stringify_attributes(&json!({"a": 1, "b": true, "c": null, "d": 1.5, "e": [1]}));
        assert_eq!(
            Value::Object(out),
            json!({"a": "1", "b": "true", "c": "", "d": "1.5", "e": "[1]"})
        );
    }
}
