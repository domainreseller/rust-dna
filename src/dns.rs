//! The DNS zone of domains on the platform nameservers. CLI: `dna dns`.
//!
//! A record set is all records with the same name and type; it has one TTL, and a record is identified by its value.
//! Changes are live at once in the platform's DNS, but resolve publicly only when the domain uses the platform
//! nameservers ([`DnsClient::apply`]). Zone calls are paced one second apart, the gateway's limit.
//!
//! The OTE test environment answers every zone call with HTTP 500, so zone code can only be tried in production. Use
//! a domain you do not mind changing.

use crate::de::{model, str_enum};
use crate::request::trim_all;
use crate::{Client, Empty, Endpoint, Error, JsonBody, OperationSafety, Query, Request, Validator};
use serde_json::Value;
use std::net::IpAddr;

/// The TTL used when a request sets none, in seconds.
pub const DEFAULT_TTL: u32 = 3600;

str_enum! {
    /// The type of a DNS record. [`RecordType::parse`] accepts any case (`"txt"` is `TXT`); the zone API also
    /// supports rarer types, which are kept as `Other` with their canonical spelling.
    pub enum RecordType {
        /// An IPv4 address.
        A => "A",
        /// An IPv6 address.
        Aaaa => "AAAA",
        /// A CNAME-like record allowed at the apex.
        Alias => "ALIAS",
        /// Certificate authority authorization: three parts, for example `0 issue "letsencrypt.org"`.
        Caa => "CAA",
        /// An alias of another name. Not allowed at the apex.
        Cname => "CNAME",
        /// A delegation signer.
        Ds => "DS",
        /// A mail server: `PRIORITY HOST`, for example `10 mail.example.com`.
        Mx => "MX",
        /// A naming authority pointer.
        Naptr => "NAPTR",
        /// A nameserver.
        Ns => "NS",
        /// A pointer.
        Ptr => "PTR",
        /// The start of authority.
        Soa => "SOA",
        /// A service locator.
        Srv => "SRV",
        /// A TLS certificate association.
        Tlsa => "TLSA",
        /// Text; the API adds the surrounding quotes.
        Txt => "TXT",
    }
}

/// The rarer record types the zone API also accepts, in their canonical spelling.
const OTHER_RECORD_TYPES: &[&str] = &[
    "DNSKey",
    "NSEC",
    "NSEC3",
    "NSEC3Param",
    "RRSIG",
    "AFSDB",
    "ATMA",
    "CERT",
    "DHCID",
    "DName",
    "HInfo",
    "ISDN",
    "LOC",
    "MB",
    "MG",
    "MInfo",
    "MR",
    "NSAP",
    "RP",
    "RT",
    "X25",
];

impl RecordType {
    /// The canonical form: known types as they are, rarer supported types in their canonical spelling, anything
    /// else trimmed.
    fn canonical(&self) -> RecordType {
        match self {
            RecordType::Other(text) => {
                let text = text.trim();
                match OTHER_RECORD_TYPES.iter().find(|t| t.eq_ignore_ascii_case(text)) {
                    Some(t) => RecordType::Other(t.to_string()),
                    None => RecordType::parse(text),
                }
            }
            known => known.clone(),
        }
    }

    /// Returns whether the zone API supports this type.
    pub fn is_supported(&self) -> bool {
        match self.canonical() {
            RecordType::Other(text) => OTHER_RECORD_TYPES.contains(&text.as_str()),
            _ => true,
        }
    }
}

model! {
    /// One value of a record set.
    pub struct Record {
        /// The value, for example `"192.0.2.10"`.
        "content" => pub content: String,
        /// Whether the record is disabled.
        "disabled" => pub disabled: bool,
    }
}

model! {
    /// The records with one name and type.
    pub struct RecordSet {
        /// The fully qualified name with a trailing dot, for example `"www.example.com."`.
        "name" => pub name: String,
        /// The type.
        "type" => pub record_type: RecordType,
        /// The TTL in seconds.
        "ttl" => pub ttl: u32,
        /// The pending change, if any.
        "changeType" => pub change_type: String,
        /// The values.
        "records" => pub records: Vec<Record>,
    }
}

impl RecordSet {
    /// Returns the contents of the set's records.
    pub fn values(&self) -> Vec<&str> {
        self.records.iter().map(|r| r.content.as_str()).collect()
    }

