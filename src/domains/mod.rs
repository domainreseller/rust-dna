//! Search, register, renew, transfer and manage domains, with URL forwarding ([`ForwardingClient`]), child
//! nameservers ([`ChildNameServersClient`]) and registry documents ([`DocumentsClient`]). CLI: `dna domain`.

use crate::contacts::{Contact, ContactType};
use crate::de::{int_enum, model};
use crate::request::{domain_body, paged, trim_all};
use crate::{
    Client, Empty, Endpoint, Error, ItemStream, JsonBody, OperationSafety, Page, Paging, Query, Request, Validator,
};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::BTreeMap;

mod documents;
mod forwarding;
mod hosts;
mod settings;

pub use documents::*;
pub use forwarding::*;
pub use hosts::*;
pub use settings::*;

int_enum! {
    /// The state of a domain on the platform.
    pub enum DomainStatus {
        /// Active.
        Active = 0 => "Active",
        /// The registration is being processed.
        WaitingForRegistration = 1 => "WaitingForRegistration",
        /// Registry documents must be uploaded ([`DocumentsClient::upload`]).
        WaitingForDocument = 2 => "WaitingForDocument",
        /// An internal transfer from another reseller is pending.
        WaitingForIncomingTransfer = 3 => "WaitingForIncomingTransfer",
        /// Moved to another registrar.
        TransferredOut = 4 => "TransferredOut",
        /// An incoming transfer was rejected.
        TransferRejected = 5 => "TransferRejected",
        /// In the redemption period; [`DomainsClient::restore`] may bring it back.
        PendingDelete = 7 => "PendingDelete",
        /// Gone.
        Deleted = 8 => "Deleted",
        /// The transfer confirmation e-mail was sent.
        ConfirmationEmailSend = 9 => "ConfirmationEmailSend",
        /// A transfer to another registrar is in progress.
        WaitingForOutgoingTransfer = 11 => "WaitingForOutgoingTransfer",
        /// Suspended, for example because the registrant e-mail was not verified within 15 days.
        PendingHold = 12 => "PendingHold",
        /// A change is being processed.
        ModificationPending = 15 => "ModificationPending",
        /// Subject to a legal process.
        InCase = 18 => "InCase",
        /// A restore is being processed.
        PendingRestore = 19 => "PendingRestore",
    }
}

model! {
    /// The availability and price of one domain.
    pub struct Availability {
        /// The domain.
        "domainName" => pub domain_name: String,
        /// The TLD.
        "tld" => pub tld: String,
        /// The registry's answer: `"AVAILABLE"`, `"NOTAVAILABLE"`, ... Use [`is_available`](Availability::is_available).
        "status" => pub status: String,
        /// Why the status is not `AVAILABLE`, for example `"Domain exists"`.
        "reason" => pub reason: String,
        /// The registration price.
        "price" => pub price: f64,
        /// The currency of the price.
        "currency" => pub currency: String,
        /// The period of the price, in years.
        "period" => pub period: i32,
        /// Whether this is a premium domain.
        "isPremium" => pub is_premium: bool,
        /// Whether the registry asks for documents.
        "isDocumentRequired" => pub is_document_required: bool,
    }
}

impl Availability {
    /// Returns whether the domain can be registered.
    pub fn is_available(&self) -> bool {
        self.status.trim().eq_ignore_ascii_case("AVAILABLE")
    }
}

model! {
    /// The answer to [`DomainSearchRequest`].
    pub struct DomainSearchResult {
        /// Whether the registry answered the search. When it is `false`, `message` and `reason` say why.
        "success" => pub success: bool,
        /// The API's result code.
        "code" => pub code: String,
        /// The API's message.
        "operationMessage" => pub message: String,
        /// Why the search failed.
        "reason" => pub reason: String,
        /// The availability and price.
        "info" => pub info: Availability,
    }
}

model! {
    /// The answer to [`DomainBulkSearchRequest`].
    pub struct DomainBulkSearchResult {
        /// Whether the registry answered the search.
        "success" => pub success: bool,
        /// The API's result code.
        "code" => pub code: String,
        /// The API's message.
        "operationMessage" => pub message: String,
        /// Why the search failed.
        "reason" => pub reason: String,
        /// One result per domain.
        "infos" => pub infos: Vec<Availability>,
    }
}

impl DomainBulkSearchResult {
    /// Returns the domains that can be registered.
    pub fn available(&self) -> impl Iterator<Item = &Availability> {
        self.infos.iter().filter(|a| a.is_available())
    }
}

model! {
    /// A nameserver of a domain in the domain list.
    pub struct NameServer {
        /// The host name.
        "name" => pub name: String,
        /// Whether it is a child nameserver of the domain.
        "isChildnameServer" => pub is_child_name_server: bool,
        /// Whether it has a glue record.
        "isGlueRecord" => pub is_glue_record: bool,
        /// Whether it is linked.
        "isLinked" => pub is_linked: bool,
    }
}

