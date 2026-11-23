//! Reusable contact handles: create a contact once, then register many domains with its handle. CLI: `dna contact`.

use crate::de::{model, str_enum};
use crate::request::path_segment;
use crate::{Client, Empty, Endpoint, Error, JsonBody, OperationSafety, Request, Validator};
use chrono::{DateTime, Utc};
use serde_json::Value;

str_enum! {
    /// The role of a contact.
    pub enum ContactType {
        /// The domain owner.
        Registrant => "Registrant" ["owner"],
        /// The administrative contact.
        Admin => "Admin" ["administrative"],
        /// The technical contact.
        Tech => "Tech" ["technical"],
        /// The billing contact.
        Billing => "Billing",
    }
}

model! {
    @open
    /// A person or organisation in a contact role.
    ///
    /// Build one with a struct literal; the requests tidy it before sending: they trim every field, upper-case the
    /// country and keep only the digits of the phone and fax fields (`"+90 (555) 123-45-67"` becomes
    /// `"905551234567"`).
    ///
    /// ```
    /// use domainnameapi::contacts::Contact;
    ///
    /// let contact = Contact {
    ///     first_name: "Ada".into(),
    ///     last_name: "Lovelace".into(),
    ///     email: "ada@example.com".into(),
    ///     address: "1 Example Street".into(),
    ///     city: "Istanbul".into(),
    ///     state: "Istanbul".into(),
    ///     postal_code: "34000".into(),
    ///     country: "TR".into(),
    ///     phone_country_code: "90".into(),
    ///     phone: "5551234567".into(),
    ///     ..Default::default()
    /// };
    /// ```
    pub struct Contact {
        /// The first name (at most 80 characters).
        "firstName" => pub first_name: String,
        /// The last name (at most 80 characters).
        "lastName" => pub last_name: String,
        /// The company name; optional.
        "companyName" => pub company_name: String,
        /// The e-mail address.
        "eMail" => pub email: String,
        /// The street address.
        "address" => pub address: String,
        /// The city.
        "city" => pub city: String,
        /// The state or province. Required: repeat the city when there is none.
        "state" => pub state: String,
        /// The postal code (at most 15 characters).
        "postalCode" => pub postal_code: String,
        /// The ISO 3166-1 alpha-2 country code, for example `"TR"` (`"GB"`, not `"UK"`).
        "country" => pub country: String,
        /// The calling code without `+`, for example `"90"`.
        "phoneCountryCode" => pub phone_country_code: String,
        /// The number without the calling code, for example `"5551234567"`.
        "phone" => pub phone: String,
        /// The fax calling code; optional.
        "faxCountryCode" => pub fax_country_code: String,
        /// The fax number; optional.
        "fax" => pub fax: String,
        /// Publishes the contact in WHOIS. Leave it `None` to keep the registry default (recommended).
        "discloseFlag" => pub disclose: Option<bool>,
        /// The role. The API sets it on the contacts of a domain; the requests that need it set it themselves.
        "contactType" => pub contact_type: Option<ContactType>,
        /// The contact's handle, for example `"DNA-12345"`. Set by the API.
        "handle" => pub handle: String,
    }
}

fn digits(text: &str) -> String {
    text.chars().filter(char::is_ascii_digit).collect()
}

impl Contact {
    /// Returns a tidied copy, as the requests send it: trimmed fields, an upper-case country and digits-only phone
    /// and fax fields.
    pub fn normalized(&self) -> Contact {
        let t = |s: &str| s.trim().to_string();
        Contact {
            first_name: t(&self.first_name),
            last_name: t(&self.last_name),
            company_name: t(&self.company_name),
            email: t(&self.email),
            address: t(&self.address),
            city: t(&self.city),
            state: t(&self.state),
            postal_code: t(&self.postal_code),
            country: self.country.trim().to_ascii_uppercase(),
            phone_country_code: digits(&self.phone_country_code),
            phone: digits(&self.phone),
            fax_country_code: digits(&self.fax_country_code),
            fax: digits(&self.fax),
            disclose: self.disclose,
            contact_type: self.contact_type.clone(),
            handle: t(&self.handle),
        }
    }

    /// The contact details as the API takes them (`ContactInformationDto`), with an optional role.
    pub(crate) fn to_json(&self, role: Option<&ContactType>) -> Value {
        let c = self.normalized();
        JsonBody::new()
            .insert("firstName", &c.first_name)
            .insert("lastName", &c.last_name)
            .insert_str("companyName", &c.company_name)
            .insert("eMail", &c.email)
            .insert("address", &c.address)
            .insert("city", &c.city)
            .insert("state", &c.state)
            .insert("postalCode", &c.postal_code)
            .insert("country", &c.country)
            .insert("phoneCountryCode", &c.phone_country_code)
            .insert("phone", &c.phone)
            .insert_str("faxCountryCode", &c.fax_country_code)
            .insert_str("fax", &c.fax)
            .insert_opt("discloseFlag", c.disclose)
            .insert_opt("contactType", role.map(ContactType::as_str))
            .into_value()
    }