    /// Returns whether the set has the given name and type. `name` may be relative (`"www"`), the apex (`""` or
    /// `"@"`) or fully qualified.
    pub fn matches(&self, domain_name: &str, name: &str, record_type: &RecordType) -> bool {
        self.record_type.as_str().eq_ignore_ascii_case(record_type.canonical().as_str())
            && self.name.trim_end_matches('.').eq_ignore_ascii_case(fqdn(domain_name, name).trim_end_matches('.'))
    }
}

/// Returns the fully qualified form of a record name, the form the zone listing uses.
///
/// ```
/// use domainnameapi::dns::fqdn;
///
/// assert_eq!(fqdn("example.com", "www"), "www.example.com.");
/// assert_eq!(fqdn("example.com", "@"), "example.com.");
/// assert_eq!(fqdn("example.com", "mail.example.com."), "mail.example.com.");
/// ```
pub fn fqdn(domain_name: &str, name: &str) -> String {
    let domain = domain_name.trim().trim_end_matches('.').to_ascii_lowercase();
    let name = name.trim().trim_end_matches('.').to_ascii_lowercase();
    if name.is_empty() || name == "@" || name == domain {
        format!("{domain}.")
    } else if name.ends_with(&format!(".{domain}")) {
        format!("{name}.")
    } else {
        format!("{name}.{domain}.")
    }
}

model! {
    /// The answer to [`DnsApplyRequest`].
    pub struct ZoneApplyResult {
        /// `false` when the domain already used the platform nameservers.
        "nameServersChanged" => pub name_servers_changed: bool,
        /// The nameservers before.
        "previousNameServers" => pub previous_name_servers: Vec<String>,
        /// The nameservers now.
        "currentNameServers" => pub current_name_servers: Vec<String>,
        /// The platform nameservers.
        "systemNameServers" => pub system_name_servers: Vec<String>,
    }
}

fn record_name(name: &str) -> String {
    match name.trim() {
        "@" => String::new(),
        other => other.to_string(),
    }
}

fn zone_struct(name: &str, record_type: &RecordType, ttl: Option<u32>, values: &[String]) -> Value {
    JsonBody::new()
        .insert("name", record_name(name))
        .insert("type", record_type.canonical().as_str())
        .insert("ttl", ttl.unwrap_or(DEFAULT_TTL))
        .insert("contents", trim_all(values))
        .into_value()
}

fn validate_record_type(v: &mut Validator, record_type: &RecordType) {
    if record_type.as_str().trim().is_empty() {
        v.add("type", "is required");
    } else if !record_type.is_supported() {
        v.add("type", format!("'{record_type}' is not a DNS record type the zone API supports"));
    }
}

fn validate_record_set(
    v: &mut Validator,
    domain_name: &str,
    name: &str,
    record_type: &RecordType,
    ttl: Option<u32>,
    values: &[String],
) {
    v.required("domainName", domain_name);
    validate_record_type(v, record_type);
    v.between("ttl", ttl, 1, 86400);
    let n = name.trim();
    v.check(n == "@" || !n.starts_with('@'), "name", "must not start with @ (use \"@\" alone for the apex)");
    let apex = n.is_empty()
        || n == "@"
        || n.trim_end_matches('.').eq_ignore_ascii_case(domain_name.trim().trim_end_matches('.'));
    let contents = trim_all(values);
    v.required_list("values", contents.len());
    match record_type.canonical() {
        RecordType::Cname => {
            v.check(!apex, "name", "a CNAME cannot be at the apex; use an ALIAS record");
            for c in &contents {
                v.check(
                    c.parse::<IpAddr>().is_err(),
                    "values",
                    format!("a CNAME points to a host name, not an IP address ({c})"),
                );
            }
        }
        RecordType::Mx => {
            for c in &contents {
                let parts: Vec<&str> = c.split_whitespace().collect();
                let ok = parts.len() == 2 && parts[0].parse::<u16>().is_ok() && parts[1].parse::<IpAddr>().is_err();
                v.check(
                    ok,
                    "values",
                    format!("an MX value is \"PRIORITY HOST\", for example \"10 mail.example.com\" ({c})"),
                );
            }
        }
        RecordType::Caa => {
            for c in &contents {
                v.check(
                    c.split_whitespace().count() == 3,
                    "values",
                    format!("a CAA value has 3 parts, for example `0 issue \"letsencrypt.org\"` ({c})"),
                );
            }
        }
        RecordType::A => {
            for c in &contents {
                v.check(
                    matches!(c.parse::<IpAddr>(), Ok(IpAddr::V4(_))),
                    "values",
                    format!("{c} is not an IPv4 address"),
                );
            }
        }
        RecordType::Aaaa => {
            for c in &contents {
                v.check(
                    matches!(c.parse::<IpAddr>(), Ok(IpAddr::V6(_))),
                    "values",
                    format!("{c} is not an IPv6 address"),
                );
            }
        }
        _ => {}
    }
}