model! {
    /// A domain in the domain list. [`DomainsClient::info`] returns more.
    pub struct Domain {
        /// The domain.
        "domainName" => pub domain_name: String,
        /// The internationalised form.
        "domainNameIdn" => pub domain_name_idn: String,
        /// Whether it is an internationalised name.
        "isIdnName" => pub is_idn: bool,
        /// The state on the platform.
        "status" => pub status: DomainStatus,
        /// The registry (EPP) status, for example `"clientTransferProhibited"`.
        "statusCode" => pub status_code: String,
        /// When it was registered.
        "startDate" => pub start_date: Option<DateTime<Utc>>,
        /// When it expires.
        "expirationDate" => pub expiration_date: Option<DateTime<Utc>>,
        /// When it was transferred in.
        "transferDate" => pub transfer_date: Option<DateTime<Utc>>,
        /// When it was last updated.
        "updatedDate" => pub updated_date: Option<DateTime<Utc>>,
        /// The end of the redemption grace period.
        "rgpExpirationDate" => pub rgp_expiration_date: Option<DateTime<Utc>>,
        /// The end of the add grace period.
        "agpExpirationDate" => pub agp_expiration_date: Option<DateTime<Utc>>,
        /// When an operation failed.
        "failureDate" => pub failure_date: Option<DateTime<Utc>>,
        /// When an operation was finalised.
        "finalizationDate" => pub finalization_date: Option<DateTime<Utc>>,
        /// The days until it expires.
        "remainingDay" => pub remaining_days: i64,
        /// Whether the transfer lock is on.
        "lockStatus" => pub locked: bool,
        /// Whether WHOIS privacy is on.
        "privacyProtectionStatus" => pub privacy_protection: bool,
        /// The renewal mode.
        "renewalMode" => pub renewal_mode: String,
        /// The registry object class.
        "objectClass" => pub object_class: String,
        /// The contact handles per role.
        "contacts" => pub contacts: BTreeMap<String, String>,
        /// The nameservers.
        "nameservers" => pub name_servers: Vec<NameServer>,
    }
}

int_enum! {
    /// What a nameserver change does to a DNSSEC-signed domain.
    pub enum DnssecMode {
        /// Refuse the change while DS records exist (the default).
        Block = 0 => "Block",
        /// Keep the DS records.
        KeepDnssec = 1 => "KeepDnssec",
        /// Remove the DS records.
        DisableDnssec = 2 => "DisableDnssec",
    }
}

model! {
    /// An address of a child nameserver.
    pub struct HostIpAddress {
        /// The address.
        "ipAddress" => pub address: String,
        /// `v4` or `v6`.
        "ipVersion" => pub version: String,
    }
}

model! {
    /// A child nameserver (glue record) such as `ns1.example.com`.
    pub struct Host {
        /// The host name.
        "name" => pub name: String,
        /// Its addresses.
        "ipAddresses" => pub ip_addresses: Vec<HostIpAddress>,
    }
}

model! {
    /// The full record of a domain.
    pub struct DomainInfo {
        /// The domain.
        "domainName" => pub domain_name: String,
        /// The internationalised form.
        "domainNameIdn" => pub domain_name_idn: String,
        /// Whether it is an internationalised name.
        "isIdnName" => pub is_idn: bool,
        /// The state on the platform.
        "status" => pub status: DomainStatus,
        /// The registry (EPP) status, for example `"clientTransferProhibited"`.
        "statusCode" => pub status_code: String,
        /// Why the domain is in its status, for example `"RddsVerificationPending"`.
        "statusReason" => pub status_reason: String,
        /// The registry's redemption grace period status.
        "rgpStatues" => pub rgp_status: String,
        /// When it was registered.
        "startDate" => pub start_date: Option<DateTime<Utc>>,
        /// When it expires.
        "expirationDate" => pub expiration_date: Option<DateTime<Utc>>,
        /// When it was transferred in.
        "transferDate" => pub transfer_date: Option<DateTime<Utc>>,
        /// When it was last updated.
        "updatedDate" => pub updated_date: Option<DateTime<Utc>>,
        /// The days until it expires.
        "remainingDay" => pub remaining_days: i64,
        /// The reseller ID.
        "resellerId" => pub reseller_id: String,
        /// The renewal mode.
        "renewalMode" => pub renewal_mode: String,
        /// The transfer authorization (EPP) code.
        "authCode" => pub auth_code: String,
        /// Whether the transfer lock is on.
        "lockStatus" => pub locked: bool,
        /// Whether WHOIS privacy is on.
        "privacyProtectionStatus" => pub privacy_protection: bool,
        /// The registry object ID.
        "objectId" => pub object_id: String,
        /// Whether URL forwarding is set.
        "isForwarded" => pub is_forwarded: bool,
        /// Whether the trustee contact is used.
        "useTrusteeContact" => pub use_trustee_contact: bool,
        /// The nameservers.
        "nameservers" => pub name_servers: Vec<String>,
        /// The child nameservers registered under the domain.
        "hosts" => pub hosts: Vec<Host>,
        /// The contacts, each with its role.
        "contacts" => pub contacts: Vec<Contact>,
        /// The registry attributes.
        "additionalAttributes" => pub additional_attributes: BTreeMap<String, String>,
    }
}

impl DomainInfo {
    /// Returns the contact in a role.
    pub fn contact(&self, role: ContactType) -> Option<&Contact> {
        self.contacts.iter().find(|c| c.contact_type.as_ref() == Some(&role))
    }
}

model! {
    /// The answer to a registration, renewal or transfer.
    ///
    /// The API can answer HTTP 200 with `success == false` (for example when the registry refuses the domain): check
    /// `success` before treating the operation as done.
    pub struct DomainOperationResult {
        /// Whether the registry accepted the operation.
        "success" => pub success: bool,
        /// The API's message; it says why when `success` is `false`.
        "message" => pub message: String,
        /// The domain's ID on the platform.
        "domainId" => pub domain_id: String,
        /// The domain's status after the operation, for example `"Active"` or a waiting status.
        "status" => pub status: String,
        /// The new expiration date.
        "expirationDate" => pub expiration_date: Option<DateTime<Utc>>,
    }
}

