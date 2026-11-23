//! DNSSEC: publish and remove DS records at the registry. CLI: `dna dnssec`.

use crate::de::{int_enum, model};
use crate::request::domain_body;
use crate::{Client, Endpoint, Error, JsonBody, OperationSafety, Query, Request, Validator};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::fmt;
use std::str::FromStr;

int_enum! {
    /// The DNSSEC state of a domain.
    pub enum DnssecStatus {
        /// No DS record at the registry.
        Unsigned = 0 => "Unsigned",
        /// A DS record was sent and awaits confirmation.
        PendingEnable = 1 => "PendingEnable",
        /// At least one active DS record.
        Signed = 2 => "Signed",
        /// A DS removal was sent and awaits confirmation.
        PendingDisable = 3 => "PendingDisable",
        /// The last DNSSEC operation failed.
        Error = 4 => "Error",
    }
}

int_enum! {
    /// The kind of a DNSSEC operation.
    pub enum DnssecOperationType {
        /// A DS record was added.
        Add = 0 => "Add",
        /// A DS record was removed.
        Remove = 1 => "Remove",
        /// Every DS record was removed.
        RemoveAll = 2 => "RemoveAll",
        /// The records were synchronised.
        Sync = 3 => "Sync",
    }
}

int_enum! {
    /// The state of a DS record.
    pub enum DsRecordStatus {
        /// Sent to the registry.
        Pending = 0 => "Pending",
        /// Active.
        Active = 1 => "Active",
        /// Being removed.
        Removing = 2 => "Removing",
        /// Removed.
        Removed = 3 => "Removed",
        /// The registry refused it.
        Failed = 4 => "Failed",
    }
}

int_enum! {
    /// A DNSSEC signing algorithm (IANA number).
    pub enum DnssecAlgorithm {
        /// 1, deprecated.
        RsaMd5 = 1 => "RsaMd5",
        /// 3, deprecated.
        Dsa = 3 => "Dsa",
        /// 5.
        RsaSha1 = 5 => "RsaSha1",
        /// 6.
        DsaNsec3Sha1 = 6 => "DsaNsec3Sha1",
        /// 7.
        RsaSha1Nsec3Sha1 = 7 => "RsaSha1Nsec3Sha1",
        /// 8.
        RsaSha256 = 8 => "RsaSha256",
        /// 10.
        RsaSha512 = 10 => "RsaSha512",
        /// 12.
        EccGost = 12 => "EccGost",
        /// 13, the most common.
        EcdsaP256Sha256 = 13 => "EcdsaP256Sha256",
        /// 14.
        EcdsaP384Sha384 = 14 => "EcdsaP384Sha384",
        /// 15.
        Ed25519 = 15 => "Ed25519",
        /// 16.
        Ed448 = 16 => "Ed448",
    }
}

int_enum! {
    /// The digest type of a DS record (IANA number).
    pub enum DsDigestType {
        /// 1.
        Sha1 = 1 => "Sha1",
        /// 2, the most common.
        Sha256 = 2 => "Sha256",
        /// 3.
        Gost = 3 => "Gost",
        /// 4.
        Sha384 = 4 => "Sha384",
    }
}

model! {
    /// A DS record at the registry.
    pub struct DsRecord {
        /// The key tag.
        "keyTag" => pub key_tag: u32,
        /// The algorithm.
        "algorithm" => pub algorithm: DnssecAlgorithm,
        /// The digest type.
        "digestType" => pub digest_type: DsDigestType,
        /// The digest.
        "digest" => pub digest: String,
        /// The record's state.
        "status" => pub status: DsRecordStatus,
        /// Whether the algorithm is no longer recommended.
        "isAlgorithmNotRecommended" => pub is_algorithm_not_recommended: bool,
        /// When it was added.
        "createdAt" => pub created_at: Option<DateTime<Utc>>,
        /// When it became active.
        "activatedAt" => pub activated_at: Option<DateTime<Utc>>,
        /// When it was removed.
        "removedAt" => pub removed_at: Option<DateTime<Utc>>,
    }
}

