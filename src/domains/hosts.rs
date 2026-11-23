//! Child nameservers (glue records) such as `ns1.example.com`. CLI: `dna domain host`.

use crate::request::trim_all;
use crate::{Client, Empty, Endpoint, Error, JsonBody, OperationSafety, Query, Request, Validator};
use serde_json::Value;
use std::net::IpAddr;

/// The addresses as the API takes them, each with its detected version (`v4` or `v6`).
fn ip_addresses(list: &[String]) -> Vec<Value> {
    trim_all(list)
        .into_iter()
        .map(|address| {
            let version = match address.parse::<IpAddr>() {
                Ok(IpAddr::V4(_)) => "v4",
                _ => "v6",
            };
            JsonBody::new().insert("ipAddress", address).insert("ipVersion", version).into_value()
        })
        .collect()
}

fn validate_ips(v: &mut Validator, list: &[String]) {
    let ips = trim_all(list);
    v.required_list("ipAddresses", ips.len());
    for ip in ips {
        v.check(ip.parse::<IpAddr>().is_ok(), "ipAddresses", format!("{ip} is not an IP address"));
    }
}

/// Creates a child nameserver. CLI: `dna domain host add`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DomainHostAddRequest {
    /// The domain.
    pub domain_name: String,
    /// The full host name, for example `"ns1.example.com"`.
    pub host_name: String,
    /// IPv4 or IPv6 addresses; the version is detected.
    pub ip_addresses: Vec<String>,
}

impl DomainHostAddRequest {
    /// Creates `host_name` under `domain_name` with `ip_addresses`.
    pub fn new<S: Into<String>>(
        domain_name: impl Into<String>,
        host_name: impl Into<String>,
        ip_addresses: impl IntoIterator<Item = S>,
    ) -> Self {
        DomainHostAddRequest {
            domain_name: domain_name.into(),
            host_name: host_name.into(),
            ip_addresses: ip_addresses.into_iter().map(Into::into).collect(),
        }
    }
}

impl Request for DomainHostAddRequest {
    type Response = Empty;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/domains/dns/host", OperationSafety::Write)
    }

    fn body(&self) -> Option<Value> {
        Some(
            JsonBody::new()
                .insert("domainName", self.domain_name.trim())
                .insert("hostName", self.host_name.trim())
                .insert("ipAddresses", ip_addresses(&self.ip_addresses))
                .into_value(),
        )
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        v.required("hostName", &self.host_name);
        validate_ips(v, &self.ip_addresses);
    }
}

/// Changes a child nameserver's addresses, and optionally its name. CLI: `dna domain host update`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DomainHostUpdateRequest {
    /// The domain.
    pub domain_name: String,
    /// The existing host name.
    pub host_name: String,
    /// The complete new list of addresses.
    pub ip_addresses: Vec<String>,
    /// A new name for the host; empty keeps the name.
    pub new_host_name: String,
}

impl DomainHostUpdateRequest {
    /// Sets the addresses of `host_name` to `ip_addresses`.
    pub fn new<S: Into<String>>(
        domain_name: impl Into<String>,
        host_name: impl Into<String>,
        ip_addresses: impl IntoIterator<Item = S>,
    ) -> Self {
        DomainHostUpdateRequest {
            domain_name: domain_name.into(),
            host_name: host_name.into(),
            ip_addresses: ip_addresses.into_iter().map(Into::into).collect(),
            new_host_name: String::new(),
        }
    }

    /// Renames the host.
    pub fn new_host_name(mut self, name: impl Into<String>) -> Self {
        self.new_host_name = name.into();
        self
    }
}

impl Request for DomainHostUpdateRequest {
    type Response = Empty;

    fn endpoint(&self) -> Endpoint {
        Endpoint::put("/api/v1/domains/dns/host", OperationSafety::Write)
    }

    fn body(&self) -> Option<Value> {
        Some(
            JsonBody::new()
                .insert("domainName", self.domain_name.trim())
                .insert("hostName", self.host_name.trim())
                .insert_str("newHostName", &self.new_host_name)
                .insert("ipAddresses", ip_addresses(&self.ip_addresses))
                .into_value(),
        )
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        v.required("hostName", &self.host_name);
        validate_ips(v, &self.ip_addresses);
    }
}

/// Deletes a child nameserver. CLI: `dna domain host delete`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DomainHostDeleteRequest {
    /// The domain.
    pub domain_name: String,
    /// The host name.
    pub host_name: String,
}

impl DomainHostDeleteRequest {
    /// Deletes `host_name` under `domain_name`.
    pub fn new(domain_name: impl Into<String>, host_name: impl Into<String>) -> Self {
        DomainHostDeleteRequest { domain_name: domain_name.into(), host_name: host_name.into() }
    }
}

impl Request for DomainHostDeleteRequest {
    type Response = Empty;

    fn endpoint(&self) -> Endpoint {
        Endpoint::delete("/api/v1/domains/dns/host", OperationSafety::Destructive)
    }

    fn query(&self) -> Query {
        Query::new().set_str("DomainName", &self.domain_name).set_str("HostName", &self.host_name)
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        v.required("hostName", &self.host_name);
    }
}

/// Child nameservers. Get it with [`DomainsClient::child_name_servers`](super::DomainsClient::child_name_servers).
#[derive(Debug, Clone, Copy)]
pub struct ChildNameServersClient<'a> {
    client: &'a Client,
}

impl<'a> ChildNameServersClient<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        ChildNameServersClient { client }
    }

    /// Creates a child nameserver such as `ns1.example.com` with one or more IPv4/IPv6 addresses.
    pub async fn add<S: Into<String>>(
        &self,
        domain_name: &str,
        host_name: &str,
        ip_addresses: impl IntoIterator<Item = S>,
    ) -> Result<(), Error> {
        self.client.send(&DomainHostAddRequest::new(domain_name, host_name, ip_addresses)).await.map(|_| ())
    }

    /// Changes a child nameserver's addresses or name.
    pub async fn update(&self, request: DomainHostUpdateRequest) -> Result<(), Error> {
        self.client.send(&request).await.map(|_| ())
    }

    /// Deletes a child nameserver.
    pub async fn delete(&self, domain_name: &str, host_name: &str) -> Result<(), Error> {
        self.client.send(&DomainHostDeleteRequest::new(domain_name, host_name)).await.map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ip_versions_are_detected() {
        let body =
            DomainHostAddRequest::new("example.com", "ns1.example.com", ["192.0.2.1", "2001:db8::1"]).body().unwrap();
        assert_eq!(
            body["ipAddresses"],
            json!([{"ipAddress": "192.0.2.1", "ipVersion": "v4"}, {"ipAddress": "2001:db8::1", "ipVersion": "v6"}])
        );
    }

    #[test]
    fn bad_addresses_are_refused() {
        let mut v = Validator::new();
        DomainHostUpdateRequest::new("example.com", "ns1.example.com", ["300.1.1.1"]).validate(&mut v);
        assert_eq!(v.errors()[0].message, "300.1.1.1 is not an IP address");
    }
}
