//! SSL certificates: buy, configure, issue, list, renew, reissue and revoke. CLI: `dna ssl`.
//!
//! The flow: find a package with [`ProductsClient::ssls`](crate::products::ProductsClient::ssls), generate a CSR (or
//! bring your own), [`initialize`](SslClient::initialize) the purchase (this charges the balance; keep the
//! certificate ID it returns), [`configure`](SslClient::configure) it within 15 days, then poll
//! [`status`](SslClient::status) until the certificate is issued and download it with
//! [`content`](SslClient::content).
//!
//! A domain can have several certificates, so every operation on an existing certificate takes its **certificate
//! ID** (a GUID), never the domain name. `initialize` and `renew` return the ID; [`list`](SslClient::list) finds it by
//! domain name.
//!
//! To renew, call [`renew`](SslClient::renew) (this charges the balance) and configure the new certificate it
//! returns. Do not call `initialize` again for that: it is an unrelated purchase at the purchase price, and the old
//! certificate would never become "Renewed".

use crate::de::{model, str_enum};
use crate::request::{paged, trim_all};
use crate::{Client, Endpoint, Error, ItemStream, JsonBody, OperationSafety, Page, Paging, Query, Request, Validator};
use chrono::{DateTime, Utc};
use serde_json::Value;

str_enum! {
    /// How domain control is validated.
    pub enum AuthenticationType {
        /// An approval e-mail to the approver address.
        Email => "Email",
        /// A DNS CNAME record.
        Cname => "Cname",
        /// An HTTP file on the site.
        Http => "Http",
        /// File-based validation.
        File => "File",
    }
}

str_enum! {
    /// The web server the certificate is for.
    pub enum WebServerType {
        /// Apache, nginx and most others.
        Apache => "Apache",
        /// Microsoft IIS.
        Iis => "IIS",
        /// AOL.
        Aol => "AOL",
        /// C2Net.
        C2Net => "C2Net",
    }
}

model! {
    @open
    /// The contact of a certificate order. Build one with a struct literal.
    pub struct SslContact {
        /// The title; optional.
        "title" => pub title: String,
        /// The first name (at most 70 characters).
        "firstName" => pub first_name: String,
        /// The last name (at most 70 characters).
        "lastName" => pub last_name: String,
        /// The company name (at most 150 characters).
        "companyName" => pub company_name: String,
        /// The e-mail address.
        "eMail" => pub email: String,
        /// The street address (at most 200 characters).
        "address" => pub address: String,
        /// The city (at most 20 characters).
        "city" => pub city: String,
        /// The state; optional (at most 20 characters).
        "state" => pub state: String,
        /// The postal code (at most 10 characters).
        "postalCode" => pub postal_code: String,
        /// The ISO 3166-1 alpha-2 country code.
        "country" => pub country: String,
        /// The calling code without `+`.
        "phoneCountryCode" => pub phone_country_code: String,
        /// The number without the calling code (at most 15 digits).
        "phone" => pub phone: String,
        /// The fax calling code; optional.
        "faxCountryCode" => pub fax_country_code: String,
        /// The fax number; optional.
        "fax" => pub fax: String,
        /// The contact's role, set by the API.
        "type" => pub contact_type: String,
    }
}

fn digits(text: &str) -> String {
    text.chars().filter(char::is_ascii_digit).collect()
}

impl SslContact {
    fn to_json(&self) -> Value {
        let t = |s: &str| s.trim().to_string();
        JsonBody::new()
            .insert_str("title", &self.title)
            .insert("firstName", t(&self.first_name))
            .insert("lastName", t(&self.last_name))
            .insert("companyName", t(&self.company_name))
            .insert("eMail", t(&self.email))
            .insert("address", t(&self.address))
            .insert("city", t(&self.city))
            .insert_str("state", &self.state)
            .insert("postalCode", t(&self.postal_code))
            .insert("country", self.country.trim().to_ascii_uppercase())
            .insert("phoneCountryCode", digits(&self.phone_country_code))
            .insert("phone", digits(&self.phone))
            .insert_str("faxCountryCode", &digits(&self.fax_country_code))
            .insert_str("fax", &digits(&self.fax))
            .into_value()
    }

