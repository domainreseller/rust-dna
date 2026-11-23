//! Web hosting packages and services. CLI: `dna hosting`.
//!
//! The flow: pick a plan from [`HostingClient::packages`], [`HostingClient::create`] it (this charges the balance; the
//! API answers 202 while provisioning runs), then poll [`HostingClient::status`] until the hosting is active. Never
//! create again for a hosting stuck in provisioning: that charges twice.

use crate::de::model;
use crate::{Client, Endpoint, Error, JsonBody, OperationSafety, Query, Request, Validator};
use chrono::{DateTime, Utc};
use serde_json::Value;

/// Provisioning is running.
pub const STATUS_PROVISIONING: &str = "Provisioning";
/// The hosting is ready.
pub const STATUS_ACTIVE: &str = "Active";
/// Provisioning failed; the balance is refunded automatically.
pub const STATUS_FAILED: &str = "Failed";
/// The hosting is suspended.
pub const STATUS_SUSPENDED: &str = "Suspended";
/// The hosting is deleted.
pub const STATUS_DELETED: &str = "Deleted";

model! {
    /// A service plan of a hosting package.
    pub struct HostingPackagePlan {
        /// The value [`HostingClient::create`] takes.
        "servicePlan" => pub service_plan: String,
        /// The operating system; part of the plan and cannot be changed later.
        "operatingSystem" => pub operating_system: String,
        /// The control panel.
        "panelType" => pub panel_type: String,
    }
}

model! {
    /// A hosting package and the plans it offers.
    pub struct HostingPackage {
        /// The package name.
        "name" => pub name: String,
        /// The shortest lease, in months.
        "requiredMinLeasing" => pub required_min_leasing: i32,
        /// The longest lease, in months.
        "permittedMaxLeasing" => pub permitted_max_leasing: i32,
        /// The plans.
        "servicePlans" => pub service_plans: Vec<HostingPackagePlan>,
    }
}

model! {
    /// A hosting plan and its limits. A `None` limit means unlimited, not zero.
    pub struct ServicePlan {
        /// The plan name.
        "name" => pub name: String,
        /// The plan's ID at the hosting platform.
        "externalId" => pub external_id: String,
        /// The disk quota in MB.
        "diskQuotaMb" => pub disk_quota_mb: Option<i64>,
        /// The bandwidth limit in MB.
        "bandwidthLimitMb" => pub bandwidth_limit_mb: Option<i64>,
        /// The number of domains.
        "maxDomains" => pub max_domains: Option<i64>,
        /// The number of subdomains.
        "maxSubDomains" => pub max_sub_domains: Option<i64>,
        /// The number of databases.
        "maxDatabases" => pub max_databases: Option<i64>,
        /// The number of FTP accounts.
        "maxFtpAccounts" => pub max_ftp_accounts: Option<i64>,
        /// The number of e-mail accounts.
        "maxEmailAccounts" => pub max_email_accounts: Option<i64>,
        /// Whether a dedicated IP address is included.
        "hasDedicatedIp" => pub has_dedicated_ip: bool,
        /// Whether shell access is included.
        "hasShellAccess" => pub has_shell_access: bool,
    }
}

model! {
    /// The answer to [`HostingCreateRequest`].
    pub struct HostingCreateResult {
        /// The service ID (`"HST-..."`) of the hosting; store it.
        "serviceId" => pub service_id: String,
        /// The provisioning state.
        "status" => pub status: String,
        /// The control panel address; empty until the hosting is active.
        "panelAddress" => pub panel_address: String,
        /// The panel user name; empty until the hosting is active.
        "username" => pub username: String,
        /// The panel password; empty until the hosting is active.
        "password" => pub password: String,
        /// A token for [`HostingClient::get_by_token`].
        "token" => pub token: String,
    }
}

model! {
    /// A hosting service.
    pub struct Hosting {
        /// The UUID [`HostingClient::plans`] takes.
        "hostingId" => pub hosting_id: String,
        /// The service ID.
        "serviceId" => pub service_id: String,
        /// The domain.
        "domainName" => pub domain_name: String,
        /// The package.
        "packageName" => pub package_name: String,
        /// When the lease started.
        "leaseOn" => pub lease_on: Option<DateTime<Utc>>,
        /// When it expires.
        "expiredOn" => pub expires_on: Option<DateTime<Utc>>,
        /// The state, for example [`STATUS_ACTIVE`].
        "status" => pub status: String,
        /// The customer's ID at the hosting platform.
        "externalCustomerId" => pub external_customer_id: String,
    }
}