model! {
    /// Whether and at what price a domain can be renewed.
    pub struct RenewCheck {
        /// Whether the domain can be renewed.
        "renewabilityStatus" => pub renewable: bool,
        /// Whether a renewal lock is set.
        "renewalLock" => pub renewal_lock: bool,
        /// Why it cannot be renewed.
        "errorMessageKey" => pub error_message_key: String,
        /// Whether this is a premium domain.
        "isPremium" => pub is_premium: bool,
        /// The premium renewal price.
        "premiumPrice" => pub premium_price: Option<f64>,
        /// The currency of the premium price.
        "premiumCurrency" => pub premium_currency: String,
    }
}

model! {
    /// Whether a domain can be transferred in.
    pub struct TransferCheck {
        /// The overall answer.
        "transferAvailabilityStatus" => pub transferable: bool,
        /// The key of the message.
        "messageKey" => pub message_key: String,
        /// The message.
        "message" => pub message: String,
        /// Whether the registry needs an auth code.
        "authCodeIsRequired" => pub auth_code_required: bool,
        /// Whether the auth code is valid.
        "authCodeIsValid" => pub auth_code_valid: bool,
        /// Whether the registrant must approve the transfer.
        "userTransferRequired" => pub user_transfer_required: bool,
        /// Whether the losing registrar still locks the domain.
        "transferLock" => pub transfer_lock: bool,
        /// The registry statuses.
        "domainStatuses" => pub domain_statuses: Vec<String>,
        /// The registry attributes.
        "additionalAttributes" => pub additional_attributes: BTreeMap<String, String>,
        /// Whether this is a premium domain.
        "isPremium" => pub is_premium: bool,
        /// The premium transfer price.
        "premiumPrice" => pub premium_price: Option<f64>,
        /// The currency of the premium price.
        "premiumCurrency" => pub premium_currency: String,
    }
}

model! {
    /// The state of a pending incoming transfer.
    pub struct TransferStatus {
        /// The domain.
        "domainName" => pub domain_name: String,
        /// The registry's transfer status.
        "status" => pub status: String,
        /// When the transfer was requested.
        "requestedDate" => pub requested_date: String,
        /// When it ends.
        "finalDate" => pub final_date: String,
        /// The registrar that requested it.
        "requestedClientId" => pub requested_client_id: String,
        /// The registrar that has to act.
        "actionClientId" => pub action_client_id: String,
    }
}

// ------------------------------------------------------------------------------------------------ contacts of a domain

/// The contact handle of each role.
///
/// ```
/// use domainnameapi::domains::ContactHandles;
///
/// let all = ContactHandles::same("DNA-12345");
/// let tech_only = ContactHandles { tech: "DNA-2".into(), ..Default::default() };
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContactHandles {
    /// The registrant's handle.
    pub registrant: String,
    /// The administrative contact's handle.
    pub admin: String,
    /// The technical contact's handle.
    pub tech: String,
    /// The billing contact's handle.
    pub billing: String,
}

impl ContactHandles {
    /// Uses one handle for all four roles.
    pub fn same(handle: impl Into<String>) -> Self {
        let handle = handle.into();
        ContactHandles { registrant: handle.clone(), admin: handle.clone(), tech: handle.clone(), billing: handle }
    }

    /// The roles that are set, as `{contactType, handle}` objects in the order Registrant, Admin, Tech, Billing.
    pub(crate) fn to_json(&self) -> Vec<Value> {
        [
            (ContactType::Registrant, &self.registrant),
            (ContactType::Admin, &self.admin),
            (ContactType::Tech, &self.tech),
            (ContactType::Billing, &self.billing),
        ]
        .into_iter()
        .filter(|(_, handle)| !handle.trim().is_empty())
        .map(|(role, handle)| {
            JsonBody::new().insert("contactType", role.as_str()).insert("handle", handle.trim()).into_value()
        })
        .collect()
    }

    pub(crate) fn validate(&self, v: &mut Validator, field: &str, all: bool) {
        let n = self.to_json().len();
        if all && n < 4 {
            v.add(
                field,
                "needs a handle for every role (Registrant, Admin, Tech, Billing); ContactHandles::same sets all four",
            );
        } else if n == 0 {
            v.add(field, "needs at least one handle");
        }
    }
}

/// The contact of each role.
///
/// ```
/// use domainnameapi::contacts::Contact;
/// use domainnameapi::domains::DomainContacts;
///
/// # let contact = Contact::default();
/// let all = DomainContacts::same(contact.clone());
/// let admin_only = DomainContacts { admin: Some(contact), ..Default::default() };
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DomainContacts {
    /// The registrant (owner).
    pub registrant: Option<Contact>,
    /// The administrative contact.
    pub admin: Option<Contact>,
    /// The technical contact.
    pub tech: Option<Contact>,
    /// The billing contact.
    pub billing: Option<Contact>,
}

impl DomainContacts {
    /// Uses one contact for all four roles.
    pub fn same(contact: Contact) -> Self {
        DomainContacts {
            registrant: Some(contact.clone()),
            admin: Some(contact.clone()),
            tech: Some(contact.clone()),
            billing: Some(contact),
        }
    }

    fn roles(&self) -> [(ContactType, Option<&Contact>); 4] {
        [
            (ContactType::Registrant, self.registrant.as_ref()),
            (ContactType::Admin, self.admin.as_ref()),
            (ContactType::Tech, self.tech.as_ref()),
            (ContactType::Billing, self.billing.as_ref()),
        ]
    }

    /// The contacts that are set, each with its role.
    pub(crate) fn to_json(&self) -> Vec<Value> {
        self.roles().into_iter().filter_map(|(role, c)| c.map(|c| c.to_json(Some(&role)))).collect()
    }

