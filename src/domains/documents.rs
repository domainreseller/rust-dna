//! Registry documents of domains (mostly `.tr`). CLI: `dna domain documents`.

use crate::de::{int_enum, model};
use crate::request::paged;
use crate::{Client, Endpoint, Error, ItemStream, JsonBody, OperationSafety, Page, Paging, Query, Request, Validator};
use base64::Engine;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::path::Path;

/// The largest file, in bytes (10 MB).
pub const MAX_DOCUMENT_SIZE: usize = 10 << 20;
/// The largest upload, in bytes (50 MB across all files).
pub const MAX_DOCUMENT_UPLOAD_SIZE: usize = 50 << 20;
/// The accepted file extensions. The extension must match the file's real format.
pub const DOCUMENT_EXTENSIONS: &[&str] = &["pdf", "jpg", "jpeg", "png", "doc", "docx", "odt"];

int_enum! {
    /// The kind of a registry document.
    pub enum DocumentType {
        /// An identity card.
        IdentityCard = 1 => "IdentityCard",
        /// A passport.
        Passport = 2 => "Passport",
        /// A tax certificate.
        TaxCertificate = 3 => "TaxCertificate",
        /// An activity certificate.
        ActivityCertificate = 4 => "ActivityCertificate",
        /// A signature circular.
        SignatureCircular = 5 => "SignatureCircular",
        /// An authorization letter.
        AuthorizationLetter = 6 => "AuthorizationLetter",
        /// A petition.
        Petition = 7 => "Petition",
        /// A trade registry gazette.
        TradeRegistryGazette = 8 => "TradeRegistryGazette",
        /// A trademark certificate.
        TrademarkCertificate = 9 => "TrademarkCertificate",
        /// An education certificate (`.edu.tr`).
        EducationCertificate = 10 => "EducationCertificate",
        /// An organization statute (`.org.tr`).
        OrganizationStatute = 11 => "OrganizationStatute",
        /// A government approval (`.gov.tr`, `.bank`, `.mil.tr`).
        GovernmentApproval = 12 => "GovernmentApproval",
        /// A domain ownership proof.
        DomainOwnershipProof = 13 => "DomainOwnershipProof",
        /// A power of attorney.
        PowerOfAttorney = 14 => "PowerOfAttorney",
        /// A bar association certificate (`.av.tr`).
        BarAssociationCertificate = 15 => "BarAssociationCertificate",
        /// A medical diploma or license (`.dr.tr`).
        MedicalDiplomaOrLicense = 16 => "MedicalDiplomaOrLicense",
        /// A municipal official letter (`.bel.tr`).
        MunicipalOfficialLetter = 17 => "MunicipalOfficialLetter",
        /// An education institution license (`.k12.tr`, `.edu.tr`).
        EducationInstitutionLicense = 18 => "EducationInstitutionLicense",
        /// A chamber of commerce certificate.
        ChamberOfCommerceCertificate = 19 => "ChamberOfCommerceCertificate",
        /// A trademark transfer agreement.
        TrademarkTransferAgreement = 20 => "TrademarkTransferAgreement",
        /// A KEP authorization (`.kep.tr`).
        KepAuthorization = 21 => "KepAuthorization",
        /// Anything else.
        Other = 99 => "Other",
    }
}

int_enum! {
    /// The review state of a document.
    pub enum DocumentStatus {
        /// Expected, not uploaded.
        Waiting = 1 => "Waiting",
        /// Uploaded, not reviewed yet.
        Pending = 2 => "Pending",
        /// Approved.
        Approved = 3 => "Approved",
        /// Rejected.
        Rejected = 4 => "Rejected",
        /// Expired.
        Expired = 5 => "Expired",
        /// The application or owner change was cancelled.
        Cancelled = 6 => "Cancelled",
    }
}

int_enum! {
    /// What a document is for.
    pub enum DocumentOperationType {
        /// A new registration.
        Create = 1 => "Create",
        /// A registrant change.
        OwnerChange = 2 => "OwnerChange",
        /// A deletion.
        Delete = 3 => "Delete",
    }
}

model! {
    /// A registry document of a domain.
    pub struct Document {
        /// The file name.
        "name" => pub name: String,
        /// The kind of document.
        "type" => pub document_type: DocumentType,
        /// The review state.
        "status" => pub status: DocumentStatus,
        /// What the document is for.
        "operationType" => pub operation_type: DocumentOperationType,
        /// When it was uploaded.
        "uploadedDate" => pub uploaded_date: Option<DateTime<Utc>>,
    }
}