    fn validate(&self, v: &mut Validator) {
        for (name, value) in [
            ("firstName", &self.first_name),
            ("lastName", &self.last_name),
            ("companyName", &self.company_name),
            ("email", &self.email),
            ("address", &self.address),
            ("city", &self.city),
            ("postalCode", &self.postal_code),
            ("country", &self.country),
        ] {
            v.required(format!("contact.{name}"), value);
        }
        v.required("contact.phoneCountryCode", &digits(&self.phone_country_code));
        v.required("contact.phone", &digits(&self.phone));
        v.max_len("contact.firstName", &self.first_name, 70);
        v.max_len("contact.lastName", &self.last_name, 70);
        v.max_len("contact.companyName", &self.company_name, 150);
        v.max_len("contact.address", &self.address, 200);
        v.max_len("contact.city", &self.city, 20);
        v.max_len("contact.state", &self.state, 20);
        v.max_len("contact.postalCode", &self.postal_code, 10);
        v.check(digits(&self.phone).len() <= 15, "contact.phone", "must be at most 15 digits");
    }
}

model! {
    /// A certificate signing request and its private key.
    pub struct Csr {
        /// The CSR, PEM-encoded.
        "csr" => pub csr: String,
        /// The private key, PEM-encoded. It is needed to install the certificate: store it safely.
        "privateKey" => pub private_key: String,
    }
}

model! {
    /// The decoded subject of a CSR.
    pub struct CsrInfo {
        /// The common name.
        "domainName" => pub domain_name: String,
        /// The country.
        "country" => pub country: String,
        /// The state.
        "state" => pub state: String,
        /// The locality.
        "locality" => pub locality: String,
        /// The organization.
        "organization" => pub organization: String,
        /// The organizational unit.
        "organizationUnit" => pub organization_unit: String,
        /// The alternative names.
        "alternativeNames" => pub alternative_names: String,
        /// Whether it is a wildcard CSR.
        "isWildcard" => pub is_wildcard: bool,
        /// Whether it has alternative names.
        "isSan" => pub is_san: bool,
    }
}

model! {
    /// The answer to a certificate operation (initialize, renew, configure, reissue, revoke).
    ///
    /// The API can answer HTTP 200 with `success == false`: check it before treating the operation as done.
    pub struct SslOperationResult {
        /// Whether the operation succeeded.
        "success" => pub success: bool,
        /// The API's message; it says why when `success` is `false`.
        "message" => pub message: String,
        /// The certificate ID. Store it: every later call for the certificate takes it. After a renewal it is the
        /// ID of the new (renewal) certificate, not of the one that was renewed.
        "certificateId" => pub certificate_id: String,
        /// The certificate's state.
        "status" => pub status: String,
        /// How to complete domain control validation (for example the CNAME record or file to create).
        "dcvDetails" => pub dcv_details: String,
        /// The CSR.
        "csr" => pub csr: String,
        /// The private key.
        "privateKey" => pub private_key: String,
    }
}

model! {
    /// The current state of a certificate.
    pub struct CertificateStatus {
        /// The certificate ID.
        "certificateId" => pub certificate_id: String,
        /// The domain.
        "domainName" => pub domain_name: String,
        /// The state.
        "status" => pub status: String,
        /// The state in words.
        "statusMessage" => pub status_message: String,
        /// Whether it was configured.
        "isConfigured" => pub is_configured: bool,
        /// Whether it was issued.
        "isIssued" => pub is_issued: bool,
        /// When the issued certificate expires.
        "certificateExpiresAt" => pub expires_at: Option<DateTime<Utc>>,
    }
}

model! {
    /// The paid term of a certificate.
    pub struct Subscription {
        /// When it was bought.
        "purchasedAt" => pub purchased_at: Option<DateTime<Utc>>,
        /// When the term ends. A renewal is allowed from 30 days before.
        "expiresAt" => pub expires_at: Option<DateTime<Utc>>,
    }
}

model! {
    /// How a certificate was configured.
    pub struct CertificateConfiguration {
        /// When it was configured.
        "configuredAt" => pub configured_at: Option<DateTime<Utc>>,
        /// The configuration link.
        "configurationLink" => pub configuration_link: String,
        /// The PIN.
        "pin" => pub pin: String,
        /// The web server type.
        "webServer" => pub web_server: String,
        /// The validation method.
        "authenticationType" => pub authentication_type: String,
        /// The additional names.
        "sanList" => pub san_list: String,
        /// The wildcard names.
        "wildcardList" => pub wildcard_list: String,
        /// The approver address.
        "approverMail" => pub approver_mail: String,
        /// The CSR.
        "csr" => pub csr: String,
        /// How to complete domain control validation.
        "dcvDetails" => pub dcv_details: String,
    }
}

model! {
    /// The validity of an issued certificate.
    pub struct IssuedCertificate {
        /// When it was issued.
        "issuedAt" => pub issued_at: Option<DateTime<Utc>>,
        /// When it expires.
        "validUntil" => pub valid_until: Option<DateTime<Utc>>,
    }
}