model! {
    /// The DNSSEC state of a domain.
    pub struct DnssecInfo {
        /// The domain.
        "domainName" => pub domain_name: String,
        /// Whether the TLD supports DNSSEC.
        "isDnssecAvailable" => pub available: bool,
        /// The state.
        "status" => pub status: DnssecStatus,
        /// Whether the delegation is signed.
        "delegationSigned" => pub delegation_signed: bool,
        /// The registry's maximum signature lifetime.
        "maxSigLife" => pub max_sig_life: Option<i64>,
        /// When the state last changed.
        "lastChangedAt" => pub last_changed_at: Option<DateTime<Utc>>,
        /// The DS records.
        "dsRecords" => pub ds_records: Vec<DsRecord>,
        /// The algorithms the registry accepts.
        "allowedAlgorithms" => pub allowed_algorithms: Vec<DnssecAlgorithm>,
        /// The digest types the registry accepts.
        "allowedDigestTypes" => pub allowed_digest_types: Vec<DsDigestType>,
        /// The largest number of DS records.
        "maxDsRecordCount" => pub max_ds_record_count: i32,
    }
}

model! {
    /// The outcome of one DNSSEC operation.
    pub struct DnssecOperationResult {
        /// The key tag of the record.
        "keyTag" => pub key_tag: Option<u32>,
        /// The kind of operation.
        "operationType" => pub operation_type: DnssecOperationType,
        /// Whether the registry accepted the change.
        "isSuccess" => pub success: bool,
        /// Whether the registry accepted the change but has not confirmed it yet.
        "isPending" => pub pending: bool,
        /// The registry's message.
        "resultMessage" => pub message: String,
    }
}

model! {
    /// The answer to [`DnssecAddRequest`].
    pub struct DnssecOperationResults {
        /// One result per record.
        "items" => pub items: Vec<DnssecOperationResult>,
    }
}

/// A DS record to publish or remove, as your DNS provider shows it.
///
/// ```
/// use domainnameapi::dnssec::{DnssecAlgorithm, DsData, DsDigestType};
///
/// let ds: DsData = "2371 13 2 1F987CC6583E92DF0890718C42".parse()?;
/// assert_eq!(ds.algorithm, DnssecAlgorithm::EcdsaP256Sha256);
/// assert_eq!(ds.digest_type, DsDigestType::Sha256);
///
/// // A whole zone-file line works too.
/// let ds: DsData = "example.com. 3600 IN DS 2371 13 2 1F98 7CC6".parse()?;
/// assert_eq!(ds.digest, "1F987CC6");
/// # Ok::<(), domainnameapi::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DsData {
    /// The key tag, 0 to 65535.
    pub key_tag: u32,
    /// The algorithm.
    pub algorithm: DnssecAlgorithm,
    /// The digest type.
    pub digest_type: DsDigestType,
    /// The digest in hexadecimal.
    pub digest: String,
}

impl DsData {
    /// A DS record from its parts.
    pub fn new(key_tag: u32, algorithm: DnssecAlgorithm, digest_type: DsDigestType, digest: impl Into<String>) -> Self {
        DsData { key_tag, algorithm, digest_type, digest: digest.into() }
    }

    /// The digest without spaces, in upper case.
    fn clean_digest(&self) -> String {
        self.digest.split_whitespace().collect::<String>().to_ascii_uppercase()
    }

    fn validate(&self, v: &mut Validator, field: &str) {
        v.check(self.key_tag <= 65535, format!("{field}.keyTag"), "must be between 0 and 65535");
        v.check(
            (1..=255).contains(&self.algorithm.code()),
            format!("{field}.algorithm"),
            "is required (for example EcdsaP256Sha256)",
        );
        v.check(
            (1..=255).contains(&self.digest_type.code()),
            format!("{field}.digestType"),
            "is required (for example Sha256)",
        );
        v.required(format!("{field}.digest"), &self.digest);
        v.max_len(format!("{field}.digest"), &self.clean_digest(), 128);
    }
}

impl FromStr for DsData {
    type Err = Error;