model! {
    /// A file stored by an upload.
    pub struct UploadedDocument {
        /// The file name.
        "name" => pub name: String,
        /// The size in bytes.
        "size" => pub size: i64,
        /// The media type.
        "mimeType" => pub mime_type: String,
        /// The kind of document.
        "type" => pub document_type: DocumentType,
        /// When it was uploaded.
        "uploadedDate" => pub uploaded_date: Option<DateTime<Utc>>,
    }
}

model! {
    /// The answer to [`DomainDocumentsUploadRequest`].
    pub struct DocumentUploadResult {
        /// The files stored.
        "uploaded" => pub uploaded: Vec<UploadedDocument>,
        /// File names that were already stored.
        "duplicates" => pub duplicates: Vec<String>,
        /// File names that could not be stored.
        "failed" => pub failed: Vec<String>,
        /// The number of documents of the domain.
        "totalDocumentCount" => pub total_document_count: i32,
        /// The size of all documents of the domain, in bytes.
        "totalDocumentSize" => pub total_document_size: i64,
    }
}

/// A file to upload as a registry document.
#[derive(Clone, PartialEq, Eq)]
pub struct DocumentUpload {
    /// The kind of document.
    pub document_type: DocumentType,
    /// The file name; it must end in one of [`DOCUMENT_EXTENSIONS`].
    pub file_name: String,
    /// The file content (at most [`MAX_DOCUMENT_SIZE`] bytes); it is base64-encoded for you.
    pub content: Vec<u8>,
}

impl std::fmt::Debug for DocumentUpload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocumentUpload")
            .field("document_type", &self.document_type)
            .field("file_name", &self.file_name)
            .field("content", &format_args!("{} bytes", self.content.len()))
            .finish()
    }
}

impl DocumentUpload {
    /// A document from bytes in memory.
    pub fn new(document_type: DocumentType, file_name: impl Into<String>, content: impl Into<Vec<u8>>) -> Self {
        DocumentUpload { document_type, file_name: file_name.into(), content: content.into() }
    }

    /// Reads a file. The file name is taken from the path.
    pub fn from_file(document_type: DocumentType, path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read(path)?;
        let file_name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        Ok(DocumentUpload { document_type, file_name, content })
    }
}

/// Lists the registry documents of a domain. CLI: `dna domain documents list`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DomainDocumentsListRequest {
    /// The domain.
    pub domain_name: String,
    /// Keeps only these kinds of document.
    pub document_types: Vec<DocumentType>,
    /// The page.
    pub paging: Paging,
}

impl DomainDocumentsListRequest {
    /// Lists the documents of `domain_name`.
    pub fn new(domain_name: impl Into<String>) -> Self {
        DomainDocumentsListRequest {
            domain_name: domain_name.into(),
            document_types: Vec::new(),
            paging: Paging::default(),
        }
    }

    /// Keeps only these kinds of document.
    pub fn document_types(mut self, types: impl IntoIterator<Item = DocumentType>) -> Self {
        self.document_types = types.into_iter().collect();
        self
    }
}

paged!(DomainDocumentsListRequest => Document, sortable);

impl Request for DomainDocumentsListRequest {
    type Response = Page<Document>;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/domains/documents")
    }

    fn query(&self) -> Query {
        let query = Query::new()
            .set_str("DomainName", &self.domain_name)
            .set_joined("DocumentTypes", self.document_types.iter().map(|t| t.code()));
        self.paging.apply(query)
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        self.paging.validate(v);
    }
}

/// Uploads the documents a registry asked for (domain status
/// [`DomainStatus::WaitingForDocument`](super::DomainStatus::WaitingForDocument)). CLI: `dna domain documents upload`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DomainDocumentsUploadRequest {
    /// The domain.
    pub domain_name: String,
    /// The files.
    pub documents: Vec<DocumentUpload>,
}

impl DomainDocumentsUploadRequest {
    /// Uploads `documents` for `domain_name`.
    pub fn new(domain_name: impl Into<String>, documents: impl IntoIterator<Item = DocumentUpload>) -> Self {
        DomainDocumentsUploadRequest { domain_name: domain_name.into(), documents: documents.into_iter().collect() }
    }
}