model! {
    /// The full record of a certificate.
    pub struct Certificate {
        /// The certificate ID.
        "certificateId" => pub certificate_id: String,
        /// The domain.
        "domainName" => pub domain_name: String,
        /// The state.
        "status" => pub status: String,
        /// Whether it was configured.
        "isConfigured" => pub is_configured: bool,
        /// Whether it was issued.
        "isIssued" => pub is_issued: bool,
        /// The package name.
        "packageName" => pub package_name: String,
        /// The package code.
        "packageCode" => pub package_code: String,
        /// The paid term.
        "subscription" => pub subscription: Subscription,
        /// How it was configured.
        "configuration" => pub configuration: CertificateConfiguration,
        /// The validity of the issued certificate.
        "certificate" => pub issued: IssuedCertificate,
        /// The contacts.
        "contacts" => pub contacts: Vec<SslContact>,
    }
}

model! {
    /// A certificate in the certificate list. [`SslClient::get`] returns more.
    pub struct CertificateListItem {
        /// The certificate ID the other operations take.
        "certificateId" => pub certificate_id: String,
        /// The domain.
        "domainName" => pub domain_name: String,
        /// The package code.
        "packageCode" => pub package_code: String,
        /// The package name.
        "packageName" => pub package_name: String,
        /// The state: `"Active"`, `"WaitingForInstall"` (bought, not configured yet), `"WaitingForValidation"`,
        /// `"Cancelled"`, `"Renewed"` (superseded by its renewal), ...
        "status" => pub status: String,
        /// Why, mainly for `"Cancelled"`: `"None"`, `"ConfigurationWindowExpired"`, `"ResellerRevoked"`, ...
        "statusReason" => pub status_reason: String,
        /// Whether it was configured.
        "isConfigured" => pub is_configured: bool,
        /// Whether it was issued.
        "isIssued" => pub is_issued: bool,
        /// When it was bought.
        "purchasedAt" => pub purchased_at: Option<DateTime<Utc>>,
        /// When the paid term ends.
        "subscriptionExpiresAt" => pub subscription_expires_at: Option<DateTime<Utc>>,
        /// When the issued certificate expires; `None` until it is issued.
        "certificateExpiresAt" => pub certificate_expires_at: Option<DateTime<Utc>>,
        /// On a renewal: the ID of the certificate it renews.
        "renewedFromCertificateId" => pub renewed_from_certificate_id: String,
    }
}

model! {
    /// An issued certificate and its chain, PEM-encoded.
    pub struct CertificateContent {
        /// The certificate ID.
        "certificateId" => pub certificate_id: String,
        /// The certificate.
        "certificate" => pub certificate: String,
        /// The first intermediate certificate.
        "intermediateCertificate1" => pub intermediate1: String,
        /// The second intermediate certificate.
        "intermediateCertificate2" => pub intermediate2: String,
        /// The root certificate.
        "rootCertificate" => pub root: String,
    }
}

impl CertificateContent {
    /// Returns the certificate followed by its intermediates, the bundle most web servers want.
    pub fn full_chain(&self) -> String {
        let parts: Vec<&str> = [&self.certificate, &self.intermediate1, &self.intermediate2]
            .into_iter()
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect();
        format!("{}\n", parts.join("\n"))
    }
}

model! {
    /// The answer to [`SslApproversRequest`].
    pub struct ApproverList {
        /// The allowed approver addresses.
        "mails" => pub mails: Vec<String>,
    }
}

// ------------------------------------------------------------------------------------------------ requests

/// Returns whether the text is a GUID: 8-4-4-4-12 hexadecimal digits.
fn is_guid(text: &str) -> bool {
    let parts: Vec<&str> = text.split('-').collect();
    parts.len() == 5
        && parts.iter().zip([8, 4, 4, 4, 12]).all(|(p, n)| p.len() == n && p.chars().all(|c| c.is_ascii_hexdigit()))
}

/// Checks a certificate ID. Certificates used to be addressed by domain name, and both are text, so a domain name
/// passed by older code is caught here instead of reaching the API.
fn validate_certificate_id(v: &mut Validator, id: &str) {
    let id = id.trim();
    if id.is_empty() {
        v.add(
            "certificateId",
            "is required: pass the certificate ID returned by initialize or renew, or look it up with ssl().list(...)",
        );
    } else if !is_guid(id) {
        v.add(
            "certificateId",
            "must be a certificate ID (GUID). Certificates are no longer addressed by domain name: use the ID \
             returned by initialize or renew, or look it up with ssl().list(SslListRequest::new().domain_name(...))",
        );
    }
}

