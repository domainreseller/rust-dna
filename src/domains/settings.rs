//! Domain settings: transfer lock, privacy, auth code, nameservers and contacts.

use super::{domain_only_request, ContactHandles, DnssecMode, DomainContacts};
use crate::de::model;
use crate::request::{domain_body, trim_all};
use crate::{Empty, Endpoint, JsonBody, OperationSafety, Request, Validator};
use serde_json::Value;

model! {
    /// A domain's transfer authorization (EPP) code.
    pub struct AuthCode {
        /// The domain.
        "domainName" => pub domain_name: String,
        /// The new auth code.
        "authCode" => pub auth_code: String,
    }
}

domain_only_request!(
    /// Turns the transfer lock on. CLI: `dna domain lock`
    DomainLockRequest,
    Empty,
    Endpoint::post("/api/v1/domains/lock", OperationSafety::Write).idempotent(),
    body
);

domain_only_request!(
    /// Turns the transfer lock off, which a transfer to another registrar needs. CLI: `dna domain unlock`
    DomainUnlockRequest,
    Empty,
    Endpoint::post("/api/v1/domains/unlock", OperationSafety::Write).idempotent(),
    body
);

domain_only_request!(
    /// Asks the registry for a new transfer auth code. CLI: `dna domain update-auth-code`
    DomainUpdateAuthCodeRequest,
    AuthCode,
    Endpoint::put("/api/v1/domains/auth", OperationSafety::Write),
    body
);

/// Turns WHOIS privacy protection on or off. CLI: `dna domain privacy`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DomainPrivacyRequest {
    /// The domain.
    pub domain_name: String,
    /// Whether privacy protection is on.
    pub enabled: bool,
}

impl DomainPrivacyRequest {
    /// Turns privacy protection of `domain_name` on or off.
    pub fn new(domain_name: impl Into<String>, enabled: bool) -> Self {
        DomainPrivacyRequest { domain_name: domain_name.into(), enabled }
    }
}

impl Request for DomainPrivacyRequest {
    type Response = Empty;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/domains/privacy", OperationSafety::Write).idempotent()
    }

    fn body(&self) -> Option<Value> {
        Some(
            JsonBody::new()
                .insert("domainName", self.domain_name.trim())
                .insert("privacyStatus", self.enabled)
                .into_value(),
        )
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
    }
}

/// Replaces the nameservers of a domain. CLI: `dna domain nameservers set`
///
/// ```
/// use domainnameapi::domains::{DnssecMode, DomainNameServersSetRequest};
///
/// let request = DomainNameServersSetRequest::new("example.com", ["ns1.example.net", "ns2.example.net"])
///     .dnssec_mode(DnssecMode::DisableDnssec);
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DomainNameServersSetRequest {
    /// The domain.
    pub domain_name: String,
    /// The complete new list of nameserver host names.
    pub name_servers: Vec<String>,
    /// What happens to a DNSSEC-signed domain. The default, [`DnssecMode::Block`], refuses the change while DS
    /// records exist.
    pub dnssec_mode: DnssecMode,
}

impl DomainNameServersSetRequest {
    /// Sets the nameservers of `domain_name`.
    pub fn new<S: Into<String>>(domain_name: impl Into<String>, name_servers: impl IntoIterator<Item = S>) -> Self {
        DomainNameServersSetRequest {
            domain_name: domain_name.into(),
            name_servers: name_servers.into_iter().map(Into::into).collect(),
            dnssec_mode: DnssecMode::Block,
        }
    }

    /// Sets what happens to a DNSSEC-signed domain.
    pub fn dnssec_mode(mut self, mode: DnssecMode) -> Self {
        self.dnssec_mode = mode;
        self
    }
}

impl Request for DomainNameServersSetRequest {
    type Response = Empty;

    fn endpoint(&self) -> Endpoint {
        Endpoint::put("/api/v1/domains/dns/name-server", OperationSafety::Write).idempotent()
    }

    fn body(&self) -> Option<Value> {
        let name_servers: Vec<String> =
            trim_all(&self.name_servers).into_iter().map(|n| n.to_ascii_lowercase()).collect();
        Some(
            JsonBody::new()
                .insert("domainName", self.domain_name.trim())
                .insert("nameServers", name_servers)
                .insert("dnssecMode", self.dnssec_mode)
                .into_value(),
        )
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        v.required_list("nameServers", trim_all(&self.name_servers).len());
        v.check(self.dnssec_mode.is_known(), "dnssecMode", "must be Block, KeepDnssec or DisableDnssec");
    }
}

/// Replaces contacts of a domain with inline contact details. Roles left `None` stay as they are.
/// CLI: `dna domain contacts update`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DomainContactsUpdateRequest {
    /// The domain.
    pub domain_name: String,
    /// The new contacts. Changing the registrant may start an owner-change process, with documents on some TLDs.
    pub contacts: DomainContacts,
}

impl DomainContactsUpdateRequest {
    /// Replaces the given contacts of `domain_name`.
    pub fn new(domain_name: impl Into<String>, contacts: DomainContacts) -> Self {
        DomainContactsUpdateRequest { domain_name: domain_name.into(), contacts }
    }
}

impl Request for DomainContactsUpdateRequest {
    type Response = Empty;

    fn endpoint(&self) -> Endpoint {
        Endpoint::put("/api/v1/domains/contacts/update", OperationSafety::Write)
    }

    fn body(&self) -> Option<Value> {
        Some(
            JsonBody::new()
                .insert("domainName", self.domain_name.trim())
                .insert("contacts", self.contacts.to_json())
                .into_value(),
        )
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        self.contacts.validate(v, "contacts", false);
    }
}

/// Points contact roles of a domain at existing contact handles. Roles left empty stay as they are.
/// CLI: `dna domain contacts set-handles`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DomainContactsSetHandlesRequest {
    /// The domain.
    pub domain_name: String,
    /// The handles.
    pub handles: ContactHandles,
}

impl DomainContactsSetHandlesRequest {
    /// Sets the given handles of `domain_name`.
    pub fn new(domain_name: impl Into<String>, handles: ContactHandles) -> Self {
        DomainContactsSetHandlesRequest { domain_name: domain_name.into(), handles }
    }
}

impl Request for DomainContactsSetHandlesRequest {
    type Response = Empty;

    fn endpoint(&self) -> Endpoint {
        Endpoint::put("/api/v1/domains/contacts/update-with-handle", OperationSafety::Write)
    }

    fn body(&self) -> Option<Value> {
        Some(
            JsonBody::new()
                .insert("domainName", self.domain_name.trim())
                .insert("handleItems", self.handles.to_json())
                .into_value(),
        )
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        self.handles.validate(v, "handles", false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn name_servers_are_lower_case() {
        let body = DomainNameServersSetRequest::new("example.com", ["NS1.Example.NET", " "])
            .dnssec_mode(DnssecMode::KeepDnssec)
            .body();
        assert_eq!(
            body.unwrap(),
            json!({"domainName": "example.com", "nameServers": ["ns1.example.net"], "dnssecMode": 1})
        );
    }

    #[test]
    fn lock_is_idempotent() {
        assert!(DomainLockRequest::new("example.com").endpoint().idempotent);
        assert_eq!(DomainLockRequest::new(" example.com ").body().unwrap(), domain_body("example.com"));
    }
}