impl Request for DomainDocumentsUploadRequest {
    type Response = DocumentUploadResult;

    fn endpoint(&self) -> Endpoint {
        Endpoint::post("/api/v1/domains/documents", OperationSafety::Write)
    }

    fn body(&self) -> Option<Value> {
        let documents: Vec<Value> = self
            .documents
            .iter()
            .map(|d| {
                JsonBody::new()
                    .insert("documentType", d.document_type)
                    .insert("fileName", d.file_name.trim())
                    .insert("fileContent", base64::engine::general_purpose::STANDARD.encode(&d.content))
                    .into_value()
            })
            .collect();
        Some(JsonBody::new().insert("domainName", self.domain_name.trim()).insert("documents", documents).into_value())
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
        v.required_list("documents", self.documents.len());
        let mut total = 0;
        for (i, d) in self.documents.iter().enumerate() {
            let field = format!("documents[{i}]");
            v.check(d.document_type.is_known(), format!("{field}.documentType"), "is not a known DocumentType");
            v.required(format!("{field}.fileName"), &d.file_name);
            let extension = Path::new(d.file_name.trim())
                .extension()
                .map(|e| e.to_string_lossy().to_ascii_lowercase())
                .unwrap_or_default();
            v.check(
                d.file_name.trim().is_empty() || DOCUMENT_EXTENSIONS.contains(&extension.as_str()),
                format!("{field}.fileName"),
                "must be a PDF, JPG, JPEG, PNG, DOC, DOCX or ODT file",
            );
            v.check(!d.content.is_empty(), format!("{field}.content"), "is empty");
            v.check(d.content.len() <= MAX_DOCUMENT_SIZE, format!("{field}.content"), "is larger than 10 MB");
            total += d.content.len();
        }
        v.check(
            total <= MAX_DOCUMENT_UPLOAD_SIZE,
            "documents",
            "are larger than 50 MB together; upload them in several requests",
        );
    }
}

/// Registry documents. Get it with [`DomainsClient::documents`](super::DomainsClient::documents).
#[derive(Debug, Clone, Copy)]
pub struct DocumentsClient<'a> {
    client: &'a Client,
}

impl<'a> DocumentsClient<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        DocumentsClient { client }
    }

    /// Returns one page of a domain's registry documents.
    pub async fn list(&self, request: DomainDocumentsListRequest) -> Result<Page<Document>, Error> {
        self.client.send(&request).await
    }

    /// Returns every document of a domain, fetching pages as the stream is read.
    pub fn list_all(&self, domain_name: &str) -> ItemStream<Document> {
        self.client.list_all(DomainDocumentsListRequest::new(domain_name))
    }

    /// Uploads registry documents for a domain.
    ///
    /// ```no_run
    /// use domainnameapi::domains::{DocumentType, DocumentUpload};
    ///
    /// # async fn run(client: domainnameapi::Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let document = DocumentUpload::from_file(DocumentType::IdentityCard, "id.pdf")?;
    /// let result = client.domains().documents().upload("example.com.tr", [document]).await?;
    /// println!("{} uploaded", result.uploaded.len());
    /// # Ok(()) }
    /// ```
    pub async fn upload(
        &self,
        domain_name: &str,
        documents: impl IntoIterator<Item = DocumentUpload>,
    ) -> Result<DocumentUploadResult, Error> {
        self.client.send(&DomainDocumentsUploadRequest::new(domain_name, documents)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn uploads_are_base64_encoded() {
        let body = DomainDocumentsUploadRequest::new(
            "example.com.tr",
            [DocumentUpload::new(DocumentType::Passport, "p.pdf", b"%PDF".to_vec())],
        )
        .body()
        .unwrap();
        assert_eq!(
            body,
            json!({"domainName": "example.com.tr", "documents": [{"documentType": 2, "fileName": "p.pdf", "fileContent": "JVBERg=="}]})
        );
    }

    #[test]
    fn uploads_are_checked() {
        let mut v = Validator::new();
        DomainDocumentsUploadRequest::new(
            "example.com.tr",
            [DocumentUpload::new(DocumentType::Unknown(50), "p.exe", Vec::new())],
        )
        .validate(&mut v);
        assert_eq!(v.errors().len(), 3);
        assert!(format!("{:?}", DocumentUpload::new(DocumentType::Passport, "a.pdf", vec![1, 2])).contains("2 bytes"));
    }
}