fn certificate_body(id: &str) -> JsonBody {
    JsonBody::new().insert("certificateId", id.trim())
}

/// Lists your certificates, newest first. CLI: `dna ssl list`
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct SslListRequest {
    /// Keeps only the certificates of this domain. For wildcard certificates both `*.example.com` and `example.com`
    /// match.
    pub domain_name: String,
    /// The page. The API's default page size is 10 and the largest 1000. Sorting is not supported: the order is
    /// always newest first.
    pub paging: Paging,
}

impl SslListRequest {
    /// Lists every certificate.
    pub fn new() -> Self {
        Self::default()
    }

    /// Keeps only the certificates of this domain.
    pub fn domain_name(mut self, domain_name: impl Into<String>) -> Self {
        self.domain_name = domain_name.into();
        self
    }
}

paged!(SslListRequest => CertificateListItem);

impl Request for SslListRequest {
    type Response = Page<CertificateListItem>;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/ssls")
    }

    fn query(&self) -> Query {
        Query::new()
            .set_str("DomainName", &self.domain_name)
            .set_opt("SkipCount", self.paging.skip)
            .set_opt("MaxResultCount", self.paging.limit)
    }

    fn validate(&self, v: &mut Validator) {
        self.paging.validate(v);
        v.check(self.paging.limit.unwrap_or(0) <= 1000, "limit", "must be at most 1000");
        v.check(self.paging.sort.is_none(), "sort", "is not supported: certificates are always listed newest first");
    }
}

/// Generates a CSR and its private key. CLI: `dna ssl generate-csr`
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct SslGenerateCsrRequest {
    /// The common name (at most 72 characters).
    pub domain_name: String,
    /// The 2-letter ISO country code.
    pub country: String,
    /// The state or province.
    pub state: String,
    /// The locality (city).
    pub locality: String,
    /// The organization.
    pub organization: String,
    /// The organizational unit.
    pub organization_unit: String,
    /// Extra names (SAN).
    pub alternative_names: Vec<String>,
}

impl SslGenerateCsrRequest {
    /// A CSR for `domain_name`. Set the subject with the other methods; all of them are required.
    pub fn new(domain_name: impl Into<String>) -> Self {
        SslGenerateCsrRequest { domain_name: domain_name.into(), ..Default::default() }
    }

    /// Sets the 2-letter ISO country code.
    pub fn country(mut self, country: impl Into<String>) -> Self {
        self.country = country.into();
        self
    }

    /// Sets the state or province.
    pub fn state(mut self, state: impl Into<String>) -> Self {
        self.state = state.into();
        self
    }

    /// Sets the locality (city).
    pub fn locality(mut self, locality: impl Into<String>) -> Self {
        self.locality = locality.into();
        self
    }

    /// Sets the organization.
    pub fn organization(mut self, organization: impl Into<String>) -> Self {
        self.organization = organization.into();
        self
    }

    /// Sets the organizational unit.
    pub fn organization_unit(mut self, unit: impl Into<String>) -> Self {
        self.organization_unit = unit.into();
        self
    }

    /// Sets extra names (SAN).
    pub fn alternative_names<S: Into<String>>(mut self, names: impl IntoIterator<Item = S>) -> Self {
        self.alternative_names = names.into_iter().map(Into::into).collect();
        self
    }
}

impl Request for SslGenerateCsrRequest {
    type Response = Csr;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/ssls/generate-csr", OperationSafety::Read)
    }

    fn body(&self) -> Option<Value> {
        Some(
            JsonBody::new()
                .insert("domainName", self.domain_name.trim())
                .insert("country", self.country.trim().to_ascii_uppercase())
                .insert("state", self.state.trim())
                .insert("locality", self.locality.trim())
                .insert("organization", self.organization.trim())
                .insert("organizationUnit", self.organization_unit.trim())
                .insert_list("alternativeNames", &trim_all(&self.alternative_names))
                .into_value(),
        )
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        v.max_len("domainName", &self.domain_name, 72);
        v.required("country", &self.country);
        v.check(
            self.country.trim().is_empty() || self.country.trim().chars().count() == 2,
            "country",
            "must be a 2-letter ISO code",
        );
        v.required("state", &self.state);
        v.required("locality", &self.locality);
        v.required("organization", &self.organization);
        v.required("organizationUnit", &self.organization_unit);
    }
}

/// Decodes a CSR. CLI: `dna ssl check-csr`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SslCheckCsrRequest {
    /// The CSR, PEM-encoded.
    pub csr: String,
}