    /// Reads a DS record in presentation format, `KEYTAG ALGORITHM DIGESTTYPE DIGEST`, or a zone-file line that
    /// contains one. Spaces inside the digest are removed.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let mut fields: Vec<&str> = text.split_whitespace().collect();
        if let Some(i) = fields.iter().position(|f| f.eq_ignore_ascii_case("DS")) {
            fields.drain(..=i);
        }
        let invalid = || {
            Error::invalid_input(
                "record",
                format!("'{text}' is not a DS record (want KEYTAG ALGORITHM DIGESTTYPE DIGEST)"),
            )
        };
        if fields.len() < 4 {
            return Err(invalid());
        }
        let key_tag = fields[0].parse::<u32>().map_err(|_| invalid())?;
        let algorithm = fields[1].parse::<DnssecAlgorithm>().map_err(|_| invalid())?;
        let digest_type = fields[2].parse::<DsDigestType>().map_err(|_| invalid())?;
        Ok(DsData { key_tag, algorithm, digest_type, digest: fields[3..].concat().to_ascii_uppercase() })
    }
}

impl fmt::Display for DsData {
    /// `2371 13 2 1F98...`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {} {}", self.key_tag, self.algorithm.code(), self.digest_type.code(), self.clean_digest())
    }
}

// ------------------------------------------------------------------------------------------------ requests

/// Reads the DNSSEC state of a domain. CLI: `dna dnssec get`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DnssecGetRequest {
    /// The domain.
    pub domain_name: String,
}

impl DnssecGetRequest {
    /// Reads the state of `domain_name`.
    pub fn new(domain_name: impl Into<String>) -> Self {
        DnssecGetRequest { domain_name: domain_name.into() }
    }
}

impl Request for DnssecGetRequest {
    type Response = DnssecInfo;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/domains/dns/dnssec")
    }

    fn query(&self) -> Query {
        Query::new().set_str("DomainName", &self.domain_name)
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
    }
}

/// Publishes DS records at the registry. CLI: `dna dnssec add`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DnssecAddRequest {
    /// The domain.
    pub domain_name: String,
    /// The records.
    pub records: Vec<DsData>,
}

impl DnssecAddRequest {
    /// Publishes `records` for `domain_name`.
    pub fn new(domain_name: impl Into<String>, records: impl IntoIterator<Item = DsData>) -> Self {
        DnssecAddRequest { domain_name: domain_name.into(), records: records.into_iter().collect() }
    }
}

impl Request for DnssecAddRequest {
    type Response = DnssecOperationResults;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/domains/dns/dnssec", OperationSafety::Write)
    }

    fn body(&self) -> Option<Value> {
        let records: Vec<Value> = self
            .records
            .iter()
            .map(|r| {
                JsonBody::new()
                    .insert("keyTag", r.key_tag)
                    .insert("algorithm", r.algorithm)
                    .insert("digestType", r.digest_type)
                    .insert("digest", r.clean_digest())
                    .into_value()
            })
            .collect();
        Some(JsonBody::new().insert("domainName", self.domain_name.trim()).insert("records", records).into_value())
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        v.required_list("records", self.records.len());
        for (i, record) in self.records.iter().enumerate() {
            record.validate(v, &format!("records[{i}]"));
        }
    }
}

/// Removes one DS record. CLI: `dna dnssec remove`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DnssecRemoveRequest {
    /// The domain.
    pub domain_name: String,
    /// The record.
    pub record: DsData,
}

impl DnssecRemoveRequest {
    /// Removes `record` from `domain_name`.
    pub fn new(domain_name: impl Into<String>, record: DsData) -> Self {
        DnssecRemoveRequest { domain_name: domain_name.into(), record }
    }
}

impl Request for DnssecRemoveRequest {
    type Response = DnssecOperationResult;

    fn endpoint(&self) -> Endpoint {
        Endpoint::delete("/api/v1/domains/dns/dnssec", OperationSafety::Destructive)
    }

    fn query(&self) -> Query {
        Query::new()
            .set_str("DomainName", &self.domain_name)
            .set("KeyTag", self.record.key_tag)
            .set("Algorithm", self.record.algorithm.code())
            .set("DigestType", self.record.digest_type.code())
            .set("Digest", self.record.clean_digest())
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        self.record.validate(v, "record");
    }
}

