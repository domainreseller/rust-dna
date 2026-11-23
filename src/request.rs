//! The building blocks of a request: [`Request`], [`Endpoint`], [`OperationSafety`], [`Query`], [`JsonBody`],
//! [`Validator`], and paging with [`PagedRequest`] and [`Page`].

use crate::de::Lenient;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};
use std::fmt;

/// The HTTP method of a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    /// `GET`
    Get,
    /// `POST`
    Post,
    /// `PUT`
    Put,
    /// `DELETE`
    Delete,
    /// `PATCH`
    Patch,
}

impl Method {
    /// Returns the method name, for example `"POST"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Patch => "PATCH",
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What a request can do to your account. Every request declares one, so a harmless lookup can be told apart from a
/// call that spends money before it is sent. The classes are the same as in the `dna` CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationSafety {
    /// No side effects, even when the API uses POST or PUT (for example a domain search).
    Read,
    /// A free change that can be undone (DNS records, locks, contacts, nameservers).
    Write,
    /// Charges your balance: registrations, renewals, transfers, restores, SSL and hosting orders.
    Billable,
    /// Deletes something or is hard to undo: record deletion, DNSSEC removal, certificate revocation.
    Destructive,
}

impl OperationSafety {
    /// Returns the CLI's name: `"read"`, `"write"`, `"billable"` or `"destructive"`.
    pub fn as_str(self) -> &'static str {
        match self {
            OperationSafety::Read => "read",
            OperationSafety::Write => "write",
            OperationSafety::Billable => "billable",
            OperationSafety::Destructive => "destructive",
        }
    }
}

impl fmt::Display for OperationSafety {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Where a request goes and what kind of request it is.
///
/// ```
/// use domainnameapi::{Endpoint, Method, OperationSafety};
///
/// let endpoint = Endpoint::post("/api/v1/domains/renew", OperationSafety::Billable);
/// assert_eq!(endpoint.method, Method::Post);
/// assert!(!endpoint.is_retry_safe());
/// assert!(Endpoint::get("/api/v1/domains/info").is_retry_safe());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    /// The HTTP method.
    pub method: Method,
    /// The path below the base address, already escaped, for example `/api/v1/domains/info`.
    pub path: String,
    /// What the request can do to your account.
    pub safety: OperationSafety,
    /// Whether a request that is not a read may be sent again after a network error or a 502/503/504 answer, because
    /// sending it twice is harmless (setting the same nameservers twice gives the same result). Billable requests are
    /// never idempotent.
    pub idempotent: bool,
}

impl Endpoint {
    /// Creates an endpoint that is not idempotent.
    pub fn new(method: Method, path: impl Into<String>, safety: OperationSafety) -> Self {
        Endpoint { method, path: path.into(), safety, idempotent: false }
    }

    /// A `GET` endpoint. GET requests are reads.
    pub fn get(path: impl Into<String>) -> Self {
        Endpoint::new(Method::Get, path, OperationSafety::Read)
    }

    /// A `POST` endpoint.
    pub fn post(path: impl Into<String>, safety: OperationSafety) -> Self {
        Endpoint::new(Method::Post, path, safety)
    }

    /// A `PUT` endpoint.
    pub fn put(path: impl Into<String>, safety: OperationSafety) -> Self {
        Endpoint::new(Method::Put, path, safety)
    }

    /// A `DELETE` endpoint.
    pub fn delete(path: impl Into<String>, safety: OperationSafety) -> Self {
        Endpoint::new(Method::Delete, path, safety)
    }

    /// Marks the endpoint as safe to send again after a network error or a 5xx answer.
    pub fn idempotent(mut self) -> Self {
        self.idempotent = true;
        self
    }

    /// Returns whether the request may be repeated after a network error or a 502/503/504 answer: reads and
    /// idempotent writes.
    pub fn is_retry_safe(&self) -> bool {
        self.safety == OperationSafety::Read || self.idempotent
    }
}