impl SslCheckCsrRequest {
    /// Decodes `csr`.
    pub fn new(csr: impl Into<String>) -> Self {
        SslCheckCsrRequest { csr: csr.into() }
    }
}

impl Request for SslCheckCsrRequest {
    type Response = CsrInfo;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/ssls/check-csr", OperationSafety::Read)
    }

    fn body(&self) -> Option<Value> {
        Some(JsonBody::new().insert("csr", self.csr.trim()).into_value())
    }

    fn validate(&self, v: &mut Validator) {
        v.required("csr", &self.csr);
    }
}

/// Buys a certificate package for a domain. It charges your balance. The result carries the ID of the new
/// certificate, which must be configured within 15 days. CLI: `dna ssl initialize`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SslInitializeRequest {
    /// The domain.
    pub domain_name: String,
    /// The package code: an [`SslProduct::product_code`](crate::products::SslProduct::product_code).
    pub package_code: String,
}

impl SslInitializeRequest {
    /// Buys `package_code` for `domain_name`.
    pub fn new(domain_name: impl Into<String>, package_code: impl Into<String>) -> Self {
        SslInitializeRequest { domain_name: domain_name.into(), package_code: package_code.into() }
    }
}

impl Request for SslInitializeRequest {
    type Response = SslOperationResult;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/ssls/initialize", OperationSafety::Billable)
    }

    fn body(&self) -> Option<Value> {
        Some(
            JsonBody::new()
                .insert("domainName", self.domain_name.trim())
                .insert("packageCode", self.package_code.trim())
                .into_value(),
        )
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        v.required("packageCode", &self.package_code);
    }
}

/// Submits the CSR, validation method and contact of a certificate for issuance. CLI: `dna ssl configure`
///
/// ```
/// use domainnameapi::ssl::{AuthenticationType, SslConfigureRequest, SslContact, WebServerType};
///
/// # let (certificate_id, csr) = ("6f1c0000-0000-4000-8000-000000000001", "-----BEGIN CERTIFICATE REQUEST-----");
/// let contact = SslContact { first_name: "Ada".into(), last_name: "Lovelace".into(), ..Default::default() };
/// let request = SslConfigureRequest::new(certificate_id, csr, contact)
///     .authentication_type(AuthenticationType::Email)
///     .approver_mail("admin@example.com")
///     .web_server_type(WebServerType::Apache);
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SslConfigureRequest {
    /// The certificate: the ID returned by initialize or renew.
    pub certificate_id: String,
    /// The CSR, PEM-encoded. Its common name must match the certificate's domain.
    pub csr: String,
    /// The contact.
    pub contact: SslContact,
    /// The private key, PEM-encoded; optional when the CSR was made elsewhere.
    pub private_key: String,
    /// The address that receives the approval e-mail ([`AuthenticationType::Email`]). List the allowed addresses
    /// with [`SslClient::approvers`].
    pub approver_mail: String,
    /// The validation method.
    pub authentication_type: Option<AuthenticationType>,
    /// The web server.
    pub web_server_type: Option<WebServerType>,
    /// Additional names.
    pub san_list: Vec<String>,
    /// Additional wildcard names.
    pub wildcard_list: Vec<String>,
}

impl SslConfigureRequest {
    /// Configures `certificate_id` with `csr` and `contact`.
    pub fn new(certificate_id: impl Into<String>, csr: impl Into<String>, contact: SslContact) -> Self {
        SslConfigureRequest {
            certificate_id: certificate_id.into(),
            csr: csr.into(),
            contact,
            private_key: String::new(),
            approver_mail: String::new(),
            authentication_type: None,
            web_server_type: None,
            san_list: Vec::new(),
            wildcard_list: Vec::new(),
        }
    }

    /// Sets the private key.
    pub fn private_key(mut self, key: impl Into<String>) -> Self {
        self.private_key = key.into();
        self
    }

    /// Sets the approver address.
    pub fn approver_mail(mut self, mail: impl Into<String>) -> Self {
        self.approver_mail = mail.into();
        self
    }

    /// Sets the validation method.
    pub fn authentication_type(mut self, method: AuthenticationType) -> Self {
        self.authentication_type = Some(method);
        self
    }

    /// Sets the web server.
    pub fn web_server_type(mut self, server: WebServerType) -> Self {
        self.web_server_type = Some(server);
        self
    }

    /// Sets the additional names.
    pub fn san_list<S: Into<String>>(mut self, names: impl IntoIterator<Item = S>) -> Self {
        self.san_list = names.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the additional wildcard names.
    pub fn wildcard_list<S: Into<String>>(mut self, names: impl IntoIterator<Item = S>) -> Self {
        self.wildcard_list = names.into_iter().map(Into::into).collect();
        self
    }
}

impl Request for SslConfigureRequest {
    type Response = SslOperationResult;

