//! The Rust SDK for the [Domain Name API](https://www.domainnameapi.com) reseller REST API.
//!
//! It covers all 67 operations of the API with typed requests and models: domains, contacts, DNS, DNSSEC, SSL
//! certificates, hosting, orders, products and your account. It is the Rust counterpart of
//! [`dna`, the Domain Name API CLI](https://github.com/domainreseller/cli): the same operations and groups, the same
//! safety classes, environment variables, retry rules and error codes.
//!
//! ```no_run
//! use domainnameapi::{Client, Environment};
//!
//! # async fn run() -> Result<(), domainnameapi::Error> {
//! let client = Client::builder()
//!     .credentials("your-reseller-id", "your-api-key")
//!     .environment(Environment::Ote) // the test environment
//!     .build()?;
//!
//! let account = client.account().get().await?;
//! println!("{}: {} USD", account.reseller_name, account.usd_balance);
//!
//! let search = client.domains().search("example.com").await?;
//! println!("{} {} {}", search.info.status, search.info.price, search.info.currency);
//! # Ok(()) }
//! ```
//!
//! # How it is organised
//!
//! - [`Client`] is the entry point. Its area clients mirror the CLI's command groups: [`Client::account`],
//!   [`Client::contacts`], [`Client::domains`] (with forwarding, child nameservers and documents), [`Client::dns`],
//!   [`Client::dnssec`], [`Client::ssl`], [`Client::hosting`], [`Client::orders`] and [`Client::products`].
//! - Every operation is a request type named after its CLI command (`dna domain renew-check` is
//!   [`domains::DomainRenewCheckRequest`]) that implements [`Request`]. The area methods build them for you; you can
//!   also build and [`send`](Client::send) them yourself, and implement [`Request`] for endpoints the SDK does not
//!   cover.
//! - Every request declares an [`OperationSafety`]: read, write, billable or destructive. Billable requests are never
//!   retried after a network error, and [read-only mode](ClientBuilder::read_only) blocks everything but reads.
//! - Every failure is an [`Error`] with a stable [`ErrorKind`], the API's error code, field-level problems, the
//!   correlation ID and a hint.
//! - Answers are typed models that are forgiving when reading (numbers as text, `1`/`"yes"` as booleans, dates with
//!   or without an offset, unknown enum values kept as `Unknown`). Requests are strict: they are checked before
//!   anything is sent.
//!
//! The SDK is async and runs on [Tokio](https://tokio.rs) through [`reqwest`] (rustls, no OpenSSL). There is no
//! telemetry: nothing leaves your process except the requests you send to the API.

#![deny(missing_docs)]

mod client;
mod de;
mod error;
mod request;

pub mod account;
pub mod contacts;
pub mod dns;
pub mod dnssec;
pub mod domains;
pub mod hosting;
pub mod orders;
pub mod products;
pub mod ssl;

pub use client::{Client, ClientBuilder, Environment, Quota, Response, DEFAULT_APP, DEFAULT_USER_AGENT, VERSION};
pub use error::{codes, Error, ErrorKind};
pub use request::{
    Empty, Endpoint, FieldError, JsonBody, Method, OperationSafety, Page, PagedRequest, Paging, Query, Request,
    Validator,
};

/// The stream type the `list_all` methods return. Read it with [`futures_util::TryStreamExt`] (or
/// `futures::TryStreamExt`), for example `stream.try_next().await?` or `stream.try_collect::<Vec<_>>().await?`.
pub type ItemStream<T> = futures_util::stream::BoxStream<'static, Result<T, Error>>;