// ------------------------------------------------------------------------------------------------ requests

/// Lists the record sets of a domain. CLI: `dna dns list`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DnsListRequest {
    /// The domain.
    pub domain_name: String,
}

impl DnsListRequest {
    /// Lists the record sets of `domain_name`.
    pub fn new(domain_name: impl Into<String>) -> Self {
        DnsListRequest { domain_name: domain_name.into() }
    }
}

impl Request for DnsListRequest {
    type Response = Vec<RecordSet>;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/domains/zones")
    }

    fn query(&self) -> Query {
        Query::new().set_str("domainName", &self.domain_name)
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
    }
}

/// Adds values to a record set, creating the set (and the zone) when needed. Values already in the set stay.
/// CLI: `dna dns add`
///
/// ```
/// use domainnameapi::dns::{DnsAddRequest, RecordType};
///
/// let www = DnsAddRequest::new("example.com", "www", RecordType::A, ["192.0.2.10"]).ttl(300);
/// let mx = DnsAddRequest::new("example.com", "@", RecordType::Mx, ["10 mx1.example.net", "20 mx2.example.net"]);
/// let txt = DnsAddRequest::new("example.com", "@", "txt", ["v=spf1 -all"]);
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DnsAddRequest {
    /// The domain.
    pub domain_name: String,
    /// The name: relative (`"www"`), the apex (`""` or `"@"`) or fully qualified.
    pub name: String,
    /// The type.
    pub record_type: RecordType,
    /// The values, for example `"192.0.2.10"`, `"10 mail.example.com"` (MX) or `0 issue "letsencrypt.org"` (CAA).
    pub values: Vec<String>,
    /// The TTL in seconds, 1 to 86400; `None` uses [`DEFAULT_TTL`].
    pub ttl: Option<u32>,
}

impl DnsAddRequest {
    /// Adds `values` to the `record_type` set named `name`.
    pub fn new<S: Into<String>>(
        domain_name: impl Into<String>,
        name: impl Into<String>,
        record_type: impl Into<RecordType>,
        values: impl IntoIterator<Item = S>,
    ) -> Self {
        DnsAddRequest {
            domain_name: domain_name.into(),
            name: name.into(),
            record_type: record_type.into(),
            values: values.into_iter().map(Into::into).collect(),
            ttl: None,
        }
    }

    /// Sets the TTL in seconds.
    pub fn ttl(mut self, ttl: u32) -> Self {
        self.ttl = Some(ttl);
        self
    }
}

impl Request for DnsAddRequest {
    type Response = Empty;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/domains/zones", OperationSafety::Write)
    }

    fn query(&self) -> Query {
        Query::new().set_str("domainName", &self.domain_name)
    }

    fn body(&self) -> Option<Value> {
        let zone = zone_struct(&self.name, &self.record_type, self.ttl, &self.values);
        Some(JsonBody::new().insert("zoneStruct", zone).into_value())
    }

    fn validate(&self, v: &mut Validator) {
        validate_record_set(v, &self.domain_name, &self.name, &self.record_type, self.ttl, &self.values);
    }
}

/// Replaces every value of a record set (and its TTL); it can also rename the set. CLI: `dna dns update`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DnsUpdateRequest {
    /// The domain.
    pub domain_name: String,
    /// The name of the existing set.
    pub record_name: String,
    /// The type.
    pub record_type: RecordType,
    /// The values; they replace all values of the set.
    pub values: Vec<String>,
    /// The TTL in seconds, 1 to 86400; `None` uses [`DEFAULT_TTL`].
    pub ttl: Option<u32>,
    /// A new name for the set; empty keeps `record_name`.
    pub new_name: String,
}