    pub(crate) fn validate(&self, v: &mut Validator, field: &str, all: bool) {
        for (role, contact) in self.roles() {
            match contact {
                Some(c) => c.validate(v, &format!("{field}.{role}.")),
                None if all => {
                    v.add(format!("{field}.{role}"), "is required (DomainContacts::same sets all four roles)")
                }
                None => {}
            }
        }
        if !all && self.to_json().is_empty() {
            v.add(field, "needs at least one contact");
        }
    }
}

// ------------------------------------------------------------------------------------------------ requests

/// Checks the availability and price of a domain. CLI: `dna domain search`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DomainSearchRequest {
    /// The domain.
    pub domain_name: String,
    /// The currency of the price, for example `"USD"`; empty uses the account's.
    pub currency: String,
}

impl DomainSearchRequest {
    /// Searches this domain.
    pub fn new(domain_name: impl Into<String>) -> Self {
        DomainSearchRequest { domain_name: domain_name.into(), currency: String::new() }
    }

    /// Sets the currency of the price.
    pub fn currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = currency.into();
        self
    }
}

impl Request for DomainSearchRequest {
    type Response = DomainSearchResult;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/domains/search", OperationSafety::Read)
    }

    fn body(&self) -> Option<Value> {
        Some(
            JsonBody::new()
                .insert("domainName", self.domain_name.trim())
                .insert_str("currency", &self.currency.trim().to_ascii_uppercase())
                .into_value(),
        )
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
    }
}

/// Checks several domains at once. CLI: `dna domain bulk-search`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DomainBulkSearchRequest {
    /// The domains.
    pub domain_names: Vec<String>,
    /// The currency of the prices; empty uses the account's.
    pub currency: String,
}

impl DomainBulkSearchRequest {
    /// Searches these domains.
    pub fn new<S: Into<String>>(domain_names: impl IntoIterator<Item = S>) -> Self {
        DomainBulkSearchRequest {
            domain_names: domain_names.into_iter().map(Into::into).collect(),
            currency: String::new(),
        }
    }

    /// Sets the currency of the prices.
    pub fn currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = currency.into();
        self
    }
}

impl Request for DomainBulkSearchRequest {
    type Response = DomainBulkSearchResult;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/domains/bulk-search", OperationSafety::Read)
    }

    fn body(&self) -> Option<Value> {
        let currency = self.currency.trim().to_ascii_uppercase();
        let items: Vec<Value> = trim_all(&self.domain_names)
            .into_iter()
            .map(|name| JsonBody::new().insert("domainName", name).insert_str("currency", &currency).into_value())
            .collect();
        Some(Value::Array(items))
    }

    fn validate(&self, v: &mut Validator) {
        v.required_list("domainNames", trim_all(&self.domain_names).len());
    }
}

/// Lists your domains. CLI: `dna domain list`
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct DomainListRequest {
    /// Keeps domains whose name contains this text.
    pub domain_name: String,
    /// Keeps only domains in these states.
    pub statuses: Vec<DomainStatus>,
    /// The page.
    pub paging: Paging,
}

impl DomainListRequest {
    /// Lists every domain.
    pub fn new() -> Self {
        Self::default()
    }

    /// Keeps domains whose name contains this text.
    pub fn domain_name(mut self, text: impl Into<String>) -> Self {
        self.domain_name = text.into();
        self
    }

    /// Keeps only domains in these states.
    pub fn statuses(mut self, statuses: impl IntoIterator<Item = DomainStatus>) -> Self {
        self.statuses = statuses.into_iter().collect();
        self
    }
}

paged!(DomainListRequest => Domain, sortable);

impl Request for DomainListRequest {
    type Response = Page<Domain>;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/domains")
    }

    fn query(&self) -> Query {
        let query = Query::new()
            .set_str("DomainName", &self.domain_name)
            .set_joined("StatusIds", self.statuses.iter().map(|s| s.code()));
        self.paging.apply(query)
    }

    fn validate(&self, v: &mut Validator) {
        self.paging.validate(v);
    }
}

