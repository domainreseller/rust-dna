//! DNS zone and URL forwarding methods (php-dna `DNARest` zone section).

use serde_json::{json, Value};

use crate::client::{local_error, set_error, DomainNameApi};
use crate::php::{self, get, obj};
use crate::{FORWARD_TYPES, RESULT_ERROR, RESULT_OK, ZONE_RECORD_TYPES};

/// Reduce a record name to the relative label the gateway expects
/// (`"@"`, the apex and `"www.example.com"` all become relative).
fn relativize_record_name(name: &str, domain_name: &str) -> String {
    let name = php::trim(name);
    let domain = php::lower(php::trim(domain_name));
    let domain = domain.trim_end_matches('.');
    if name.is_empty() || name == "@" {
        return String::new();
    }
    let bare = name.trim_end_matches('.');
    if php::strcaseeq(bare, domain) {
        return String::new();
    }
    let suffix = format!(".{domain}");
    if bare.len() > suffix.len() {
        let cut = bare.len() - suffix.len();
        if bare.is_char_boundary(cut) && php::strcaseeq(&bare[cut..], &suffix) {
            return bare[..cut].to_string();
        }
    }
    bare.to_string()
}

/// Relative label → FQDN as the zone listing returns it (`www` →
/// `www.example.com.`). Already-qualified names pass through.
fn qualify_record_name(name: &str, domain_name: &str) -> String {
    let name = php::trim(name);
    let domain = php::lower(php::trim(domain_name));
    let domain = domain.trim_end_matches('.');
    let apex = format!("{domain}.");
    if name.is_empty() || name == "@" {
        return apex;
    }
    let bare = name.trim_end_matches('.');
    if php::strcaseeq(bare, domain) || php::lower(bare).ends_with(&format!(".{domain}")) {
        return format!("{bare}.");
    }
    format!("{bare}.{apex}")
}

/// Error message when the type is not supported, `None` when it is.
fn unsupported_zone_type(kind: &str) -> Option<String> {
    let kind = php::upper(php::trim(kind));
    if kind.is_empty() {
        return Some("Record type is required.".to_string());
    }
    if !ZONE_RECORD_TYPES.contains(&kind.as_str()) {
        return Some(format!(
            "Record type \"{kind}\" is not supported by the DNS zone API. Supported types: {}.",
            ZONE_RECORD_TYPES.join(", ")
        ));
    }
    None
}

/// Whether two record values mean the same (TXT quotes, trailing dots, case).
fn same_record_value(kind: &str, a: &str, b: &str) -> bool {
    let txt = php::upper(kind) == "TXT";
    let normalize = |v: &str| -> String {
        let v = php::trim(v);
        if txt {
            if v.len() > 1 && v.starts_with('"') && v.ends_with('"') {
                return v[1..v.len() - 1].to_string();
            }
            return v.to_string();
        }
        php::lower(v.trim_end_matches('.'))
    };
    normalize(a) == normalize(b)
}

/// `ZoneCreateDto` body.
fn build_zone_struct(name: &str, kind: &str, content: &Value, ttl: i64) -> Value {
    let ttl = zone_ttl(ttl);
    let contents: Vec<Value> = match content {
        Value::Null => Vec::new(),
        Value::Array(_) | Value::Object(_) => php::values(content)
            .into_iter()
            .map(|v| json!(php::strval(v)))
            .collect(),
        other => vec![json!(php::strval(other))],
    };
    obj([
        ("name", json!(name)),
        ("ttl", json!(ttl)),
        ("type", json!(php::upper(kind))),
        ("contents", Value::Array(contents)),
    ])
}

/// php-dna `buildZoneStruct()` TTL rule: below 1 becomes 3600, above 86400
/// becomes 86400.
fn zone_ttl(ttl: i64) -> i64 {
    if ttl < 1 {
        3600
    } else {
        ttl.min(86_400)
    }
}

/// One record set read back from the zone: `(ttl, contents)`.
enum SetRead {
    Found { ttl: i64, contents: Vec<String> },
    Failed(Value),
}