impl DnsUpdateRequest {
    /// Replaces the values of the `record_type` set named `record_name` with `values`.
    pub fn new<S: Into<String>>(
        domain_name: impl Into<String>,
        record_name: impl Into<String>,
        record_type: impl Into<RecordType>,
        values: impl IntoIterator<Item = S>,
    ) -> Self {
        DnsUpdateRequest {
            domain_name: domain_name.into(),
            record_name: record_name.into(),
            record_type: record_type.into(),
            values: values.into_iter().map(Into::into).collect(),
            ttl: None,
            new_name: String::new(),
        }
    }

    /// Sets the TTL in seconds.
    pub fn ttl(mut self, ttl: u32) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Renames the set.
    pub fn new_name(mut self, name: impl Into<String>) -> Self {
        self.new_name = name.into();
        self
    }

    fn target_name(&self) -> &str {
        if self.new_name.trim().is_empty() {
            &self.record_name
        } else {
            &self.new_name
        }
    }
}

impl Request for DnsUpdateRequest {
    type Response = Empty;

    fn endpoint(&self) -> Endpoint {
        Endpoint::put("/api/v1/domains/zones", OperationSafety::Write).idempotent()
    }

    fn query(&self) -> Query {
        Query::new().set_str("domainName", &self.domain_name).set("recordName", record_name(&self.record_name))
    }

    fn body(&self) -> Option<Value> {
        let zone = zone_struct(self.target_name(), &self.record_type, self.ttl, &self.values);
        Some(JsonBody::new().insert("zoneStruct", zone).into_value())
    }

    fn validate(&self, v: &mut Validator) {
        validate_record_set(v, &self.domain_name, self.target_name(), &self.record_type, self.ttl, &self.values);
    }
}

/// Removes one value from a record set; the set's other values stay. CLI: `dna dns delete`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DnsDeleteRequest {
    /// The domain.
    pub domain_name: String,
    /// The name of the set.
    pub name: String,
    /// The type.
    pub record_type: RecordType,
    /// The exact value to remove.
    pub value: String,
}

impl DnsDeleteRequest {
    /// Removes `value` from the `record_type` set named `name`.
    pub fn new(
        domain_name: impl Into<String>,
        name: impl Into<String>,
        record_type: impl Into<RecordType>,
        value: impl Into<String>,
    ) -> Self {
        DnsDeleteRequest {
            domain_name: domain_name.into(),
            name: name.into(),
            record_type: record_type.into(),
            value: value.into(),
        }
    }
}

impl Request for DnsDeleteRequest {
    type Response = Empty;

    fn endpoint(&self) -> Endpoint {
        Endpoint::delete("/api/v1/domains/zones", OperationSafety::Destructive)
    }

    fn query(&self) -> Query {
        Query::new()
            .set_str("domainName", &self.domain_name)
            .set("Name", record_name(&self.name)) // "" is the apex, and is sent
            .set("RecordType", self.record_type.canonical().as_str())
            .set("Record", &self.value)
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        validate_record_type(v, &self.record_type);
        v.required("value", &self.value);
    }
}

/// Switches a domain to the platform nameservers so its zone is served. CLI: `dna dns apply`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DnsApplyRequest {
    /// The domain.
    pub domain_name: String,
}

impl DnsApplyRequest {
    /// Applies the zone of `domain_name`.
    pub fn new(domain_name: impl Into<String>) -> Self {
        DnsApplyRequest { domain_name: domain_name.into() }
    }
}

impl Request for DnsApplyRequest {
    type Response = ZoneApplyResult;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/domains/zones/apply", OperationSafety::Write).idempotent()
    }

    fn query(&self) -> Query {
        Query::new().set_str("domainName", &self.domain_name)
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
    }
}

// ------------------------------------------------------------------------------------------------ client

/// DNS zone records. Get it with [`Client::dns`].
#[derive(Debug, Clone, Copy)]
pub struct DnsClient<'a> {
    client: &'a Client,
}