    fn endpoint(&self) -> Endpoint {
        Endpoint::put("/api/v1/ssls/configure", OperationSafety::Write)
    }

    fn body(&self) -> Option<Value> {
        Some(
            certificate_body(&self.certificate_id)
                .insert("csr", self.csr.trim())
                .insert("contact", self.contact.to_json())
                .insert_str("privateKey", &self.private_key)
                .insert_str("approverMail", &self.approver_mail)
                .insert_opt("authenticationType", self.authentication_type.as_ref().map(AuthenticationType::as_str))
                .insert_opt("webServerType", self.web_server_type.as_ref().map(WebServerType::as_str))
                .insert_str("sanList", &trim_all(&self.san_list).join(","))
                .insert_str("wildcardList", &trim_all(&self.wildcard_list).join(","))
                .into_value(),
        )
    }

    fn validate(&self, v: &mut Validator) {
        validate_certificate_id(v, &self.certificate_id);
        v.required("csr", &self.csr);
        let mail = self.approver_mail.trim();
        v.check(mail.is_empty() || mail.contains('@'), "approverMail", "is not an e-mail address");
        self.contact.validate(v);
    }
}

/// Reissues a certificate with a new CSR, key or approver. CLI: `dna ssl reissue`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SslReissueRequest {
    /// The certificate ID.
    pub certificate_id: String,
    /// A new approver address.
    pub approver_mail: String,
    /// A new CSR.
    pub csr: String,
    /// A new private key.
    pub private_key: String,
}

impl SslReissueRequest {
    /// Reissues `certificate_id`.
    pub fn new(certificate_id: impl Into<String>) -> Self {
        SslReissueRequest {
            certificate_id: certificate_id.into(),
            approver_mail: String::new(),
            csr: String::new(),
            private_key: String::new(),
        }
    }

    /// Sets a new approver address.
    pub fn approver_mail(mut self, mail: impl Into<String>) -> Self {
        self.approver_mail = mail.into();
        self
    }

    /// Sets a new CSR.
    pub fn csr(mut self, csr: impl Into<String>) -> Self {
        self.csr = csr.into();
        self
    }

    /// Sets a new private key.
    pub fn private_key(mut self, key: impl Into<String>) -> Self {
        self.private_key = key.into();
        self
    }
}

impl Request for SslReissueRequest {
    type Response = SslOperationResult;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/ssls/reissue", OperationSafety::Write)
    }

    fn body(&self) -> Option<Value> {
        Some(
            certificate_body(&self.certificate_id)
                .insert_str("approverMail", &self.approver_mail)
                .insert_str("csr", &self.csr)
                .insert_str("privateKey", &self.private_key)
                .into_value(),
        )
    }

    fn validate(&self, v: &mut Validator) {
        validate_certificate_id(v, &self.certificate_id);
    }
}

/// Lists the e-mail addresses allowed for e-mail validation: of an existing certificate, or of a domain and package
/// before a purchase. CLI: `dna ssl approvers`
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct SslApproversRequest {
    /// An existing certificate. The API takes the domain and the package from it.
    pub certificate_id: String,
    /// The domain you are about to buy a certificate for; needs `package_code`.
    pub domain_name: String,
    /// The package you are about to buy.
    pub package_code: String,
}

impl SslApproversRequest {
    /// The approvers of an existing certificate.
    pub fn for_certificate(certificate_id: impl Into<String>) -> Self {
        SslApproversRequest { certificate_id: certificate_id.into(), ..Default::default() }
    }

    /// The approvers of a domain and package, before a purchase.
    pub fn for_purchase(domain_name: impl Into<String>, package_code: impl Into<String>) -> Self {
        SslApproversRequest { domain_name: domain_name.into(), package_code: package_code.into(), ..Default::default() }
    }
}

impl Request for SslApproversRequest {
    type Response = ApproverList;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/ssls/approvers")
    }

    fn query(&self) -> Query {
        Query::new()
            .set_str("CertificateId", &self.certificate_id)
            .set_str("DomainName", &self.domain_name)
            .set_str("PackageCode", &self.package_code)
    }

    fn validate(&self, v: &mut Validator) {
        if !self.certificate_id.trim().is_empty() {
            validate_certificate_id(v, &self.certificate_id);
        } else if self.domain_name.trim().is_empty() {
            v.add(
                "certificateId",
                "or domainName with packageCode is required: use for_certificate for an existing certificate, or \
                 for_purchase before a purchase",
            );
        } else {
            v.required("packageCode", &self.package_code);
        }
    }
}