macro_rules! domain_only_request {
    ($(#[$meta:meta])* $name:ident, $response:ty, $endpoint:expr, $style:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        #[non_exhaustive]
        pub struct $name {
            /// The domain.
            pub domain_name: String,
        }

        impl $name {
            /// Creates the request for this domain.
            pub fn new(domain_name: impl Into<String>) -> Self {
                $name { domain_name: domain_name.into() }
            }
        }

        impl Request for $name {
            type Response = $response;

            fn endpoint(&self) -> Endpoint {
                $endpoint
            }

            domain_only_request!(@$style);

            fn validate(&self, v: &mut Validator) {
                v.required("domainName", &self.domain_name);
            }
        }
    };
    (@body) => {
        fn body(&self) -> Option<Value> {
            Some(domain_body(&self.domain_name))
        }
    };
    (@query) => {
        fn query(&self) -> Query {
            Query::new().set_str("DomainName", &self.domain_name)
        }
    };
    (@query_lower) => {
        fn query(&self) -> Query {
            Query::new().set_str("domainName", &self.domain_name)
        }
    };
}
pub(crate) use domain_only_request;

domain_only_request!(
    /// Reads the full record of a domain. CLI: `dna domain info`
    DomainInfoRequest,
    DomainInfo,
    Endpoint::get("/api/v1/domains/info"),
    query
);

domain_only_request!(
    /// Reads the state of a pending incoming transfer. CLI: `dna domain transfer-status`
    DomainTransferStatusRequest,
    TransferStatus,
    Endpoint::post("/api/v1/domains/transfers/query", OperationSafety::Read),
    body
);

domain_only_request!(
    /// Restores a deleted domain during the registry's redemption period ([`DomainStatus::PendingDelete`]). It
    /// charges your balance; restore fees are usually high. CLI: `dna domain restore`
    DomainRestoreRequest,
    Empty,
    Endpoint::post("/api/v1/domains/restore", OperationSafety::Billable),
    body
);

/// Registers a domain with existing contact handles. It charges your balance. CLI: `dna domain register`
///
/// ```
/// use domainnameapi::domains::{ContactHandles, DomainRegisterRequest};
///
/// let request = DomainRegisterRequest::new("example.com", ContactHandles::same("DNA-12345"), ["ns1.example.net", "ns2.example.net"])
///     .years(2);
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DomainRegisterRequest {
    /// The domain.
    pub domain_name: String,
    /// The contact handles of the four roles (see [`ContactHandles::same`] and [`ContactsClient::create`](crate::contacts::ContactsClient::create)).
    pub handles: ContactHandles,
    /// The nameserver host names; most registries want two or more.
    pub name_servers: Vec<String>,
    /// The registration period, 1 to 10 years; `None` lets the API use 1.
    pub years: Option<u32>,
    /// The registry attributes some TLDs require; see [`Tld::attributes`](crate::products::Tld::attributes).
    pub tld_attributes: BTreeMap<String, String>,
}

impl DomainRegisterRequest {
    /// Registers `domain_name` with `handles` and `name_servers`.
    pub fn new<S: Into<String>>(
        domain_name: impl Into<String>,
        handles: ContactHandles,
        name_servers: impl IntoIterator<Item = S>,
    ) -> Self {
        DomainRegisterRequest {
            domain_name: domain_name.into(),
            handles,
            name_servers: name_servers.into_iter().map(Into::into).collect(),
            years: None,
            tld_attributes: BTreeMap::new(),
        }
    }

    /// Sets the registration period, 1 to 10 years.
    pub fn years(mut self, years: u32) -> Self {
        self.years = Some(years);
        self
    }

    /// Adds a registry attribute.
    pub fn tld_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tld_attributes.insert(key.into(), value.into());
        self
    }
}

impl Request for DomainRegisterRequest {
    type Response = DomainOperationResult;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/domains/register", OperationSafety::Billable)
    }

    fn body(&self) -> Option<Value> {
        Some(
            JsonBody::new()
                .insert("domainName", self.domain_name.trim())
                .insert_opt("period", self.years)
                .insert("nameServers", trim_all(&self.name_servers))
                .insert("handles", self.handles.to_json())
                .insert_opt("tldAttributes", (!self.tld_attributes.is_empty()).then_some(&self.tld_attributes))
                .into_value(),
        )
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        v.between("years", self.years, 1, 10);
        v.required_list("nameServers", trim_all(&self.name_servers).len());
        self.handles.validate(v, "handles", true);
    }
}

/// Registers a domain with the contact details given inline. It charges your balance.
/// CLI: `dna domain register-with-contacts`
///
/// ```
/// use domainnameapi::contacts::Contact;
/// use domainnameapi::domains::{DomainContacts, DomainRegisterWithContactsRequest};
///
/// # let contact = Contact::default();
/// let request = DomainRegisterWithContactsRequest::new("example.com.tr", DomainContacts::same(contact), ["tr.apiname.com", "eu.apiname.com"])
///     .years(1)
///     .tld_attribute("<key>", "<value>");
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DomainRegisterWithContactsRequest {
    /// The domain.
    pub domain_name: String,
    /// The contacts; all four roles unless `use_trustee_contact` is set.
    pub contacts: DomainContacts,
    /// The nameserver host names.
    pub name_servers: Vec<String>,
    /// The registration period, 1 to 10 years; `None` lets the API use 1.
    pub years: Option<u32>,
    /// The registry attributes some TLDs require.
    pub tld_attributes: BTreeMap<String, String>,
    /// Uses the registrar's trustee contact, for TLDs with [`Tld::supports_trustee`](crate::products::Tld::supports_trustee).
    pub use_trustee_contact: bool,
}

impl DomainRegisterWithContactsRequest {
    /// Registers `domain_name` with `contacts` and `name_servers`.
    pub fn new<S: Into<String>>(
        domain_name: impl Into<String>,
        contacts: DomainContacts,
        name_servers: impl IntoIterator<Item = S>,
    ) -> Self {
        DomainRegisterWithContactsRequest {
            domain_name: domain_name.into(),
            contacts,
            name_servers: name_servers.into_iter().map(Into::into).collect(),
            years: None,
            tld_attributes: BTreeMap::new(),
            use_trustee_contact: false,
        }
    }

    /// Sets the registration period, 1 to 10 years.
    pub fn years(mut self, years: u32) -> Self {
        self.years = Some(years);
        self
    }

    /// Adds a registry attribute.
    pub fn tld_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tld_attributes.insert(key.into(), value.into());
        self
    }

    /// Uses the registrar's trustee contact.
    pub fn use_trustee_contact(mut self, use_trustee: bool) -> Self {
        self.use_trustee_contact = use_trustee;
        self
    }
}