impl DomainNameApi {
    /// Every DNS record of the domain, one row per value:
    /// `{"result": "OK", "data": {"records": [{"Name", "Type", "TTL",
    /// "Content", "Disabled"}]}}`. Names are FQDNs with a trailing dot.
    pub fn get_resource_records(&mut self, domain_name: &str) -> Value {
        let response =
            match self.request("GET", "domains/zones", &json!({"domainName": domain_name})) {
                Ok(r) => r,
                Err(e) => return self.error_envelope(&e, "GET_ZONE"),
            };
        let mut records = Vec::new();
        for set in php::values(&php::to_array(&response)) {
            if !php::is_array(set) {
                continue;
            }
            let name = get(set, "name").cloned().unwrap_or(json!(""));
            let kind = get(set, "type").cloned().unwrap_or(json!(""));
            let ttl = php::intval_or(get(set, "ttl"), 0);
            let rows = get(set, "records").cloned().unwrap_or(json!([]));
            if !php::is_array(&rows) || !php::truthy(&rows) {
                records.push(obj([
                    ("Name", name),
                    ("Type", kind),
                    ("TTL", json!(ttl)),
                    ("Content", json!("")),
                    ("Disabled", json!(false)),
                ]));
                continue;
            }
            for rec in php::values(&rows) {
                records.push(obj([
                    ("Name", name.clone()),
                    ("Type", kind.clone()),
                    ("TTL", json!(ttl)),
                    ("Content", get(rec, "content").cloned().unwrap_or(json!(""))),
                    ("Disabled", json!(!php::empty(get(rec, "disabled")))),
                ]));
            }
        }
        obj([
            ("result", json!(RESULT_OK)),
            ("data", obj([("records", Value::Array(records))])),
        ])
    }

    /// Add a record set. `content` is one value (`json!("192.0.2.1")`) or a
    /// list. Names may be relative, `@`, or fully qualified. php-dna
    /// defaults: `ttl` 3600, `apply` true.
    ///
    /// Note: a second add for the same name and type replaces the first on
    /// the gateway; use [`add_resource_record_value`](Self::add_resource_record_value)
    /// to append a value.
    pub fn add_resource_record(
        &mut self,
        domain_name: &str,
        name: &str,
        kind: &str,
        content: &Value,
        ttl: i64,
        apply: bool,
    ) -> Value {
        if let Some(why) = unsupported_zone_type(kind) {
            return local_error("ADD_ZONE", &why);
        }
        let name = relativize_record_name(name, domain_name);
        let body = json!({"zoneStruct": build_zone_struct(&name, kind, content, ttl)});
        let endpoint = format!(
            "domains/zones?{}",
            php::http_build_query(&json!({"domainName": domain_name}))
        );
        if let Err(e) = self.request("POST", &endpoint, &body) {
            return self.error_envelope(&e, "ADD_ZONE");
        }
        if apply {
            self.apply_resource_records(domain_name);
        }
        obj([
            ("result", json!(RESULT_OK)),
            (
                "data",
                obj([
                    ("Name", json!(name)),
                    ("Type", json!(kind)),
                    ("TTL", json!(ttl)),
                    ("Content", content.clone()),
                ]),
            ),
        ])
    }

    /// Edit the record set identified by `record_name`. php-dna defaults:
    /// `ttl` 3600, `apply` true.
    #[allow(clippy::too_many_arguments)]
    pub fn edit_resource_record(
        &mut self,
        domain_name: &str,
        record_name: &str,
        name: &str,
        kind: &str,
        content: &Value,
        ttl: i64,
        apply: bool,
    ) -> Value {
        if let Some(why) = unsupported_zone_type(kind) {
            return local_error("EDIT_ZONE", &why);
        }
        let name = relativize_record_name(name, domain_name);
        let body = json!({"zoneStruct": build_zone_struct(&name, kind, content, ttl)});
        let endpoint = format!(
            "domains/zones?{}",
            php::http_build_query(&json!({"domainName": domain_name, "recordName": record_name}))
        );
        if let Err(e) = self.request("PUT", &endpoint, &body) {
            return self.error_envelope(&e, "EDIT_ZONE");
        }
        if apply {
            self.apply_resource_records(domain_name);
        }
        obj([
            ("result", json!(RESULT_OK)),
            (
                "data",
                obj([
                    ("Name", json!(name)),
                    ("Type", json!(kind)),
                    ("TTL", json!(ttl)),
                    ("Content", content.clone()),
                ]),
            ),
        ])
    }