macro_rules! certificate_request {
    ($(#[$meta:meta])* $name:ident, $response:ty, $endpoint:expr, $style:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        #[non_exhaustive]
        pub struct $name {
            /// The certificate ID (a GUID).
            pub certificate_id: String,
        }

        impl $name {
            /// Creates the request for this certificate.
            pub fn new(certificate_id: impl Into<String>) -> Self {
                $name { certificate_id: certificate_id.into() }
            }
        }

        impl Request for $name {
            type Response = $response;

            fn endpoint(&self) -> Endpoint {
                $endpoint
            }

            certificate_request!(@$style);

            fn validate(&self, v: &mut Validator) {
                validate_certificate_id(v, &self.certificate_id);
            }
        }
    };
    (@body) => {
        fn body(&self) -> Option<Value> {
            Some(certificate_body(&self.certificate_id).into_value())
        }
    };
    (@query) => {
        fn query(&self) -> Query {
            Query::new().set_str("CertificateId", &self.certificate_id)
        }
    };
}

certificate_request!(
    /// Renews a certificate: it opens a new one-year certificate linked to it. It charges your balance (the renewal
    /// price). CLI: `dna ssl renew`
    SslRenewRequest,
    SslOperationResult,
    Endpoint::post("/api/v1/ssls/renew", OperationSafety::Billable),
    body
);

certificate_request!(
    /// Refreshes and reads the state of a certificate. CLI: `dna ssl status`
    SslStatusRequest,
    CertificateStatus,
    Endpoint::put("/api/v1/ssls/check-status", OperationSafety::Read).idempotent(),
    body
);

certificate_request!(
    /// Reads the full record of a certificate. CLI: `dna ssl get`
    SslGetRequest,
    Certificate,
    Endpoint::get("/api/v1/ssls/certificate"),
    query
);

certificate_request!(
    /// Downloads an issued certificate. CLI: `dna ssl content`
    SslContentRequest,
    CertificateContent,
    Endpoint::get("/api/v1/ssls/certificate-content"),
    query
);

certificate_request!(
    /// Revokes a certificate. This cannot be undone. CLI: `dna ssl revoke`
    SslRevokeRequest,
    SslOperationResult,
    Endpoint::delete("/api/v1/ssls/revoke", OperationSafety::Destructive),
    query
);

// ------------------------------------------------------------------------------------------------ client

/// SSL certificates. Get it with [`Client::ssl`].
#[derive(Debug, Clone, Copy)]
pub struct SslClient<'a> {
    client: &'a Client,
}

