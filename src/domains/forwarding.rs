//! URL forwarding of domains. CLI: `dna domain forward`.

use super::domain_only_request;
use crate::de::{model, str_enum};
use crate::{codes, Client, Empty, Endpoint, Error, JsonBody, OperationSafety, Query, Request, Validator};
use serde_json::Value;

str_enum! {
    /// How a domain is forwarded.
    pub enum ForwardType {
        /// An HTTP 301 redirect (the default).
        Standard => "Standard" ["redirect", "301"],
        /// Shows the target in a frame (masked URL).
        Frame => "Frame" ["masked"],
    }
}

model! {
    /// The URL forwarding of a domain.
    pub struct Forwarding {
        /// The target address.
        "redirectAlias" => pub url: String,
        /// Redirect or frame.
        "redirectType" => pub forward_type: ForwardType,
    }
}

domain_only_request!(
    /// Reads the URL forwarding of a domain. CLI: `dna domain forward get`
    DomainForwardGetRequest,
    Forwarding,
    Endpoint::get("/api/v1/domains/forwards"),
    query_lower
);

domain_only_request!(
    /// Removes the URL forwarding of a domain. CLI: `dna domain forward delete`
    DomainForwardDeleteRequest,
    Empty,
    Endpoint::delete("/api/v1/domains/forwards", OperationSafety::Destructive),
    query
);

/// Forwards a domain to a URL. CLI: `dna domain forward set`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DomainForwardSetRequest {
    /// The domain.
    pub domain_name: String,
    /// The target, for example `"https://www.example.org"`.
    pub url: String,
    /// [`ForwardType::Standard`] (a 301 redirect) or [`ForwardType::Frame`].
    pub forward_type: ForwardType,
}

impl DomainForwardSetRequest {
    /// Forwards `domain_name` to `url` with `forward_type`.
    pub fn new(domain_name: impl Into<String>, url: impl Into<String>, forward_type: ForwardType) -> Self {
        DomainForwardSetRequest { domain_name: domain_name.into(), url: url.into(), forward_type }
    }
}

impl Request for DomainForwardSetRequest {
    type Response = Empty;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/domains/forwards", OperationSafety::Write)
    }

    fn body(&self) -> Option<Value> {
        Some(
            JsonBody::new()
                .insert("domainName", self.domain_name.trim())
                .insert("redirectAddress", self.url.trim())
                .insert("forwardType", self.forward_type.as_str())
                .into_value(),
        )
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        v.required("url", &self.url);
        let url = self.url.trim();
        if !url.is_empty() {
            let ok =
                reqwest::Url::parse(url).is_ok_and(|u| matches!(u.scheme(), "http" | "https") && u.host().is_some());
            v.check(ok, "url", "must be an http:// or https:// URL");
        }
        v.check(self.forward_type.is_known(), "forwardType", "must be Standard or Frame");
    }
}

/// URL forwarding. Get it with [`DomainsClient::forwarding`](super::DomainsClient::forwarding).
#[derive(Debug, Clone, Copy)]
pub struct ForwardingClient<'a> {
    client: &'a Client,
}

impl<'a> ForwardingClient<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        ForwardingClient { client }
    }

    /// Returns the URL forwarding of a domain, or `None` when the domain has none (the API answers
    /// [`codes::FORWARDING_NOT_FOUND`] for that).
    pub async fn get(&self, domain_name: &str) -> Result<Option<Forwarding>, Error> {
        match self.client.send(&DomainForwardGetRequest::new(domain_name)).await {
            Ok(forwarding) if forwarding.url.trim().is_empty() => Ok(None),
            Ok(forwarding) => Ok(Some(forwarding)),
            Err(e) if e.code() == codes::FORWARDING_NOT_FOUND => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Forwards a domain to `url` with a redirect ([`ForwardType::Standard`]) or a frame ([`ForwardType::Frame`]).
    pub async fn set(&self, domain_name: &str, url: &str, forward_type: ForwardType) -> Result<(), Error> {
        self.client.send(&DomainForwardSetRequest::new(domain_name, url, forward_type)).await.map(|_| ())
    }

    /// Removes the URL forwarding of a domain.
    pub async fn delete(&self, domain_name: &str) -> Result<(), Error> {
        self.client.send(&DomainForwardDeleteRequest::new(domain_name)).await.map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_validates_the_url() {
        let mut v = Validator::new();
        DomainForwardSetRequest::new("example.com", "ftp://x", ForwardType::Other("Bounce".into())).validate(&mut v);
        assert_eq!(v.errors().len(), 2);
        let mut v = Validator::new();
        DomainForwardSetRequest::new("example.com", "https://www.example.org", ForwardType::Frame).validate(&mut v);
        assert!(v.is_valid());
    }

    #[test]
    fn queries_use_the_documented_case() {
        assert_eq!(DomainForwardGetRequest::new("example.com").query().pairs()[0].0, "domainName");
        assert_eq!(DomainForwardDeleteRequest::new("example.com").query().pairs()[0].0, "DomainName");
    }
}