    /// Checks the fields the API requires. `prefix` is put in front of the field names, for example `"contact."`.
    pub fn validate(&self, v: &mut Validator, prefix: &str) {
        let c = self.normalized();
        let f = |name: &str| format!("{prefix}{name}");
        v.required(f("firstName"), &c.first_name);
        v.required(f("lastName"), &c.last_name);
        v.required(f("email"), &c.email);
        v.check(c.email.is_empty() || c.email.contains('@'), f("email"), "is not an e-mail address");
        v.required(f("address"), &c.address);
        v.required(f("city"), &c.city);
        v.required(f("state"), &c.state);
        v.required(f("postalCode"), &c.postal_code);
        v.required(f("country"), &c.country);
        v.check(
            c.country.is_empty() || c.country.chars().count() == 2,
            f("country"),
            "must be a 2-letter ISO code such as TR or US",
        );
        v.required(f("phoneCountryCode"), &c.phone_country_code);
        v.check(c.phone_country_code.len() <= 3, f("phoneCountryCode"), "must be at most 3 digits, for example 90");
        v.required(f("phone"), &c.phone);
        v.check(c.phone.len() <= 16, f("phone"), "must be at most 16 digits");
        v.max_len(f("firstName"), &c.first_name, 80);
        v.max_len(f("lastName"), &c.last_name, 80);
        v.max_len(f("companyName"), &c.company_name, 256);
        v.max_len(f("address"), &c.address, 256);
        v.max_len(f("city"), &c.city, 80);
        v.max_len(f("state"), &c.state, 80);
        v.max_len(f("postalCode"), &c.postal_code, 15);
    }
}

model! {
    /// The e-mail or phone verification state of a contact. A domain stays usable after registration, but when the
    /// registrant e-mail is not verified within 15 days the domain is put on hold.
    pub struct ContactVerification {
        /// The owner's name.
        "ownerName" => pub owner_name: String,
        /// The e-mail address or phone number.
        "fieldValue" => pub field_value: String,
        /// What is verified, for example `"Email"`.
        "fieldType" => pub field_type: String,
        /// The state of the verification.
        "operationStatus" => pub operation_status: String,
        /// Whether the owner has verified it.
        "isUserVerified" => pub verified: bool,
        /// When the owner verified it.
        "userVerificationDate" => pub verified_at: Option<DateTime<Utc>>,
        /// When the verification started.
        "createdDate" => pub created_at: Option<DateTime<Utc>>,
        /// When the verification link expires.
        "tokenExpireDate" => pub token_expires_at: Option<DateTime<Utc>>,
        /// When the notice was last sent again.
        "resendDate" => pub resent_at: Option<DateTime<Utc>>,
        /// How often the notice was sent again.
        "resendRetryCount" => pub resend_retry_count: i32,
        /// When the owner last tried.
        "lastAttemptDate" => pub last_attempt_at: Option<DateTime<Utc>>,
        /// The number of attempts.
        "attemptNumber" => pub attempt_number: i32,
    }
}

model! {
    /// A contact handle with its verification state.
    pub struct ContactDetails {
        /// The contact.
        "contact" => pub contact: Contact,
        /// The e-mail and phone verifications.
        "verificationObjects" => pub verifications: Vec<ContactVerification>,
    }
}

model! {
    /// The answer to [`ContactCreateRequest`].
    pub struct ContactCreateResult {
        /// The new handle, for example `"DNA-12345"`.
        "handle" => pub handle: String,
        /// The API's message.
        "operationMessage" => pub message: String,
        /// Whether the contact was created. Check it: the API can answer HTTP 200 with `false`.
        "isSuccess" => pub success: bool,
    }
}

model! {
    /// How the domains using a contact handle were updated.
    pub struct ContactUpdateStats {
        /// The domains that use the handle.
        "totalDomainCount" => pub total_domain_count: i32,
        /// The domains updated.
        "successCount" => pub success_count: i32,
        /// The domains waiting for documents.
        "pendingDocumentCount" => pub pending_document_count: i32,
        /// The domains that failed.
        "failedCount" => pub failed_count: i32,
        /// The domains that got a new handle.
        "splitCount" => pub split_count: i32,
    }
}