impl<'a> SslClient<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        SslClient { client }
    }

    /// Returns one page of your certificates, newest first. Use it to find the ID of a certificate by its domain.
    pub async fn list(&self, request: SslListRequest) -> Result<Page<CertificateListItem>, Error> {
        self.client.send(&request).await
    }

    /// Returns every matching certificate, newest first, fetching pages as the stream is read.
    pub fn list_all(&self, request: SslListRequest) -> ItemStream<CertificateListItem> {
        self.client.list_all(request)
    }

    /// Returns a new CSR and its private key.
    pub async fn generate_csr(&self, request: SslGenerateCsrRequest) -> Result<Csr, Error> {
        self.client.send(&request).await
    }

    /// Decodes a PEM-encoded CSR.
    pub async fn check_csr(&self, csr: &str) -> Result<CsrInfo, Error> {
        self.client.send(&SslCheckCsrRequest::new(csr)).await
    }

    /// Buys a certificate package for a domain and charges your balance. Store the certificate ID of the result:
    /// [`configure`](SslClient::configure) and every later call take it. Configure the certificate within 15 days;
    /// after that it is cancelled. Do not use this to renew a certificate: see [`renew`](SslClient::renew).
    pub async fn initialize(&self, domain_name: &str, package_code: &str) -> Result<SslOperationResult, Error> {
        self.client.send(&SslInitializeRequest::new(domain_name, package_code)).await
    }

    /// Renews a certificate and charges your balance (the renewal price). It opens a new one-year certificate linked
    /// to the old one; the result's certificate ID is the new certificate's, which waits for configuration: pass it to
    /// [`configure`](SslClient::configure) within 15 days.
    ///
    /// The API allows a renewal from 30 days before the subscription ends and only once per certificate. Calling it
    /// again while the renewal still waits for configuration returns the same renewal without charging again, which
    /// is the way to recover when [`Error::is_outcome_unknown`] is true.
    pub async fn renew(&self, certificate_id: &str) -> Result<SslOperationResult, Error> {
        self.client.send(&SslRenewRequest::new(certificate_id)).await
    }

    /// Submits the CSR, validation method and contact of a certificate bought with
    /// [`initialize`](SslClient::initialize) or [`renew`](SslClient::renew). Then poll
    /// [`status`](SslClient::status) until it is issued.
    pub async fn configure(&self, request: SslConfigureRequest) -> Result<SslOperationResult, Error> {
        self.client.send(&request).await
    }

    /// Refreshes and returns the state of a certificate.
    pub async fn status(&self, certificate_id: &str) -> Result<CertificateStatus, Error> {
        self.client.send(&SslStatusRequest::new(certificate_id)).await
    }

    /// Returns the full record of a certificate: subscription, configuration and contacts.
    pub async fn get(&self, certificate_id: &str) -> Result<Certificate, Error> {
        self.client.send(&SslGetRequest::new(certificate_id)).await
    }

    /// Returns the issued certificate and its chain.
    pub async fn content(&self, certificate_id: &str) -> Result<CertificateContent, Error> {
        self.client.send(&SslContentRequest::new(certificate_id)).await
    }

    /// Reissues a certificate with a new CSR, key or approver.
    pub async fn reissue(&self, request: SslReissueRequest) -> Result<SslOperationResult, Error> {
        self.client.send(&request).await
    }

    /// Returns the e-mail addresses allowed for e-mail validation.
    pub async fn approvers(&self, request: SslApproversRequest) -> Result<Vec<String>, Error> {
        Ok(self.client.send(&request).await?.mails)
    }

    /// Revokes a certificate. This cannot be undone.
    pub async fn revoke(&self, certificate_id: &str) -> Result<SslOperationResult, Error> {
        self.client.send(&SslRevokeRequest::new(certificate_id)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const ID: &str = "6f1c0000-0000-4000-8000-000000000001";

    fn errors(request: &impl Request) -> Vec<String> {
        let mut v = Validator::new();
        request.validate(&mut v);
        v.errors().iter().map(ToString::to_string).collect()
    }

    #[test]
    fn certificate_ids_must_be_guids() {
        assert!(is_guid(ID));
        assert!(!is_guid("example.com"));
        assert!(errors(&SslGetRequest::new(ID)).is_empty());
        assert!(errors(&SslRenewRequest::new("example.com"))[0].contains("no longer addressed by domain name"));
        assert!(errors(&SslRevokeRequest::new(""))[0].contains("is required"));
    }

    #[test]
    fn approvers_need_a_certificate_or_a_purchase() {
        assert!(errors(&SslApproversRequest::for_certificate(ID)).is_empty());
        assert!(errors(&SslApproversRequest::for_purchase("example.com", "PositiveSSL")).is_empty());
        assert_eq!(errors(&SslApproversRequest::for_purchase("example.com", "")).len(), 1);
        assert_eq!(errors(&SslApproversRequest::default()).len(), 1);
    }

    #[test]
    fn configure_body() {
        let contact = SslContact {
            first_name: "Ada".into(),
            last_name: "Lovelace".into(),
            company_name: "Example".into(),
            email: "ada@example.com".into(),
            address: "1 Street".into(),
            city: "Istanbul".into(),
            postal_code: "34000".into(),
            country: "tr".into(),
            phone_country_code: "+90".into(),
            phone: "555 123 45 67".into(),
            ..Default::default()
        };
        let request = SslConfigureRequest::new(ID, "csr", contact)
            .authentication_type(AuthenticationType::Cname)
            .san_list(["a.example.com", "b.example.com"]);
        assert!(errors(&request).is_empty(), "{:?}", errors(&request));
        let body = request.body().unwrap();
        assert_eq!(body["authenticationType"], "Cname");
        assert_eq!(body["sanList"], "a.example.com,b.example.com");
        assert_eq!(body["contact"]["phone"], "5551234567");
        assert_eq!(body["contact"]["country"], "TR");
        assert!(body.get("wildcardList").is_none());
    }

    #[test]
    fn list_does_not_sort() {
        let mut request = SslListRequest::new().limit(2000);
        request.paging.sort = Some("x".into());
        assert_eq!(errors(&request).len(), 2);
    }

    #[test]
    fn full_chain() {
        let content: CertificateContent = serde_json::from_value(json!({
            "certificate": "CERT\n", "intermediateCertificate1": "INT1", "intermediateCertificate2": "", "rootCertificate": "ROOT"
        }))
        .unwrap();
        assert_eq!(content.full_chain(), "CERT\nINT1\n");
    }
}