impl fmt::Display for Endpoint {
    /// `METHOD path`, for example `POST /api/v1/domains/renew`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.method, self.path)
    }
}

/// One call to the API: the base of every request type.
///
/// Every `XxxRequest` type of this crate implements it, and so can your own types, to call an endpoint the SDK does
/// not cover yet. Only [`endpoint`](Request::endpoint) is required; [`query`](Request::query),
/// [`body`](Request::body) and [`validate`](Request::validate) have defaults (no query string, no body, nothing to
/// check). Send any request with [`Client::send`](crate::Client::send), which adds the credentials, validates the
/// request, honours read-only mode, paces and retries it according to its safety class and decodes the answer.
///
/// ```no_run
/// use domainnameapi::{Client, Endpoint, Query, Request, Validator};
/// use serde::Deserialize;
///
/// /// A request for an endpoint the SDK does not cover.
/// struct DomainNotesRequest {
///     domain_name: String,
/// }
///
/// #[derive(Deserialize)]
/// struct DomainNotes {
///     notes: Vec<String>,
/// }
///
/// impl Request for DomainNotesRequest {
///     type Response = DomainNotes;
///
///     fn endpoint(&self) -> Endpoint {
///         Endpoint::get("/api/v1/domains/notes")
///     }
///
///     fn query(&self) -> Query {
///         Query::new().set_str("DomainName", &self.domain_name)
///     }
///
///     fn validate(&self, v: &mut Validator) {
///         v.required("domainName", &self.domain_name);
///     }
/// }
///
/// # async fn run(client: Client) -> Result<(), domainnameapi::Error> {
/// let notes = client.send(&DomainNotesRequest { domain_name: "example.com".into() }).await?;
/// println!("{:?}", notes.notes);
/// # Ok(()) }
/// ```
pub trait Request {
    /// The type the answer is decoded into. Use [`Empty`] for operations that answer without a body.
    type Response: serde::de::DeserializeOwned;

    /// Returns the method, path, safety class and idempotency.
    fn endpoint(&self) -> Endpoint;

    /// Returns the query string parameters. None by default.
    fn query(&self) -> Query {
        Query::new()
    }

    /// Returns the JSON body. None by default.
    fn body(&self) -> Option<Value> {
        None
    }

    /// Checks the request before it is sent. Problems added to `v` stop the request with an
    /// [`ErrorKind::InvalidInput`](crate::ErrorKind::InvalidInput) error; nothing reaches the API. Checks nothing by
    /// default.
    fn validate(&self, v: &mut Validator) {
        let _ = v;
    }
}

/// The answer of an operation that returns no body. It accepts (and ignores) any body.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Empty;

impl<'de> Deserialize<'de> for Empty {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        serde::de::IgnoredAny::deserialize(deserializer)?;
        Ok(Empty)
    }
}

impl Serialize for Empty {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_unit()
    }
}

/// The paging of a list request: which items to skip, the page size and the sort order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Paging {
    /// How many items to skip (`SkipCount`); `None` starts at the first item.
    pub skip: Option<u32>,
    /// The page size (`MaxResultCount`); `None` uses the API's default of 10.
    pub limit: Option<u32>,
    /// A sort expression such as `"expirationDate desc"` (`Sorting`); `None` keeps the API's order.
    pub sort: Option<String>,
}

impl Paging {
    /// The page size [`Client::list_all`](crate::Client::list_all) uses when the request sets none.
    pub const DEFAULT_PAGE_SIZE: u32 = 100;

    /// Adds `SkipCount`, `MaxResultCount` and `Sorting` to a query string.
    pub fn apply(&self, query: Query) -> Query {
        query
            .set_opt("SkipCount", self.skip)
            .set_opt("MaxResultCount", self.limit)
            .set_str("Sorting", self.sort.as_deref().unwrap_or(""))
    }

    /// Checks that the page size is at least 1.
    pub fn validate(&self, v: &mut Validator) {
        v.check(self.limit != Some(0), "limit", "must be at least 1");
    }
}