impl Request for DomainRegisterWithContactsRequest {
    type Response = DomainOperationResult;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/domains/register-with-contacts", OperationSafety::Billable)
    }

    fn body(&self) -> Option<Value> {
        Some(
            JsonBody::new()
                .insert("domainName", self.domain_name.trim())
                .insert_opt("period", self.years)
                .insert("nameServers", trim_all(&self.name_servers))
                .insert("contacts", self.contacts.to_json())
                .insert_opt("tldAttributes", (!self.tld_attributes.is_empty()).then_some(&self.tld_attributes))
                .insert_opt("useTrusteeContact", self.use_trustee_contact.then_some(true))
                .into_value(),
        )
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        v.between("years", self.years, 1, 10);
        v.required_list("nameServers", trim_all(&self.name_servers).len());
        if !self.use_trustee_contact || !self.contacts.to_json().is_empty() {
            self.contacts.validate(v, "contacts", !self.use_trustee_contact);
        }
    }
}

/// Renews a domain. It charges your balance. CLI: `dna domain renew`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DomainRenewRequest {
    /// The domain.
    pub domain_name: String,
    /// The years to add, 1 to 10.
    pub years: u32,
}

impl DomainRenewRequest {
    /// Renews `domain_name` for `years`.
    pub fn new(domain_name: impl Into<String>, years: u32) -> Self {
        DomainRenewRequest { domain_name: domain_name.into(), years }
    }
}

impl Request for DomainRenewRequest {
    type Response = DomainOperationResult;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/domains/renew", OperationSafety::Billable)
    }

    fn body(&self) -> Option<Value> {
        Some(JsonBody::new().insert("domainName", self.domain_name.trim()).insert("period", self.years).into_value())
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        v.between("years", Some(self.years), 1, 10);
    }
}

/// Checks whether and at what price a domain can be renewed. CLI: `dna domain renew-check`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DomainRenewCheckRequest {
    /// The domain.
    pub domain_name: String,
    /// The years to add, 1 to 10; `None` lets the API choose.
    pub years: Option<u32>,
}

impl DomainRenewCheckRequest {
    /// Checks `domain_name`.
    pub fn new(domain_name: impl Into<String>) -> Self {
        DomainRenewCheckRequest { domain_name: domain_name.into(), years: None }
    }

    /// Sets the years to add.
    pub fn years(mut self, years: u32) -> Self {
        self.years = Some(years);
        self
    }
}

impl Request for DomainRenewCheckRequest {
    type Response = RenewCheck;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/domains/renew/check", OperationSafety::Read)
    }

    fn body(&self) -> Option<Value> {
        Some(
            JsonBody::new().insert("domainName", self.domain_name.trim()).insert_opt("period", self.years).into_value(),
        )
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        v.between("years", self.years, 1, 10);
    }
}

/// Starts an incoming transfer. It charges your balance. CLI: `dna domain transfer`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DomainTransferRequest {
    /// The domain.
    pub domain_name: String,
    /// The transfer authorization (EPP) code from the current registrar. It is sent as given: surrounding spaces can
    /// be part of it.
    pub auth_code: String,
    /// The years added on transfer, 1 to 10; `None` lets the registry decide (usually 1).
    pub years: Option<u32>,
}

impl DomainTransferRequest {
    /// Transfers `domain_name` with `auth_code`.
    pub fn new(domain_name: impl Into<String>, auth_code: impl Into<String>) -> Self {
        DomainTransferRequest { domain_name: domain_name.into(), auth_code: auth_code.into(), years: None }
    }

    /// Sets the years added on transfer.
    pub fn years(mut self, years: u32) -> Self {
        self.years = Some(years);
        self
    }
}

impl Request for DomainTransferRequest {
    type Response = DomainOperationResult;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/domains/transfer", OperationSafety::Billable)
    }

    fn body(&self) -> Option<Value> {
        let auth_code = (!self.auth_code.is_empty()).then_some(&self.auth_code);
        Some(
            JsonBody::new()
                .insert("domainName", self.domain_name.trim())
                .insert_opt("authCode", auth_code)
                .insert_opt("period", self.years)
                .into_value(),
        )
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        v.between("years", self.years, 1, 10);
    }
}

/// Checks whether a domain can be transferred in with an auth code. CLI: `dna domain transfer-check`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DomainTransferCheckRequest {
    /// The domain.
    pub domain_name: String,
    /// The transfer authorization (EPP) code.
    pub auth_code: String,
}

impl DomainTransferCheckRequest {
    /// Checks `domain_name` with `auth_code`.
    pub fn new(domain_name: impl Into<String>, auth_code: impl Into<String>) -> Self {
        DomainTransferCheckRequest { domain_name: domain_name.into(), auth_code: auth_code.into() }
    }
}

impl Request for DomainTransferCheckRequest {
    type Response = TransferCheck;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/domains/transfers/check", OperationSafety::Read)
    }

    fn body(&self) -> Option<Value> {
        Some(
            JsonBody::new()
                .insert("domainName", self.domain_name.trim())
                .insert("authCode", &self.auth_code)
                .into_value(),
        )
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        v.required("authCode", &self.auth_code);
    }
}

// ------------------------------------------------------------------------------------------------ client

/// Domains. Get it with [`Client::domains`].
#[derive(Debug, Clone, Copy)]
pub struct DomainsClient<'a> {
    client: &'a Client,
}