    /// Delete a record (the gateway removes the whole name + type set).
    /// php-dna default: `apply` true.
    pub fn delete_resource_record(
        &mut self,
        domain_name: &str,
        name: &str,
        record: &str,
        record_type: &str,
        apply: bool,
    ) -> Value {
        let query = obj([
            ("domainName", json!(domain_name)),
            ("Name", json!(name)),
            ("Record", json!(record)),
            ("RecordType", json!(record_type)),
        ]);
        if let Err(e) = self.request("DELETE", "domains/zones", &query) {
            return self.error_envelope(&e, "DELETE_ZONE");
        }
        if apply {
            self.apply_resource_records(domain_name);
        }
        obj([
            ("result", json!(RESULT_OK)),
            (
                "data",
                obj([
                    ("Name", json!(name)),
                    ("Type", json!(record_type)),
                    ("Content", json!(record)),
                ]),
            ),
        ])
    }

    /// Commit staged zone changes.
    pub fn apply_resource_records(&mut self, domain_name: &str) -> Value {
        let endpoint = format!(
            "domains/zones/apply?{}",
            php::http_build_query(&json!({"domainName": domain_name}))
        );
        match self.request("POST", &endpoint, &json!([])) {
            Ok(_) => obj([
                ("result", json!(RESULT_OK)),
                ("data", json!({"DomainName": domain_name})),
            ]),
            Err(e) => self.error_envelope(&e, "APPLY_ZONE"),
        }
    }