/// A list request that returns one [`Page`] at a time. [`Client::list_all`](crate::Client::list_all) and the
/// `list_all` methods of the area clients use it to fetch every page.
pub trait PagedRequest<T>: Request<Response = Page<T>> + Clone {
    /// Returns the request's paging.
    fn paging(&self) -> &Paging;

    /// Returns the request's paging for changes.
    fn paging_mut(&mut self) -> &mut Paging;
}

/// One page of a list.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[non_exhaustive]
pub struct Page<T> {
    /// The items of this page.
    pub items: Vec<T>,
    /// The number of items across all pages.
    #[serde(rename = "totalCount")]
    pub total_count: i64,
}

impl<T> Page<T> {
    /// Creates a page, for example in tests of code that uses the SDK.
    pub fn new(items: Vec<T>, total_count: i64) -> Self {
        Page { items, total_count }
    }
}

impl<T> Default for Page<T> {
    fn default() -> Self {
        Page { items: Vec::new(), total_count: 0 }
    }
}

impl<T: Lenient> Lenient for Page<T> {
    fn from_lenient(value: Value) -> Option<Self> {
        let Value::Object(mut map) = value else { return None };
        let items = map.remove("items").and_then(Vec::<T>::from_lenient).unwrap_or_default();
        let total_count = map.remove("totalCount").and_then(i64::from_lenient).unwrap_or(items.len() as i64);
        Some(Page { items, total_count })
    }
}

impl<'de, T: Lenient> Deserialize<'de> for Page<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        Page::from_lenient(value).ok_or_else(|| serde::de::Error::custom("expected a page object"))
    }
}

/// A query string under construction. Values are trimmed; the `set_str` and `set_opt` forms leave out empty values,
/// so the API only sees what the caller set.
///
/// ```
/// use domainnameapi::Query;
///
/// let query = Query::new().set_str("DomainName", " example.com ").set_str("Sorting", "").set_opt("SkipCount", Some(10));
/// assert_eq!(query.pairs(), &[("DomainName".to_string(), "example.com".to_string()), ("SkipCount".to_string(), "10".to_string())]);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Query {
    pairs: Vec<(String, String)>,
}

impl Query {
    /// Creates an empty query string.
    pub fn new() -> Self {
        Query::default()
    }

    /// Adds a parameter, even when the value is empty.
    pub fn set(mut self, key: &str, value: impl fmt::Display) -> Self {
        self.pairs.push((key.to_string(), value.to_string().trim().to_string()));
        self
    }

    /// Adds a parameter unless the text is empty or blank.
    pub fn set_str(self, key: &str, value: &str) -> Self {
        if value.trim().is_empty() {
            self
        } else {
            self.set(key, value)
        }
    }

    /// Adds a parameter when the value is `Some`.
    pub fn set_opt<V: fmt::Display>(self, key: &str, value: Option<V>) -> Self {
        match value {
            Some(value) => self.set(key, value),
            None => self,
        }
    }

    /// Adds the key once per value: `TransactionTypes=1&TransactionTypes=2`. Blank values are left out.
    pub fn set_each<V: fmt::Display>(mut self, key: &str, values: impl IntoIterator<Item = V>) -> Self {
        for value in values {
            self = self.set_str(key, &value.to_string());
        }
        self
    }

    /// Adds the values as one comma-separated value: `StatusIds=0,7`. Nothing is added for an empty list.
    pub fn set_joined<V: fmt::Display>(self, key: &str, values: impl IntoIterator<Item = V>) -> Self {
        let parts: Vec<String> =
            values.into_iter().map(|v| v.to_string().trim().to_string()).filter(|v| !v.is_empty()).collect();
        if parts.is_empty() {
            self
        } else {
            self.set(key, parts.join(","))
        }
    }

    /// Returns the parameters in the order they were added.
    pub fn pairs(&self) -> &[(String, String)] {
        &self.pairs
    }

