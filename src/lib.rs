//! # domainnameapi
//!
//! Rust client for the [DomainNameAPI](https://www.domainnameapi.com) domain
//! registrar REST API. It is a faithful port of the reference PHP library
//! [php-dna](https://github.com/domainreseller/php-dna): the same 45 methods,
//! the same requests on the wire and the same return values.
//!
//! Methods return [`serde_json::Value`] with exactly the structure php-dna
//! returns (`{"result": "OK", "data": {...}}` or
//! `{"result": "ERROR", "error": {"Code", "Message", "Details"}}`). Errors are
//! returned, never thrown or panicked.
//!
//! ```no_run
//! use domainnameapi::DomainNameApi;
//!
//! let mut dna = DomainNameApi::new("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx", "your-api-key", false);
//! let details = dna.get_details("example.com");
//! if details["result"] == "OK" {
//!     println!("expires {}", details["data"]["Dates"]["Expiration"]);
//! } else {
//!     eprintln!("{}", details["error"]["Message"]);
//! }
//! ```
//!
//! ## Default arguments
//!
//! Rust has no default arguments, so every method takes php-dna's full
//! argument list in the same order. The php-dna defaults are exported as
//! constants ([`DEFAULT_TTL`], [`DEFAULT_REASON`], [`DEFAULT_FORWARD_TYPE`],
//! [`DEFAULT_CURRENCY`], [`DEFAULT_TLD_COUNT`], [`DEFAULT_COMMAND`]).
//! `register_with_contact_info` has four optional arguments; besides the
//! full-argument method there is
//! [`DomainNameApi::register_with_contact_info_opts`] taking a
//! [`RegisterOptions`] whose `Default` is php-dna's defaults.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod client;
mod contacts;
pub mod php;
pub mod telemetry;
pub mod transport;
mod zone;

pub use client::{BoolFlag, BoolFlag as TestModeFlag, DomainNameApi, RegisterOptions};
pub use serde_json::{json, Value};
pub use transport::{HttpRequest, HttpResponse, Transport, TransportError, UreqTransport};

/// Library version.
pub const VERSION: &str = "1.0.1";

/// Application name used in telemetry tags.
pub const APPLICATION: &str = "DomainNameAPI-Rust";

/// User-Agent sent with every request. Required: Cloudflare in front of the
/// gateway rejects default client agents with error 1010.
pub const USER_AGENT: &str = "DomainNameAPI-Rust/1.0.1";

/// Live gateway.
pub const URL_PROD: &str = "https://api.domainresellerapi.com/api/v1";

/// Test (OTE) gateway.
pub const URL_OTE: &str = "https://ote.domainresellerapi.com/api/v1";

/// `result` value of a successful call.
pub const RESULT_OK: &str = "OK";
/// `result` value of a failed call.
pub const RESULT_ERROR: &str = "ERROR";
/// Legacy success marker kept for parity with php-dna.
pub const RESULT_SUCCESS: &str = "SUCCESS";

/// Nameservers used when a registration supplies none.
pub const DEFAULT_NAMESERVERS: [&str; 2] = ["ns1.domainnameapi.com", "ns2.domainnameapi.com"];

/// Total request timeout in seconds.
pub const DEFAULT_TIMEOUT: u64 = 30;
/// Connect timeout in seconds.
pub const DEFAULT_CONNECT_TIMEOUT: u64 = 10;

/// php-dna default for `modify_privacy_protection_status`'s reason.
pub const DEFAULT_REASON: &str = "Owner request";
/// php-dna default TTL for the DNS zone methods.
pub const DEFAULT_TTL: i64 = 3600;
/// php-dna default forwarding type.
pub const DEFAULT_FORWARD_TYPE: &str = "Standard";
/// php-dna default currency for `get_current_balance`.
pub const DEFAULT_CURRENCY: &str = "USD";
/// php-dna default count for `get_tld_list`.
pub const DEFAULT_TLD_COUNT: i64 = 20;
/// php-dna default command for `check_availability`.
pub const DEFAULT_COMMAND: &str = "create";

/// Zone record types the gateway accepts.
pub const ZONE_RECORD_TYPES: [&str; 12] = [
    "A", "AAAA", "ALIAS", "CAA", "CNAME", "DS", "MX", "NAPTR", "PTR", "SRV", "TLSA", "TXT",
];

/// Redirect types accepted by the forwarding endpoint.
pub const FORWARD_TYPES: [&str; 2] = ["Standard", "Frame"];

/// Minimum gap between two `domains/zones*` calls, in seconds.
pub const ZONE_MIN_INTERVAL: f64 = 1.0;

/// Predefined error texts (php-dna `DEFAULT_ERRORS`): `(code, message, description)`.
pub const DEFAULT_ERRORS: [(&str, &str, &str); 15] = [
    ("DOMAIN_DETAILS", "Invalid domain details! Details format is not valid", "The provided domain details are not in the expected format"),
    ("CREDENTIALS", "Invalid reseller ID or API key", "The provided API credentials are invalid"),
    ("DOMAIN_LIST", "The domain list is invalid or contains multiple entries!", "The domain list response format is incorrect or unexpected"),
    ("TLD_LIST", "TLD info is not a valid array or more than one TLD info has returned!", "The TLD list response is not in the expected format"),
    ("RESPONSE", "Invalid response received from server! Response is empty.", "The API response is empty or null"),
    ("RESPONSE_FORMAT", "Invalid response received from server! Response format is not valid.", "The API response format is not in the expected structure"),
    ("RESPONSE_COUNT", "Invalid parameters passed to function! Response data contains more than one result!", "The API response contains multiple results when only one was expected"),
    ("RESPONSE_CODE", "Invalid parameters passed to function! Operation result or Error code not received from server", "The API response is missing required operation result or error code fields"),
    ("RESPONSE_REST", "Invalid parameters passed to function! REST return is not a valid array!", "The REST response is not in a valid array format"),
    ("CONTACT_INFO", "Invalid response received from server! Contact info is not a valid array or more than one contact info has returned!", "The contact information response is not in the expected format"),
    ("CONTACT_SAVE", "Invalid response received from server! Contact info could not be saved!", "The contact information could not be saved on the server"),
    ("DOMAIN_TRANSFER_REQUEST", "Invalid response received from server! Domain transfer request could not be completed!", "The domain transfer request failed to complete"),
    ("DOMAIN_RENEW", "Invalid response received from server! Domain renew request could not be completed!", "The domain renewal request failed to complete"),
    ("DOMAIN_REGISTER", "Invalid response received from server! Domain register request could not be completed!", "The domain registration request failed to complete"),
    ("DOMAIN_SYNC", "Invalid response received from server! Domain sync request could not be completed!", "The domain synchronization request failed to complete"),
];