model! {
    /// The provisioning state of a hosting. The panel access fields are set only once the hosting is active.
    pub struct HostingStatus {
        /// The service ID.
        "serviceId" => pub service_id: String,
        /// The domain.
        "domainName" => pub domain_name: String,
        /// The state: [`STATUS_PROVISIONING`], [`STATUS_ACTIVE`], [`STATUS_FAILED`], ...
        "status" => pub status: String,
        /// The state in words.
        "statusMessage" => pub status_message: String,
        /// The control panel address.
        "panelAddress" => pub panel_address: String,
        /// The panel user name.
        "username" => pub username: String,
        /// The panel password.
        "password" => pub password: String,
        /// A token for [`HostingClient::get_by_token`].
        "token" => pub token: String,
        /// Why provisioning failed.
        "failureReason" => pub failure_reason: String,
    }
}

impl HostingStatus {
    /// Returns whether the hosting is ready to use.
    pub fn is_active(&self) -> bool {
        self.status.trim().eq_ignore_ascii_case(STATUS_ACTIVE)
    }

    /// Returns whether provisioning failed.
    pub fn is_failed(&self) -> bool {
        self.status.trim().eq_ignore_ascii_case(STATUS_FAILED)
    }
}

// ------------------------------------------------------------------------------------------------ requests

/// Lists hosting packages. CLI: `dna hosting packages`
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct HostingPackagesRequest {}

impl HostingPackagesRequest {
    /// Creates the request.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Request for HostingPackagesRequest {
    type Response = Vec<HostingPackage>;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/hostings/packages")
    }
}

/// Lists service plans and their limits. CLI: `dna hosting plans`
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct HostingPlansRequest {
    /// Limits the list to the plans available to one hosting ([`Hosting::hosting_id`]); empty for all.
    pub hosting_id: String,
}

impl HostingPlansRequest {
    /// Lists every plan.
    pub fn new() -> Self {
        Self::default()
    }

    /// Limits the list to the plans available to one hosting.
    pub fn hosting_id(mut self, hosting_id: impl Into<String>) -> Self {
        self.hosting_id = hosting_id.into();
        self
    }
}

impl Request for HostingPlansRequest {
    type Response = Vec<ServicePlan>;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/hostings/plans")
    }

    fn query(&self) -> Query {
        Query::new().set_str("hostingId", &self.hosting_id)
    }
}

/// Orders a hosting for a domain. It charges your balance. CLI: `dna hosting create`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct HostingCreateRequest {
    /// The domain.
    pub domain_name: String,
    /// The service plan, from [`HostingPackage::service_plans`].
    pub service_plan: String,
}

impl HostingCreateRequest {
    /// Orders `service_plan` for `domain_name`.
    pub fn new(domain_name: impl Into<String>, service_plan: impl Into<String>) -> Self {
        HostingCreateRequest { domain_name: domain_name.into(), service_plan: service_plan.into() }
    }
}

impl Request for HostingCreateRequest {
    type Response = HostingCreateResult;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/hostings/create", OperationSafety::Billable)
    }

    fn body(&self) -> Option<Value> {
        Some(
            JsonBody::new()
                .insert("domainName", self.domain_name.trim())
                .insert("servicePlan", self.service_plan.trim())
                .into_value(),
        )
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        v.required("servicePlan", &self.service_plan);
    }
}

/// Which hosting a [`HostingGetRequest`] or [`HostingStatusRequest`] reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostingRef {
    /// By service ID (`"HST-..."`), the preferred way.
    ServiceId(String),
    /// By domain name.
    DomainName(String),
}

impl HostingRef {
    fn validate(&self, v: &mut Validator) {
        match self {
            HostingRef::ServiceId(id) => v.required("serviceId", id),
            HostingRef::DomainName(name) => v.required("domainName", name),
        }
    }
}

/// Reads a hosting by service ID or domain name. CLI: `dna hosting get`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct HostingGetRequest {
    /// The hosting.
    pub hosting: HostingRef,
}

impl HostingGetRequest {
    /// Reads the hosting with this service ID.
    pub fn by_service_id(service_id: impl Into<String>) -> Self {
        HostingGetRequest { hosting: HostingRef::ServiceId(service_id.into()) }
    }

    /// Reads the hosting of this domain.
    pub fn by_domain(domain_name: impl Into<String>) -> Self {
        HostingGetRequest { hosting: HostingRef::DomainName(domain_name.into()) }
    }
}

impl Request for HostingGetRequest {
    type Response = Hosting;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/hostings")
    }

    fn query(&self) -> Query {
        match &self.hosting {
            HostingRef::ServiceId(id) => Query::new().set_str("serviceId", id),
            HostingRef::DomainName(name) => Query::new().set_str("domainName", name),
        }
    }

    fn validate(&self, v: &mut Validator) {
        self.hosting.validate(v);
    }
}

