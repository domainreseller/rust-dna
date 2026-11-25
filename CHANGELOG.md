# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.1.0] - 2026-11-26

### Added

- `X-App` and `X-Version` headers on every request (retries included), so the API team can tell integrations apart.
  They default to `rust` and the SDK version; set your own with `ClientBuilder::app(name, version)` or the
  `DNA_APP` and `DNA_APP_VERSION` variables read by `from_env()`. The name is lower-cased and keeps only
  `a-z0-9._-`, the version only digits and dots (`WiseCP5`, `V2.0.20-beta` → `wisecp5`, `2.0.20`); empty values fall
  back to the defaults. `Client::app()`, `Client::app_version()` and `DEFAULT_APP` expose them. `User-Agent` is
  unchanged.

### Build

- Releases are published to crates.io by a workflow that runs when a GitHub release is published, once a
  crates.io token or trusted publishing is set up for this repository.

## [2.0.0] - 2026-11-23

A new SDK, modelled on the [`dna` CLI](https://github.com/domainreseller/cli) and built from the same API document.
1.x was a port of the PHP library; 2.0 is not compatible with it. See "Upgrading from 1.x" in the README.

### Added

- Typed area clients for all 67 operations of the REST API: `account()`, `contacts()`, `domains()` (with
  `forwarding()`, `child_name_servers()` and `documents()`), `dns()`, `dnssec()`, `ssl()`, `hosting()`, `orders()` and
  `products()`. DNSSEC, SSL certificates (purchase, configure, list, renew, reissue, revoke), hosting, orders,
  products, contact handles, registry documents, auth codes, restores, the reseller profile and the transaction
  ledger are new.
- One request type per operation, named after its CLI command (`dna domain renew-check` is
  `DomainRenewCheckRequest`), all implementing the public `Request` trait (`endpoint()`, and default `query()`,
  `body()` and `validate()`); list requests also implement `PagedRequest`. Implement `Request` for endpoints the SDK
  does not cover; `Client::send` sends any request, `send_with_response` also returns the correlation ID, the
  soft-quota state, the number of attempts and the duration.
- Documented serde models for every answer and enums for coded values (`DomainStatus`, `TransactionType`,
  `DnssecAlgorithm`, ...) with an `Unknown(n)` fallback, plus open text enums (`RecordType`, `ContactType`,
  `ForwardType`, ...) with `Other(text)`. Reading is lenient: numbers and booleans as text, dates with or without an
  offset, `null` lists, unknown values. Requests are strict and are checked before anything is sent.
- Async client on Tokio and `reqwest` (rustls, no OpenSSL). `Client` is `Clone + Send + Sync`; `list_all` methods
  return a lazy `Stream` of items.
- Every request declares an `OperationSafety` (`Read`, `Write`, `Billable`, `Destructive`), the same classes the CLI
  uses. Read-only mode blocks everything but reads before it is sent.
- Retries like the CLI: 429 always (honouring `Retry-After`), network errors and 502/503/504 only for reads and
  idempotent writes, so a billable request is never sent twice. Client-side pacing keeps requests under the
  gateway's rate limits.
- One `Error` type with a stable `kind()` (the CLI's kinds), the API's error `code()`, validation errors,
  correlation ID, hint, `is_retryable()` and `is_outcome_unknown()`; common codes in `domainnameapi::codes`.
- `Client::new`, `Client::builder()` and `Client::from_env()`, which reads the CLI's variables (`DNA_RESELLER_ID`,
  `DNA_API_KEY`, `DNA_ENV`, `DNA_TEST_MODE`, `DNA_BASE_URL`, `DNA_READ_ONLY`). A custom `reqwest::Client` is accepted.
- Helpers: `DomainContacts::same`, `ContactHandles::same`, `DsData` parsing, `DocumentUpload::from_file`,
  `CertificateContent::full_chain`, `Tld::register_price`, `ProductsClient::tld`, `dns::fqdn`.
- SSL certificates are addressed by their certificate ID (a GUID), as the API requires since 2026-11-20; a domain
  name passed as an ID is refused before sending.
- Tests against a local fake API, and a spec coverage test that checks every operation of the API document has one
  request type with the CLI's method, path, safety class and idempotency, and sends only documented parameters and
  body properties.
- `AGENTS.md` with the rules for contributors and AI coding agents; README in nine languages.

### Changed

- `"success": false` answers with HTTP 200 (register, renew, transfer, SSL operations, contact creation) are returned
  as they are, like the Java SDK; check the `success` field.
- MSRV stays 1.83; CI runs fmt, clippy and the tests on 1.83 and on stable.
- User-Agent is `DomainNameAPI-Rust/2.0.0 (+https://github.com/domainreseller/rust-dna)`.

### Removed

- The php-dna compatible API (`DomainNameApi` with `serde_json::Value` envelopes, the `Transport` trait, the `php`
  helpers, `ureq`) and the php-dna conformance fixtures.
- Sentry error reporting. The SDK sends nothing anywhere except to the API.

## [1.0.1] - 2026-11-14

### Changed

- Error reporting now has its own Sentry project: the built-in default DSN points to the rust-dna project, so telemetry is active by default as documented.
- Turn it off with `set_error_reporting_enabled(false)`; a DSN set with `set_telemetry_dsn` still overrides the default.

## [1.0.0] - 2026-11-09

### Added

- First release: a faithful Rust port of [php-dna](https://github.com/domainreseller/php-dna) (state of commit f946d69, after 4.0.4), the DomainNameAPI REST client.
- `DomainNameApi` client with all 45 public methods of php-dna in snake_case:
  - account: `get_reseller_details`, `get_current_balance`;
  - search and lists: `check_availability`, `get_list`, `get_tld_list`;
  - domain: `get_details`, `sync_from_registry`, `get_contacts`, `save_contacts`, `modify_name_server`, `enable_theft_protection_lock`, `disable_theft_protection_lock`, `modify_privacy_protection_status`, `renew`, `register_with_contact_info` (plus `register_with_contact_info_opts` with `RegisterOptions`);
  - child nameservers: `add_child_name_server`, `delete_child_name_server`, `modify_child_name_server`;
  - transfers: `transfer`, `cancel_transfer`, `approve_transfer`, `reject_transfer`, `check_transfer`;
  - DNS zone: `get_resource_records`, `add_resource_record`, `edit_resource_record`, `delete_resource_record`, `apply_resource_records`, `set_resource_record_set`, `delete_resource_record_set`, `add_resource_record_value`, `remove_resource_record_value`, `replace_resource_record_value`;
  - URL forwarding: `get_forwarding`, `set_forwarding`, `delete_forwarding`;
  - utilities: `is_tr_tld`, `validate_contact`;
  - diagnostics and settings: `get_request_data`, `get_response_data`, `get_response_headers`, `get_parsed_response_data`, `get_last_function`, `get_service_url`, `set_service_url`, `set_error_reporting_enabled`, `set_telemetry_minimal`, `set_ssl_verification`.
- Return values are `serde_json::Value` with exactly php-dna's structure and envelopes; errors are returned, never thrown.
- Blocking HTTP via `ureq` with rustls (no OpenSSL, no async runtime), `User-Agent: DomainNameAPI-Rust/1.0.0`, 30 s / 10 s timeouts, redirects not followed, TLS verification switchable.
- php-dna's error mapping: `API_<status>` codes with the gateway's own code as `GatewayCode`, readable messages for empty bodies, ABP boilerplate replacement (never with the raw body), flattened validation errors, transport failures as errors with `CURL_<errno>` codes and php-dna's messages.
- Refusals are errors: register, transfer and renew answered with `success: false` return `REGISTER_DOMAIN` / `TRANSFER_DOMAIN` / `RENEW_DOMAIN`; DNS zone writes return the apply error when the automatic apply fails.
- Transfers never send contacts (`contacts: []`); the registrar takes them over.
- `get_forwarding` treats only "Redirect information not found" as disabled; `delete_forwarding` reports `Removed` and is a no-op success when nothing is set.
- `modify_privacy_protection_status` accepts panel flags (`"1"`, `"on"`, `"true"`, `"yes"`, `1`) through the `BoolFlag` trait; the reason is not sent.
- `get_current_balance` fails with `BALANCE` for currencies the account does not report; balances are formatted with two decimals.
- `validate_contact` keeps a number's own `+country` code and falls back to the city for a missing state.
- Pluggable `Transport` trait for tests, proxies and custom HTTP clients.
- Zone call pacing (one second between `domains/zones*` calls), adjustable with `set_zone_min_interval`.
- Telemetry ported from php-dna (credential masking, ignored errors, stable classification, performance sampling, minimal mode), sent in the background; the Sentry DSN is empty until the project for this library exists, so nothing is sent yet. `set_telemetry_dsn` overrides it.
- Test-mode flag accepts `bool`, integers or strings (`"true"`, `"1"`, `"on"`, `"yes"`), credentials are trimmed.
- 39 runnable examples (`cargo run --example <name>`), one per php-dna example plus DNS zone, forwarding, transfer and utility examples.
- Conformance test replaying all 134 php-dna fixtures, plus unit tests for the HTTP layer, error mapping, telemetry and utilities.
- README in English with translations in Turkish, German, French, Italian, Dutch, Azerbaijani, Arabic and Simplified Chinese.

[Unreleased]: https://github.com/domainreseller/rust-dna/compare/v2.1.0...HEAD
[2.1.0]: https://github.com/domainreseller/rust-dna/releases/tag/v2.1.0
[2.0.0]: https://github.com/domainreseller/rust-dna/releases/tag/v2.0.0
[1.0.1]: https://github.com/domainreseller/rust-dna/releases/tag/v1.0.1
[1.0.0]: https://github.com/domainreseller/rust-dna/releases/tag/v1.0.0