model! {
    /// The outcome of a contact update for one domain.
    pub struct AffectedDomain {
        /// The domain.
        "domainName" => pub domain_name: String,
        /// The outcome.
        "status" => pub status: String,
        /// A new handle: some registries cannot share a handle. Store it when it is set.
        "newHandle" => pub new_handle: String,
        /// Why, for a failure or a split.
        "reason" => pub reason: String,
        /// The API's message.
        "message" => pub message: String,
    }
}

model! {
    /// The answer to [`ContactUpdateRequest`]: the contact itself and every domain that uses it.
    pub struct ContactUpdateResult {
        /// The handle.
        "handle" => pub handle: String,
        /// Whether the contact itself was updated. Check it: the API can answer HTTP 200 with `false`.
        "contactUpdateSuccess" => pub success: bool,
        /// The counts.
        "stats" => pub stats: ContactUpdateStats,
        /// Each domain using the handle.
        "affectedDomains" => pub affected_domains: Vec<AffectedDomain>,
    }
}

model! {
    /// The answer to [`ContactVerificationRequest`].
    pub struct ContactVerificationList {
        /// The verifications.
        "results" => pub results: Vec<ContactVerification>,
    }
}

// ------------------------------------------------------------------------------------------------ requests

/// Reads a contact handle. CLI: `dna contact get`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ContactGetRequest {
    /// The handle, for example `"DNA-12345"`.
    pub handle: String,
}

impl ContactGetRequest {
    /// Reads this handle.
    pub fn new(handle: impl Into<String>) -> Self {
        ContactGetRequest { handle: handle.into() }
    }
}

impl Request for ContactGetRequest {
    type Response = ContactDetails;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get(format!("/api/v1/contacts/{}", path_segment(&self.handle)))
    }

    fn validate(&self, v: &mut Validator) {
        v.required("handle", &self.handle);
    }
}

/// Creates a contact handle. CLI: `dna contact create`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ContactCreateRequest {
    /// The contact.
    pub contact: Contact,
}

impl ContactCreateRequest {
    /// Creates a handle for this contact.
    pub fn new(contact: Contact) -> Self {
        ContactCreateRequest { contact }
    }
}

impl Request for ContactCreateRequest {
    type Response = ContactCreateResult;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/contacts", OperationSafety::Write)
    }

    fn body(&self) -> Option<Value> {
        Some(JsonBody::new().insert("contact", self.contact.to_json(None)).into_value())
    }

    fn validate(&self, v: &mut Validator) {
        self.contact.validate(v, "contact.");
    }
}

/// Replaces a contact handle's details, and those of every domain using it. CLI: `dna contact update`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ContactUpdateRequest {
    /// The handle.
    pub handle: String,
    /// The complete new contact: the update is a full replacement, not a patch.
    pub contact: Contact,
}

impl ContactUpdateRequest {
    /// Replaces the details of `handle` with `contact`.
    pub fn new(handle: impl Into<String>, contact: Contact) -> Self {
        ContactUpdateRequest { handle: handle.into(), contact }
    }
}

impl Request for ContactUpdateRequest {
    type Response = ContactUpdateResult;

    fn endpoint(&self) -> Endpoint {
        Endpoint::put(format!("/api/v1/contacts/{}", path_segment(&self.handle)), OperationSafety::Write)
    }

    fn body(&self) -> Option<Value> {
        Some(JsonBody::new().insert("contact", self.contact.to_json(None)).into_value())
    }

    fn validate(&self, v: &mut Validator) {
        v.required("handle", &self.handle);
        self.contact.validate(v, "contact.");
    }
}

/// Reads the verification state of a contact. CLI: `dna contact verification`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ContactVerificationRequest {
    /// The handle.
    pub handle: String,
}

impl ContactVerificationRequest {
    /// Reads the verification state of this handle.
    pub fn new(handle: impl Into<String>) -> Self {
        ContactVerificationRequest { handle: handle.into() }
    }
}

impl Request for ContactVerificationRequest {
    type Response = ContactVerificationList;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/contacts/verification/check")
    }

    fn query(&self) -> crate::Query {
        crate::Query::new().set_str("HandleCode", &self.handle)
    }

    fn validate(&self, v: &mut Validator) {
        v.required("handle", &self.handle);
    }
}

/// Sends the verification notice of a contact again. CLI: `dna contact resend-verification`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ContactResendVerificationRequest {
    /// The handle.
    pub handle: String,
}

impl ContactResendVerificationRequest {
    /// Sends the notice of this handle again.
    pub fn new(handle: impl Into<String>) -> Self {
        ContactResendVerificationRequest { handle: handle.into() }
    }
}