    /// Returns whether no parameter was added.
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }
}

/// A JSON object under construction for a request body. The `insert_str`, `insert_opt` and `insert_list` forms
/// leave out empty values.
///
/// ```
/// use domainnameapi::JsonBody;
///
/// let body = JsonBody::new()
///     .insert("domainName", "example.com")
///     .insert_str("currency", "")
///     .insert_opt("period", Some(2))
///     .into_value();
/// assert_eq!(body, serde_json::json!({"domainName": "example.com", "period": 2}));
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct JsonBody {
    map: Map<String, Value>,
}

impl JsonBody {
    /// Creates an empty object.
    pub fn new() -> Self {
        JsonBody::default()
    }

    /// Adds a property, whatever its value.
    pub fn insert(mut self, key: &str, value: impl Serialize) -> Self {
        let value = serde_json::to_value(value).unwrap_or(Value::Null);
        self.map.insert(key.to_string(), value);
        self
    }

    /// Adds a trimmed text property unless it is blank.
    pub fn insert_str(self, key: &str, value: &str) -> Self {
        let value = value.trim();
        if value.is_empty() {
            self
        } else {
            self.insert(key, value)
        }
    }

    /// Adds a property when the value is `Some`.
    pub fn insert_opt(self, key: &str, value: Option<impl Serialize>) -> Self {
        match value {
            Some(value) => self.insert(key, value),
            None => self,
        }
    }

    /// Adds a list property unless the list is empty.
    pub fn insert_list<V: Serialize>(self, key: &str, values: &[V]) -> Self {
        if values.is_empty() {
            self
        } else {
            self.insert(key, values)
        }
    }

    /// Returns the object as a JSON value.
    pub fn into_value(self) -> Value {
        Value::Object(self.map)
    }
}

impl From<JsonBody> for Value {
    fn from(body: JsonBody) -> Value {
        body.into_value()
    }
}

/// One problem with one input field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldError {
    /// The field, for example `"domainName"` or `"contacts.Registrant.email"`. Empty when the problem is not about
    /// one field.
    pub field: String,
    /// What is wrong.
    pub message: String,
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.field.is_empty() {
            f.write_str(&self.message)
        } else {
            write!(f, "{}: {}", self.field, self.message)
        }
    }
}

/// Collects the problems [`Request::validate`] finds.
#[derive(Debug, Clone, Default)]
pub struct Validator {
    errors: Vec<FieldError>,
}

impl Validator {
    /// Creates a validator without problems.
    pub fn new() -> Self {
        Validator::default()
    }

    /// Records a problem.
    pub fn add(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors.push(FieldError { field: field.into(), message: message.into() });
    }

    /// Records a problem unless `ok` is true.
    pub fn check(&mut self, ok: bool, field: impl Into<String>, message: impl Into<String>) {
        if !ok {
            self.add(field, message);
        }
    }

    /// Records "is required" when the text is empty or blank.
    pub fn required(&mut self, field: impl Into<String>, value: &str) {
        self.check(!value.trim().is_empty(), field, "is required");
    }

    /// Records "needs at least one value" when the list is empty.
    pub fn required_list(&mut self, field: impl Into<String>, len: usize) {
        self.check(len > 0, field, "needs at least one value");
    }

    /// Records a problem when an optional number is set and outside `min..=max`.
    pub fn between(&mut self, field: impl Into<String>, value: Option<u32>, min: u32, max: u32) {
        if let Some(n) = value {
            self.check((min..=max).contains(&n), field, format!("must be between {min} and {max}"));
        }
    }

    /// Records a problem when the text is longer than `max` characters.
    pub fn max_len(&mut self, field: impl Into<String>, value: &str, max: usize) {
        self.check(value.trim().chars().count() <= max, field, format!("must be at most {max} characters"));
    }

    /// Returns the problems found so far.
    pub fn errors(&self) -> &[FieldError] {
        &self.errors
    }