impl<'a> DnsClient<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        DnsClient { client }
    }

    /// Returns the record sets of a domain. A domain without a zone has none.
    pub async fn list(&self, domain_name: &str) -> Result<Vec<RecordSet>, Error> {
        self.client.send(&DnsListRequest::new(domain_name)).await
    }

    /// Adds values to a record set, creating it (and the zone) when needed.
    pub async fn add(&self, request: DnsAddRequest) -> Result<(), Error> {
        self.client.send(&request).await.map(|_| ())
    }

    /// Replaces all values (and the TTL) of a record set.
    pub async fn update(&self, request: DnsUpdateRequest) -> Result<(), Error> {
        self.client.send(&request).await.map(|_| ())
    }

    /// Removes one value from a record set.
    pub async fn delete(&self, request: DnsDeleteRequest) -> Result<(), Error> {
        self.client.send(&request).await.map(|_| ())
    }

    /// Switches the domain to the platform nameservers, so the records managed here become live. This replaces any
    /// other DNS provider of the domain. It does nothing when the domain already uses them.
    pub async fn apply(&self, domain_name: &str) -> Result<ZoneApplyResult, Error> {
        self.client.send(&DnsApplyRequest::new(domain_name)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn errors(request: &impl Request) -> Vec<String> {
        let mut v = Validator::new();
        request.validate(&mut v);
        v.errors().iter().map(ToString::to_string).collect()
    }

    #[test]
    fn add_body_uses_relative_names_and_default_ttl() {
        let body = DnsAddRequest::new("example.com", "@", "txt", [" v=spf1 -all "]).body().unwrap();
        assert_eq!(body, json!({"zoneStruct": {"name": "", "type": "TXT", "ttl": 3600, "contents": ["v=spf1 -all"]}}));
    }

    #[test]
    fn records_are_checked() {
        assert!(errors(&DnsAddRequest::new("example.com", "www", RecordType::A, ["192.0.2.1"])).is_empty());
        assert_eq!(errors(&DnsAddRequest::new("example.com", "www", RecordType::A, ["2001:db8::1"])).len(), 1);
        assert_eq!(errors(&DnsAddRequest::new("example.com", "@", RecordType::Cname, ["target.example.net"])).len(), 1);
        assert_eq!(errors(&DnsAddRequest::new("example.com", "@", RecordType::Mx, ["mail.example.com"])).len(), 1);
        assert_eq!(errors(&DnsAddRequest::new("example.com", "@", RecordType::Caa, ["0 issue"])).len(), 1);
        assert_eq!(errors(&DnsAddRequest::new("example.com", "www", "BOGUS", ["x"])).len(), 1);
        assert!(errors(&DnsAddRequest::new("example.com", "www", "nsec3param", ["x"])).is_empty());
        assert_eq!(errors(&DnsAddRequest::new("example.com", "www", RecordType::A, ["192.0.2.1"]).ttl(0)).len(), 1);
        assert_eq!(errors(&DnsAddRequest::new("example.com", "www", RecordType::Txt, Vec::<String>::new())).len(), 1);
    }

    #[test]
    fn update_can_rename() {
        let request =
            DnsUpdateRequest::new("example.com", "www", RecordType::A, ["192.0.2.2"]).new_name("web").ttl(600);
        assert_eq!(request.body().unwrap()["zoneStruct"]["name"], "web");
        assert_eq!(request.query().pairs()[1], ("recordName".to_string(), "www".to_string()));
        assert!(request.endpoint().idempotent);
    }

    #[test]
    fn delete_sends_the_apex_as_empty_name() {
        let pairs = DnsDeleteRequest::new("example.com", "@", "mx", "10 mx.example.net").query();
        let keys: Vec<String> = pairs.pairs().iter().map(|(k, v)| format!("{k}={v}")).collect();
        assert_eq!(keys, ["domainName=example.com", "Name=", "RecordType=MX", "Record=10 mx.example.net"]);
    }

    #[test]
    fn record_sets_match_names() {
        let set: RecordSet = serde_json::from_value(json!({
            "name": "www.example.com.", "type": "A", "ttl": "300", "records": [{"content": "192.0.2.1", "disabled": false}]
        }))
        .unwrap();
        assert!(set.matches("example.com", "www", &RecordType::A));
        assert!(set.matches("example.com", "WWW.example.com.", &"a".into()));
        assert!(!set.matches("example.com", "@", &RecordType::A));
        assert_eq!(set.values(), ["192.0.2.1"]);
        assert_eq!(set.ttl, 300);
    }
}