impl Request for ContactResendVerificationRequest {
    type Response = Empty;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/contacts/verification/request", OperationSafety::Write)
    }

    fn body(&self) -> Option<Value> {
        Some(JsonBody::new().insert("handleCode", self.handle.trim()).into_value())
    }

    fn validate(&self, v: &mut Validator) {
        v.required("handle", &self.handle);
    }
}

// ------------------------------------------------------------------------------------------------ client

/// Contact handles. Get it with [`Client::contacts`].
#[derive(Debug, Clone, Copy)]
pub struct ContactsClient<'a> {
    client: &'a Client,
}

impl<'a> ContactsClient<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        ContactsClient { client }
    }

    /// Returns a contact handle and its verification state.
    pub async fn get(&self, handle: &str) -> Result<ContactDetails, Error> {
        self.client.send(&ContactGetRequest::new(handle)).await
    }

    /// Creates a contact handle, which [`DomainRegisterRequest`](crate::domains::DomainRegisterRequest) and other
    /// requests take. Check [`ContactCreateResult::success`].
    pub async fn create(&self, contact: Contact) -> Result<ContactCreateResult, Error> {
        self.client.send(&ContactCreateRequest::new(contact)).await
    }

    /// Replaces the details of a contact handle. Every domain using the handle is updated too; the result reports
    /// each of them.
    pub async fn update(&self, handle: &str, contact: Contact) -> Result<ContactUpdateResult, Error> {
        self.client.send(&ContactUpdateRequest::new(handle, contact)).await
    }

    /// Returns the e-mail and phone verification state of a contact.
    pub async fn verification(&self, handle: &str) -> Result<Vec<ContactVerification>, Error> {
        Ok(self.client.send(&ContactVerificationRequest::new(handle)).await?.results)
    }

    /// Sends the verification notice of a contact again.
    pub async fn resend_verification(&self, handle: &str) -> Result<(), Error> {
        self.client.send(&ContactResendVerificationRequest::new(handle)).await.map(|_| ())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::json;

    pub(crate) fn sample() -> Contact {
        Contact {
            first_name: " Ada ".into(),
            last_name: "Lovelace".into(),
            email: "ada@example.com".into(),
            address: "1 Example Street".into(),
            city: "Istanbul".into(),
            state: "Istanbul".into(),
            postal_code: "34000".into(),
            country: "tr".into(),
            phone_country_code: "+90".into(),
            phone: "(555) 123-45-67".into(),
            ..Default::default()
        }
    }

    #[test]
    fn contacts_are_tidied_before_sending() {
        let body = ContactCreateRequest::new(sample()).body().unwrap();
        assert_eq!(
            body,
            json!({"contact": {
                "firstName": "Ada", "lastName": "Lovelace", "eMail": "ada@example.com", "address": "1 Example Street",
                "city": "Istanbul", "state": "Istanbul", "postalCode": "34000", "country": "TR",
                "phoneCountryCode": "90", "phone": "5551234567"
            }})
        );
    }

    #[test]
    fn contacts_are_validated() {
        let mut v = Validator::new();
        Contact { email: "nope".into(), country: "TUR".into(), ..Default::default() }.validate(&mut v, "contact.");
        let fields: Vec<&str> = v.errors().iter().map(|e| e.field.as_str()).collect();
        assert!(fields.contains(&"contact.firstName"));
        assert!(fields.contains(&"contact.email"));
        assert!(fields.contains(&"contact.country"));

        let mut v = Validator::new();
        sample().validate(&mut v, "");
        assert!(v.is_valid(), "{:?}", v.errors());
    }

    #[test]
    fn contact_types_parse_aliases() {
        assert_eq!(ContactType::parse("owner"), ContactType::Registrant);
        assert_eq!(ContactType::parse("TECHNICAL"), ContactType::Tech);
        assert_eq!(ContactType::parse("Reseller"), ContactType::Other("Reseller".into()));
        assert_eq!(ContactType::Admin.to_string(), "Admin");
    }

    #[test]
    fn contact_details_read_loose_values() {
        let details: ContactDetails = serde_json::from_value(json!({
            "contact": {"firstName": "Ada", "phone": 5551234567u64, "discloseFlag": "1", "contactType": "registrant"},
            "verificationObjects": [{"isUserVerified": "true", "createdDate": "2026-01-02T03:04:05"}]
        }))
        .unwrap();
        assert_eq!(details.contact.phone, "5551234567");
        assert_eq!(details.contact.disclose, Some(true));
        assert_eq!(details.contact.contact_type, Some(ContactType::Registrant));
        assert!(details.verifications[0].verified);
        assert!(details.verifications[0].created_at.is_some());
    }
}