/// Removes every DS record of a domain. CLI: `dna dnssec disable`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DnssecDisableRequest {
    /// The domain.
    pub domain_name: String,
}

impl DnssecDisableRequest {
    /// Disables DNSSEC of `domain_name`.
    pub fn new(domain_name: impl Into<String>) -> Self {
        DnssecDisableRequest { domain_name: domain_name.into() }
    }
}

impl Request for DnssecDisableRequest {
    type Response = DnssecOperationResult;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/domains/dns/dnssec/disable", OperationSafety::Destructive)
    }

    fn body(&self) -> Option<Value> {
        Some(domain_body(&self.domain_name))
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
    }
}

// ------------------------------------------------------------------------------------------------ client

/// DNSSEC. Get it with [`Client::dnssec`].
#[derive(Debug, Clone, Copy)]
pub struct DnssecClient<'a> {
    client: &'a Client,
}

impl<'a> DnssecClient<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        DnssecClient { client }
    }

    /// Returns the DNSSEC state of a domain: status, DS records and the algorithms the registry accepts.
    pub async fn get(&self, domain_name: &str) -> Result<DnssecInfo, Error> {
        self.client.send(&DnssecGetRequest::new(domain_name)).await
    }

    /// Publishes DS records from your DNS provider. Wrong DS records make the domain unresolvable for validating
    /// resolvers: copy them exactly. Check `success` or `pending` of each result.
    pub async fn add(
        &self,
        domain_name: &str,
        records: impl IntoIterator<Item = DsData>,
    ) -> Result<Vec<DnssecOperationResult>, Error> {
        Ok(self.client.send(&DnssecAddRequest::new(domain_name, records)).await?.items)
    }

    /// Removes one DS record.
    pub async fn remove(&self, domain_name: &str, record: DsData) -> Result<DnssecOperationResult, Error> {
        self.client.send(&DnssecRemoveRequest::new(domain_name, record)).await
    }

    /// Removes every DS record of a domain.
    pub async fn disable(&self, domain_name: &str) -> Result<DnssecOperationResult, Error> {
        self.client.send(&DnssecDisableRequest::new(domain_name)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ds_records_parse_and_print() {
        let ds: DsData = "12345 ECDSAP256SHA256 sha256 49fd 46e6".parse().unwrap();
        assert_eq!(ds.to_string(), "12345 13 2 49FD46E6");
        assert!("12345 13".parse::<DsData>().is_err());
        assert!("x 13 2 AB".parse::<DsData>().is_err());
        assert!("1 99 2 AB".parse::<DsData>().is_err());
    }

    #[test]
    fn add_and_remove() {
        let ds = DsData::new(2371, DnssecAlgorithm::EcdsaP256Sha256, DsDigestType::Sha256, "1f 98");
        let body = DnssecAddRequest::new("example.com", [ds.clone()]).body().unwrap();
        assert_eq!(
            body,
            json!({"domainName": "example.com", "records": [{"keyTag": 2371, "algorithm": 13, "digestType": 2, "digest": "1F98"}]})
        );
        let q = DnssecRemoveRequest::new("example.com", ds).query();
        let pairs: Vec<String> = q.pairs().iter().map(|(k, v)| format!("{k}={v}")).collect();
        assert_eq!(pairs, ["DomainName=example.com", "KeyTag=2371", "Algorithm=13", "DigestType=2", "Digest=1F98"]);

        let mut v = Validator::new();
        DnssecAddRequest::new(
            "example.com",
            [DsData::new(70000, DnssecAlgorithm::Unknown(-1), DsDigestType::Sha1, "")],
        )
        .validate(&mut v);
        assert_eq!(v.errors().len(), 3);
    }

    #[test]
    fn info_reads_enums() {
        let info: DnssecInfo = serde_json::from_value(json!({
            "status": 2, "allowedAlgorithms": [8, 13, 99], "dsRecords": [{"keyTag": 1, "status": 1, "algorithm": 13}]
        }))
        .unwrap();
        assert_eq!(info.status, DnssecStatus::Signed);
        assert_eq!(info.allowed_algorithms[2], DnssecAlgorithm::Unknown(99));
        assert_eq!(info.ds_records[0].status, DsRecordStatus::Active);
    }
}