    /// Returns whether no problem was found.
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns the problems found.
    pub fn into_errors(self) -> Vec<FieldError> {
        self.errors
    }
}

// ------------------------------------------------------------------------------------------------ crate helpers

/// Trims every value and drops the blank ones.
pub(crate) fn trim_all<S: AsRef<str>>(values: &[S]) -> Vec<String> {
    values.iter().map(|v| v.as_ref().trim().to_string()).filter(|v| !v.is_empty()).collect()
}

/// Escapes one path segment, for example a contact handle.
pub(crate) fn path_segment(value: &str) -> String {
    let mut out = String::new();
    for byte in value.trim().bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// The body of the many requests that only name a domain.
pub(crate) fn domain_body(domain_name: &str) -> Value {
    JsonBody::new().insert("domainName", domain_name.trim()).into_value()
}

/// Implements [`PagedRequest`] and the `skip`, `limit` and `sort` setters for a request with a `paging` field.
macro_rules! paged {
    ($request:ty => $item:ty) => {
        impl $crate::PagedRequest<$item> for $request {
            fn paging(&self) -> &$crate::Paging {
                &self.paging
            }

            fn paging_mut(&mut self) -> &mut $crate::Paging {
                &mut self.paging
            }
        }

        impl $request {
            /// Sets how many items to skip (`SkipCount`).
            pub fn skip(mut self, skip: u32) -> Self {
                self.paging.skip = Some(skip);
                self
            }

            /// Sets the page size (`MaxResultCount`). The API uses 10 when it is not set.
            pub fn limit(mut self, limit: u32) -> Self {
                self.paging.limit = Some(limit);
                self
            }
        }
    };
    ($request:ty => $item:ty, sortable) => {
        $crate::request::paged!($request => $item);

        impl $request {
            /// Sets the sort expression (`Sorting`), for example `"expirationDate desc"`.
            pub fn sort(mut self, sort: impl Into<String>) -> Self {
                self.paging.sort = Some(sort.into());
                self
            }
        }
    };
}
pub(crate) use paged;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_segments_are_escaped() {
        assert_eq!(path_segment(" DNA-12345 "), "DNA-12345");
        assert_eq!(path_segment("a b/c"), "a%20b%2Fc");
        assert_eq!(path_segment("ç"), "%C3%A7");
    }

    #[test]
    fn query_skips_blank_values() {
        let q = Query::new()
            .set_str("A", " ")
            .set_opt::<u32>("B", None)
            .set_each("C", ["1", " ", "2"])
            .set_joined("D", [0, 7])
            .set_joined("E", Vec::<String>::new());
        let keys: Vec<_> = q.pairs().iter().map(|(k, v)| format!("{k}={v}")).collect();
        assert_eq!(keys, ["C=1", "C=2", "D=0,7"]);
    }

    #[test]
    fn validator_collects_problems() {
        let mut v = Validator::new();
        v.required("domainName", "  ");
        v.required_list("nameServers", 0);
        v.between("years", Some(11), 1, 10);
        v.between("years", None, 1, 10);
        v.max_len("name", "abcd", 3);
        assert_eq!(v.errors().len(), 4);
        assert_eq!(v.errors()[0].to_string(), "domainName: is required");
        assert_eq!(v.errors()[2].message, "must be between 1 and 10");
    }

    #[test]
    fn pages_are_forgiving() {
        let page: Page<i64> = serde_json::from_str(r#"{"items":[1,"2"],"totalCount":"5"}"#).unwrap();
        assert_eq!(page.items, vec![1, 2]);
        assert_eq!(page.total_count, 5);
        let page: Page<i64> = serde_json::from_str(r#"{"items":null}"#).unwrap();
        assert!(page.items.is_empty());
    }

    #[test]
    fn empty_accepts_anything() {
        serde_json::from_str::<Empty>("null").unwrap();
        serde_json::from_str::<Empty>(r#"{"a":1}"#).unwrap();
    }
}