impl<'a> DomainsClient<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        DomainsClient { client }
    }

    /// URL forwarding. CLI: `dna domain forward`.
    pub fn forwarding(&self) -> ForwardingClient<'a> {
        ForwardingClient::new(self.client)
    }

    /// Child nameservers (glue records). CLI: `dna domain host`.
    pub fn child_name_servers(&self) -> ChildNameServersClient<'a> {
        ChildNameServersClient::new(self.client)
    }

    /// Registry documents. CLI: `dna domain documents`.
    pub fn documents(&self) -> DocumentsClient<'a> {
        DocumentsClient::new(self.client)
    }

    /// Checks whether a domain is available and what it costs, in the account's currency. The gateway allows 2
    /// searches per second; the client paces them. Check [`DomainSearchResult::success`] and
    /// [`Availability::is_available`].
    pub async fn search(&self, domain_name: &str) -> Result<DomainSearchResult, Error> {
        self.client.send(&DomainSearchRequest::new(domain_name)).await
    }

    /// Like [`search`](DomainsClient::search), with all options (the currency).
    pub async fn search_with(&self, request: DomainSearchRequest) -> Result<DomainSearchResult, Error> {
        self.client.send(&request).await
    }

    /// Checks several domains at once. Each result has its own status.
    pub async fn bulk_search<S: Into<String>>(
        &self,
        domain_names: impl IntoIterator<Item = S>,
    ) -> Result<DomainBulkSearchResult, Error> {
        self.client.send(&DomainBulkSearchRequest::new(domain_names)).await
    }

    /// Like [`bulk_search`](DomainsClient::bulk_search), with all options (the currency).
    pub async fn bulk_search_with(&self, request: DomainBulkSearchRequest) -> Result<DomainBulkSearchResult, Error> {
        self.client.send(&request).await
    }

    /// Returns one page of your domains.
    pub async fn list(&self, request: DomainListRequest) -> Result<Page<Domain>, Error> {
        self.client.send(&request).await
    }

    /// Returns every matching domain, fetching pages as the stream is read.
    pub fn list_all(&self, request: DomainListRequest) -> ItemStream<Domain> {
        self.client.list_all(request)
    }

    /// Returns the full record of a domain: dates, status, lock and privacy, nameservers, child hosts, contacts and
    /// the auth code. It is not rate limited.
    pub async fn info(&self, domain_name: &str) -> Result<DomainInfo, Error> {
        self.client.send(&DomainInfoRequest::new(domain_name)).await
    }

    /// Registers a domain with existing contact handles and charges your balance. Check availability and price with
    /// [`search`](DomainsClient::search) first, and [`DomainOperationResult::success`] afterwards.
    pub async fn register(&self, request: DomainRegisterRequest) -> Result<DomainOperationResult, Error> {
        self.client.send(&request).await
    }

    /// Registers a domain with inline contact details and charges your balance. See
    /// [`register`](DomainsClient::register).
    pub async fn register_with_contacts(
        &self,
        request: DomainRegisterWithContactsRequest,
    ) -> Result<DomainOperationResult, Error> {
        self.client.send(&request).await
    }

    /// Adds `years` (1 to 10) to a domain and charges your balance. Use [`renew_check`](DomainsClient::renew_check)
    /// first to see whether it is renewable and at what price.
    pub async fn renew(&self, domain_name: &str, years: u32) -> Result<DomainOperationResult, Error> {
        self.client.send(&DomainRenewRequest::new(domain_name, years)).await
    }

    /// Returns whether a domain can be renewed for `years` (`None` lets the API choose) and the premium price, if
    /// any.
    pub async fn renew_check(&self, domain_name: &str, years: Option<u32>) -> Result<RenewCheck, Error> {
        let mut request = DomainRenewCheckRequest::new(domain_name);
        request.years = years;
        self.client.send(&request).await
    }

    /// Starts an incoming transfer and charges your balance. Check it first with
    /// [`transfer_check`](DomainsClient::transfer_check); follow it with
    /// [`transfer_status`](DomainsClient::transfer_status).
    pub async fn transfer(&self, request: DomainTransferRequest) -> Result<DomainOperationResult, Error> {
        self.client.send(&request).await
    }

    /// Returns whether a domain can be transferred in with `auth_code`: lock state, auth code validity, registry
    /// statuses and premium price.
    pub async fn transfer_check(&self, domain_name: &str, auth_code: &str) -> Result<TransferCheck, Error> {
        self.client.send(&DomainTransferCheckRequest::new(domain_name, auth_code)).await
    }

    /// Returns the state of a pending incoming transfer.
    pub async fn transfer_status(&self, domain_name: &str) -> Result<TransferStatus, Error> {
        self.client.send(&DomainTransferStatusRequest::new(domain_name)).await
    }

    /// Brings back a deleted domain during the redemption period and charges your balance.
    pub async fn restore(&self, domain_name: &str) -> Result<(), Error> {
        self.client.send(&DomainRestoreRequest::new(domain_name)).await.map(|_| ())
    }

    /// Turns the transfer lock on.
    pub async fn lock(&self, domain_name: &str) -> Result<(), Error> {
        self.client.send(&DomainLockRequest::new(domain_name)).await.map(|_| ())
    }

    /// Turns the transfer lock off, which a transfer to another registrar needs.
    pub async fn unlock(&self, domain_name: &str) -> Result<(), Error> {
        self.client.send(&DomainUnlockRequest::new(domain_name)).await.map(|_| ())
    }

    /// Turns WHOIS privacy protection on or off. TLDs without privacy answer with
    /// [`codes::PRIVACY_NOT_SUPPORTED`](crate::codes::PRIVACY_NOT_SUPPORTED).
    pub async fn set_privacy(&self, domain_name: &str, enabled: bool) -> Result<(), Error> {
        self.client.send(&DomainPrivacyRequest::new(domain_name, enabled)).await.map(|_| ())
    }

    /// Asks the registry for a new transfer auth code and returns it. To read the current code, use
    /// [`info`](DomainsClient::info).
    pub async fn update_auth_code(&self, domain_name: &str) -> Result<AuthCode, Error> {
        self.client.send(&DomainUpdateAuthCodeRequest::new(domain_name)).await
    }

    /// Replaces the nameservers of a domain. For a DNSSEC-signed domain, set
    /// [`DomainNameServersSetRequest::dnssec_mode`].
    pub async fn set_name_servers(&self, request: DomainNameServersSetRequest) -> Result<(), Error> {
        self.client.send(&request).await.map(|_| ())
    }

    /// Replaces contacts of a domain with inline details; roles left `None` stay as they are. Changing the
    /// registrant may start an owner-change process, with documents on some TLDs.
    pub async fn update_contacts(&self, domain_name: &str, contacts: DomainContacts) -> Result<(), Error> {
        self.client.send(&DomainContactsUpdateRequest::new(domain_name, contacts)).await.map(|_| ())
    }

    /// Points contact roles of a domain at existing handles; roles left empty stay as they are.
    pub async fn set_contact_handles(&self, domain_name: &str, handles: ContactHandles) -> Result<(), Error> {
        self.client.send(&DomainContactsSetHandlesRequest::new(domain_name, handles)).await.map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contacts::tests::sample;
    use serde_json::json;

    #[test]
    fn register_body() {
        let request =
            DomainRegisterRequest::new("example.com", ContactHandles::same("DNA-1"), [" ns1.example.net ", ""])
                .years(2)
                .tld_attribute("a", "b");
        assert_eq!(
            request.body().unwrap(),
            json!({
                "domainName": "example.com", "period": 2, "nameServers": ["ns1.example.net"],
                "handles": [
                    {"contactType": "Registrant", "handle": "DNA-1"}, {"contactType": "Admin", "handle": "DNA-1"},
                    {"contactType": "Tech", "handle": "DNA-1"}, {"contactType": "Billing", "handle": "DNA-1"}
                ],
                "tldAttributes": {"a": "b"}
            })
        );
    }

    #[test]
    fn register_needs_all_handles() {
        let request = DomainRegisterRequest::new(
            "example.com",
            ContactHandles { tech: "DNA-1".into(), ..Default::default() },
            ["ns1"],
        )
        .years(11);
        let mut v = Validator::new();
        request.validate(&mut v);
        let fields: Vec<&str> = v.errors().iter().map(|e| e.field.as_str()).collect();
        assert_eq!(fields, ["years", "handles"]);
    }

    #[test]
    fn register_with_contacts_sets_roles() {
        let body =
            DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(sample()), ["ns1.example.net"])
                .body()
                .unwrap();
        let roles: Vec<&str> =
            body["contacts"].as_array().unwrap().iter().map(|c| c["contactType"].as_str().unwrap()).collect();
        assert_eq!(roles, ["Registrant", "Admin", "Tech", "Billing"]);
        assert!(body.get("useTrusteeContact").is_none());
        assert!(body.get("period").is_none());

        let mut v = Validator::new();
        DomainRegisterWithContactsRequest::new("example.com", DomainContacts::default(), ["ns1"]).validate(&mut v);
        assert_eq!(v.errors().len(), 4);

        let mut v = Validator::new();
        DomainRegisterWithContactsRequest::new("example.com", DomainContacts::default(), ["ns1"])
            .use_trustee_contact(true)
            .validate(&mut v);
        assert!(v.is_valid());
    }

    #[test]
    fn transfer_keeps_the_auth_code_as_given() {
        let body = DomainTransferRequest::new(" example.com ", " Xy7!kP2q").years(1).body().unwrap();
        assert_eq!(body, json!({"domainName": "example.com", "authCode": " Xy7!kP2q", "period": 1}));
        assert!(DomainTransferRequest::new("example.com", "").body().unwrap().get("authCode").is_none());
    }

    #[test]
    fn list_query() {
        let q = DomainListRequest::new()
            .domain_name("example")
            .statuses([DomainStatus::Active, DomainStatus::PendingDelete])
            .skip(10)
            .limit(50)
            .sort("expirationDate desc");
        let pairs: Vec<String> = q.query().pairs().iter().map(|(k, v)| format!("{k}={v}")).collect();
        assert_eq!(
            pairs,
            ["DomainName=example", "StatusIds=0,7", "SkipCount=10", "MaxResultCount=50", "Sorting=expirationDate desc"]
        );
    }

    #[test]
    fn domain_info_reads_contacts_and_status_names() {
        let info: DomainInfo = serde_json::from_value(json!({
            "domainName": "example.com", "status": "Active", "lockStatus": "true", "remainingDay": "30",
            "expirationDate": "2027-01-01T00:00:00", "nameservers": ["ns1.example.net"],
            "contacts": [{"contactType": "Registrant", "eMail": "a@example.com"}, {"contactType": "Tech", "eMail": "t@example.com"}],
            "additionalAttributes": []
        }))
        .unwrap();
        assert_eq!(info.status, DomainStatus::Active);
        assert!(info.locked);
        assert_eq!(info.remaining_days, 30);
        assert_eq!(info.contact(ContactType::Tech).unwrap().email, "t@example.com");
        assert!(info.contact(ContactType::Billing).is_none());
        assert!(info.additional_attributes.is_empty());
    }

    #[test]
    fn search_results() {
        let result: DomainBulkSearchResult = serde_json::from_value(json!({
            "success": true,
            "infos": [{"domainName": "a.com", "status": "available"}, {"domainName": "b.com", "status": "NOTAVAILABLE"}]
        }))
        .unwrap();
        assert_eq!(result.available().map(|a| a.domain_name.as_str()).collect::<Vec<_>>(), ["a.com"]);
        let body = DomainBulkSearchRequest::new(["a.com", " "]).currency("usd").body().unwrap();
        assert_eq!(body, json!([{"domainName": "a.com", "currency": "USD"}]));
    }
}