    /// Make the record set `name` + `kind` hold exactly `contents` (an empty
    /// list removes the set). php-dna defaults: `ttl` 3600, `apply` true.
    pub fn set_resource_record_set<I, S>(
        &mut self,
        domain_name: &str,
        name: &str,
        kind: &str,
        contents: I,
        ttl: i64,
        apply: bool,
    ) -> Value
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        if let Some(why) = unsupported_zone_type(kind) {
            return local_error("SET_ZONE", &why);
        }
        let name = relativize_record_name(name, domain_name);
        let contents: Vec<String> = contents
            .into_iter()
            .map(|c| c.as_ref().to_string())
            .filter(|c| !php::trim(c).is_empty())
            .collect();
        if contents.is_empty() {
            return self.delete_resource_record_set(domain_name, &name, kind, apply, "");
        }
        let body = json!({"zoneStruct": build_zone_struct(&name, kind, &json!(contents), ttl)});
        let endpoint = format!(
            "domains/zones?{}",
            php::http_build_query(&json!({"domainName": domain_name}))
        );
        if let Err(e) = self.request("POST", &endpoint, &body) {
            return self.error_envelope(&e, "SET_ZONE");
        }
        if apply {
            self.apply_resource_records(domain_name);
        }
        obj([
            ("result", json!(RESULT_OK)),
            (
                "data",
                obj([
                    ("Name", json!(name)),
                    ("Type", json!(php::upper(kind))),
                    ("TTL", json!(ttl)),
                    ("Contents", json!(contents)),
                ]),
            ),
        ])
    }

    /// Remove every value under `name` + `kind`. `known_value` (any current
    /// value) saves a lookup; pass `""` to look it up. Removing a set that
    /// does not exist is a success with `Removed: false`. php-dna defaults:
    /// `apply` true, `known_value` "".
    pub fn delete_resource_record_set(
        &mut self,
        domain_name: &str,
        name: &str,
        kind: &str,
        apply: bool,
        known_value: &str,
    ) -> Value {
        let kind = php::upper(php::trim(kind));
        let mut value = known_value.to_string();
        if value.is_empty() {
            let current = self.get_resource_records(domain_name);
            if current.get("result").and_then(Value::as_str) != Some(RESULT_OK) {
                return current;
            }
            let fqdn = qualify_record_name(name, domain_name);
            for rec in php::values(&current["data"]["records"]) {
                if php::strcaseeq(&php::strval(&rec["Name"]), &fqdn)
                    && php::strcaseeq(&php::strval(&rec["Type"]), &kind)
                {
                    value = php::strval(&rec["Content"]);
                    break;
                }
            }
            if value.is_empty() {
                return obj([
                    ("result", json!(RESULT_OK)),
                    (
                        "data",
                        obj([
                            ("Name", json!(name)),
                            ("Type", json!(kind)),
                            ("Removed", json!(false)),
                        ]),
                    ),
                ]);
            }
        }
        let fqdn = qualify_record_name(name, domain_name);
        self.delete_resource_record(domain_name, &fqdn, &value, &kind, apply)
    }

    fn read_resource_record_set(&mut self, domain_name: &str, name: &str, kind: &str) -> SetRead {
        let current = self.get_resource_records(domain_name);
        if current.get("result").and_then(Value::as_str) != Some(RESULT_OK) {
            return SetRead::Failed(current);
        }
        let fqdn = qualify_record_name(name, domain_name);
        let kind = php::upper(php::trim(kind));
        let mut contents = Vec::new();
        let mut ttl = 0;
        for rec in php::values(&current["data"]["records"]) {
            if !php::strcaseeq(&php::strval(&rec["Name"]), &fqdn)
                || !php::strcaseeq(&php::strval(&rec["Type"]), &kind)
            {
                continue;
            }
            let content = php::strval(&rec["Content"]);
            if !content.is_empty() {
                contents.push(content);
            }
            ttl = php::intval(&rec["TTL"]);
        }
        SetRead::Found { ttl, contents }
    }

    /// Add one value to a record set, keeping the values already there
    /// (read, merge, write). Adding a value that is present is a no-op
    /// (`Changed: false`). `ttl` `None` keeps the set's TTL. php-dna
    /// defaults: `ttl` None, `apply` true.
    pub fn add_resource_record_value(
        &mut self,
        domain_name: &str,
        name: &str,
        kind: &str,
        value: &str,
        ttl: Option<i64>,
        apply: bool,
    ) -> Value {
        if let Some(why) = unsupported_zone_type(kind) {
            return local_error("ADD_ZONE", &why);
        }
        let name = relativize_record_name(name, domain_name);
        let (set_ttl, contents) = match self.read_resource_record_set(domain_name, &name, kind) {
            SetRead::Failed(envelope) => return envelope,
            SetRead::Found { ttl, contents } => (ttl, contents),
        };
        if contents
            .iter()
            .any(|existing| same_record_value(kind, existing, value))
        {
            return obj([
                ("result", json!(RESULT_OK)),
                (
                    "data",
                    obj([
                        ("Name", json!(name)),
                        ("Type", json!(php::upper(kind))),
                        ("Contents", json!(contents)),
                        ("Changed", json!(false)),
                    ]),
                ),
            ]);
        }
        let mut contents = contents;
        contents.push(value.to_string());
        let ttl = ttl.unwrap_or(if set_ttl != 0 { set_ttl } else { 3600 });
        self.set_resource_record_set(domain_name, &name, kind, contents, ttl, apply)
    }

    /// Remove one value from a record set, leaving the others. Removing the
    /// last value deletes the set; removing an absent value is a no-op.
    /// php-dna default: `apply` true.
    pub fn remove_resource_record_value(
        &mut self,
        domain_name: &str,
        name: &str,
        kind: &str,
        value: &str,
        apply: bool,
    ) -> Value {
        if let Some(why) = unsupported_zone_type(kind) {
            return local_error("DELETE_ZONE", &why);
        }
        let name = relativize_record_name(name, domain_name);
        let (set_ttl, contents) = match self.read_resource_record_set(domain_name, &name, kind) {
            SetRead::Failed(envelope) => return envelope,
            SetRead::Found { ttl, contents } => (ttl, contents),
        };
        let mut remaining = Vec::new();
        let mut found = false;
        for existing in &contents {
            if !found && same_record_value(kind, existing, value) {
                found = true;
                continue;
            }
            remaining.push(existing.clone());
        }
        if !found {
            return obj([
                ("result", json!(RESULT_OK)),
                (
                    "data",
                    obj([
                        ("Name", json!(name)),
                        ("Type", json!(php::upper(kind))),
                        ("Contents", json!(contents)),
                        ("Changed", json!(false)),
                    ]),
                ),
            ]);
        }
        if remaining.is_empty() {
            return self.delete_resource_record_set(domain_name, &name, kind, apply, value);
        }
        let ttl = if set_ttl != 0 { set_ttl } else { 3600 };
        self.set_resource_record_set(domain_name, &name, kind, remaining, ttl, apply)
    }

    /// Replace one value of a record set, keeping the others. php-dna
    /// defaults: `ttl` None (keep the set's TTL), `apply` true.
    #[allow(clippy::too_many_arguments)]
    pub fn replace_resource_record_value(
        &mut self,
        domain_name: &str,
        name: &str,
        kind: &str,
        old_value: &str,
        new_value: &str,
        ttl: Option<i64>,
        apply: bool,
    ) -> Value {
        if let Some(why) = unsupported_zone_type(kind) {
            return local_error("EDIT_ZONE", &why);
        }
        let name = relativize_record_name(name, domain_name);
        let (set_ttl, contents) = match self.read_resource_record_set(domain_name, &name, kind) {
            SetRead::Failed(envelope) => return envelope,
            SetRead::Found { ttl, contents } => (ttl, contents),
        };
        let mut updated = Vec::new();
        let mut found = false;
        for existing in &contents {
            if !found && same_record_value(kind, existing, old_value) {
                found = true;
                updated.push(new_value.to_string());
                continue;
            }
            updated.push(existing.clone());
        }
        if !found {
            let why = format!(
                "No {} record \"{}\" with value \"{}\" to edit.",
                php::upper(kind),
                name,
                old_value
            );
            return local_error("EDIT_ZONE", &why);
        }
        let ttl = ttl.unwrap_or(if set_ttl != 0 { set_ttl } else { 3600 });
        self.set_resource_record_set(domain_name, &name, kind, updated, ttl, apply)
    }

    // ------------------------------------------------------------------
    // Forwarding
    // ------------------------------------------------------------------

    /// URL forwarding of the domain. When none is set the result is OK with
    /// `Enabled: false`.
    pub fn get_forwarding(&mut self, domain_name: &str) -> Value {
        match self.request(
            "GET",
            "domains/forwards",
            &json!({"domainName": domain_name}),
        ) {
            Ok(r) => obj([
                ("result", json!(RESULT_OK)),
                (
                    "data",
                    obj([
                        ("Enabled", json!(!php::empty(get(&r, "redirectAlias")))),
                        (
                            "RedirectAddress",
                            json!(php::strval_or(get(&r, "redirectAlias"), "")),
                        ),
                        (
                            "ForwardType",
                            json!(php::strval_or(get(&r, "redirectType"), "")),
                        ),
                    ]),
                ),
            ]),
            Err(e) => {
                let raw = get(&self.get_response_data(), "raw_response")
                    .cloned()
                    .unwrap_or_else(|| json!(e.message));
                let raw_str = php::strval(&raw).to_ascii_lowercase();
                if raw_str.contains("not found")
                    || e.message.to_ascii_lowercase().contains("not found")
                {
                    return obj([
                        ("result", json!(RESULT_OK)),
                        (
                            "data",
                            json!({"Enabled": false, "RedirectAddress": "", "ForwardType": ""}),
                        ),
                    ]);
                }
                let details = get(&self.last_parsed_response, "Details")
                    .cloned()
                    .unwrap_or(raw);
                obj([
                    ("result", json!(RESULT_ERROR)),
                    (
                        "error",
                        set_error(json!(format!("API_{}", e.code)), json!(e.message), details),
                    ),
                ])
            }
        }
    }

    /// Point the domain at a URL. `forward_type` is `Standard` (redirect) or
    /// `Frame` (masked), case-insensitive; php-dna default `Standard`.
    pub fn set_forwarding(
        &mut self,
        domain_name: &str,
        redirect_address: &str,
        forward_type: &str,
    ) -> Value {
        let kind = php::ucfirst(&php::lower(php::trim(forward_type)));
        if !FORWARD_TYPES.contains(&kind.as_str()) {
            let why = format!(
                "Forward type \"{forward_type}\" is not supported. Supported types: {}.",
                FORWARD_TYPES.join(", ")
            );
            return local_error("SET_FORWARD", &why);
        }
        if php::trim(redirect_address).is_empty() {
            return local_error("SET_FORWARD", "Redirect address is required.");
        }
        let payload = obj([
            ("domainName", json!(domain_name)),
            ("redirectAddress", json!(redirect_address)),
            ("forwardType", json!(kind)),
        ]);
        match self.request("POST", "domains/forwards", &payload) {
            Ok(_) => obj([
                ("result", json!(RESULT_OK)),
                (
                    "data",
                    obj([
                        ("RedirectAddress", json!(redirect_address)),
                        ("ForwardType", json!(kind)),
                    ]),
                ),
            ]),
            Err(e) => self.error_envelope(&e, "SET_FORWARD"),
        }
    }

    /// Remove URL forwarding.
    pub fn delete_forwarding(&mut self, domain_name: &str) -> Value {
        match self.request(
            "DELETE",
            "domains/forwards",
            &json!({"DomainName": domain_name}),
        ) {
            Ok(_) => obj([
                ("result", json!(RESULT_OK)),
                ("data", json!({"DomainName": domain_name})),
            ]),
            Err(e) => self.error_envelope(&e, "DELETE_FORWARD"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert_eq!(relativize_record_name("@", "example.com"), "");
        assert_eq!(relativize_record_name("example.com.", "example.com"), "");
        assert_eq!(
            relativize_record_name("www.Example.com", "example.com"),
            "www"
        );
        assert_eq!(relativize_record_name("www", "example.com"), "www");
        assert_eq!(qualify_record_name("", "example.com"), "example.com.");
        assert_eq!(
            qualify_record_name("www", "example.com"),
            "www.example.com."
        );
        assert_eq!(
            qualify_record_name("www.example.com.", "example.com"),
            "www.example.com."
        );
    }

    #[test]
    fn values_compare_like_php() {
        assert!(same_record_value("TXT", "\"v=spf1\"", "v=spf1"));
        assert!(!same_record_value("TXT", "V=SPF1", "v=spf1"));
        assert!(same_record_value(
            "MX",
            "10 MX1.example.com.",
            "10 mx1.example.com"
        ));
    }

    #[test]
    fn ttl_rules() {
        assert_eq!(zone_ttl(0), 3600);
        assert_eq!(zone_ttl(999_999), 86_400);
        assert_eq!(zone_ttl(300), 300);
        assert_eq!(build_zone_struct("a", "mx", &json!("x"), 0)["ttl"], 3600);
        assert_eq!(
            build_zone_struct("a", "mx", &json!(["x", 1]), 99_999_999)["contents"],
            json!(["x", "1"])
        );
    }

    #[test]
    fn types() {
        assert!(unsupported_zone_type("a").is_none());
        assert_eq!(
            unsupported_zone_type(" ").as_deref(),
            Some("Record type is required.")
        );
        assert!(unsupported_zone_type("NS")
            .unwrap()
            .contains("\"NS\" is not supported"));
    }
}