/// Reads a hosting by the token [`HostingClient::create`] returned. CLI: `dna hosting get-by-token`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct HostingGetByTokenRequest {
    /// The token.
    pub token: String,
}

impl HostingGetByTokenRequest {
    /// Reads the hosting of this token.
    pub fn new(token: impl Into<String>) -> Self {
        HostingGetByTokenRequest { token: token.into() }
    }
}

impl Request for HostingGetByTokenRequest {
    type Response = Hosting;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/hostings/by-token")
    }

    fn query(&self) -> Query {
        Query::new().set_str("token", &self.token)
    }

    fn validate(&self, v: &mut Validator) {
        v.required("token", &self.token);
    }
}

/// Reads the provisioning state of a hosting. CLI: `dna hosting status`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct HostingStatusRequest {
    /// The hosting.
    pub hosting: HostingRef,
}

impl HostingStatusRequest {
    /// Reads the state of the hosting with this service ID.
    pub fn by_service_id(service_id: impl Into<String>) -> Self {
        HostingStatusRequest { hosting: HostingRef::ServiceId(service_id.into()) }
    }

    /// Reads the state of the hosting of this domain.
    pub fn by_domain(domain_name: impl Into<String>) -> Self {
        HostingStatusRequest { hosting: HostingRef::DomainName(domain_name.into()) }
    }
}

impl Request for HostingStatusRequest {
    type Response = HostingStatus;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/hostings/check-status")
    }

    fn query(&self) -> Query {
        match &self.hosting {
            HostingRef::ServiceId(id) => Query::new().set_str("ServiceId", id),
            HostingRef::DomainName(name) => Query::new().set_str("DomainName", name),
        }
    }

    fn validate(&self, v: &mut Validator) {
        self.hosting.validate(v);
    }
}

// ------------------------------------------------------------------------------------------------ client

/// Web hosting. Get it with [`Client::hosting`].
#[derive(Debug, Clone, Copy)]
pub struct HostingClient<'a> {
    client: &'a Client,
}

impl<'a> HostingClient<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        HostingClient { client }
    }

    /// Returns the hosting packages and their service plans.
    pub async fn packages(&self) -> Result<Vec<HostingPackage>, Error> {
        self.client.send(&HostingPackagesRequest::new()).await
    }

    /// Returns the service plans and their limits; `hosting_id` limits them to one hosting.
    pub async fn plans(&self, hosting_id: Option<&str>) -> Result<Vec<ServicePlan>, Error> {
        self.client.send(&HostingPlansRequest::new().hosting_id(hosting_id.unwrap_or_default())).await
    }

    /// Orders a hosting and charges your balance. Provisioning continues after the call returns: poll
    /// [`status`](HostingClient::status) with the result's service ID until it is active.
    pub async fn create(&self, domain_name: &str, service_plan: &str) -> Result<HostingCreateResult, Error> {
        self.client.send(&HostingCreateRequest::new(domain_name, service_plan)).await
    }

    /// Returns a hosting by service ID or domain name.
    pub async fn get(&self, request: HostingGetRequest) -> Result<Hosting, Error> {
        self.client.send(&request).await
    }

    /// Returns a hosting by the token [`create`](HostingClient::create) returned.
    pub async fn get_by_token(&self, token: &str) -> Result<Hosting, Error> {
        self.client.send(&HostingGetByTokenRequest::new(token)).await
    }

    /// Returns the provisioning state of a hosting.
    pub async fn status(&self, request: HostingStatusRequest) -> Result<HostingStatus, Error> {
        self.client.send(&request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn plans_keep_unlimited_as_none() {
        let plan: ServicePlan =
            serde_json::from_value(json!({"name": "Starter", "diskQuotaMb": 1024, "maxDomains": null})).unwrap();
        assert_eq!(plan.disk_quota_mb, Some(1024));
        assert_eq!(plan.max_domains, None);
    }

    #[test]
    fn get_and_status_use_their_parameter_names() {
        assert_eq!(HostingGetRequest::by_service_id("HST-1").query().pairs()[0].0, "serviceId");
        assert_eq!(HostingStatusRequest::by_domain("example.com").query().pairs()[0].0, "DomainName");
        let mut v = Validator::new();
        HostingStatusRequest::by_service_id(" ").validate(&mut v);
        assert_eq!(v.errors()[0].field, "serviceId");
        let status: HostingStatus = serde_json::from_value(json!({"status": "active"})).unwrap();
        assert!(status.is_active());
    }
}
