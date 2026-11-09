# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-11-09

### Added

- First release: a faithful Rust port of [php-dna](https://github.com/domainreseller/php-dna) 4.0.4, the DomainNameAPI REST client.
- `DomainNameApi` client with all 45 public methods of php-dna in snake_case:
  - account: `get_reseller_details`, `get_current_balance`;
  - search and lists: `check_availability`, `get_list`, `get_tld_list`;
  - domain: `get_details`, `sync_from_registry`, `get_contacts`, `save_contacts`, `modify_name_server`, `enable_theft_protection_lock`, `disable_theft_protection_lock`, `modify_privacy_protection_status`, `renew`, `register_with_contact_info` (plus `register_with_contact_info_opts` with `RegisterOptions`);
  - child nameservers: `add_child_name_server`, `delete_child_name_server`, `modify_child_name_server`;
  - transfers: `transfer` (plus `transfer_with_contacts`), `cancel_transfer`, `approve_transfer`, `reject_transfer`, `check_transfer`;
  - DNS zone: `get_resource_records`, `add_resource_record`, `edit_resource_record`, `delete_resource_record`, `apply_resource_records`, `set_resource_record_set`, `delete_resource_record_set`, `add_resource_record_value`, `remove_resource_record_value`, `replace_resource_record_value`;
  - URL forwarding: `get_forwarding`, `set_forwarding`, `delete_forwarding`;
  - utilities: `is_tr_tld`, `validate_contact`;
  - diagnostics and settings: `get_request_data`, `get_response_data`, `get_response_headers`, `get_parsed_response_data`, `get_last_function`, `get_service_url`, `set_service_url`, `set_error_reporting_enabled`, `set_telemetry_minimal`, `set_ssl_verification`.
- Return values are `serde_json::Value` with exactly php-dna's structure and envelopes; errors are returned, never thrown.
- Blocking HTTP via `ureq` with rustls (no OpenSSL, no async runtime), `User-Agent: DomainNameAPI-Rust/1.0.0`, 30 s / 10 s timeouts, redirects not followed, TLS verification switchable.
- php-dna's error mapping: `API_<status>` codes, readable messages for empty bodies, ABP boilerplate replacement, flattened validation errors, transport failures as errors (`API_<curl errno>` with php-dna's messages).
- Pluggable `Transport` trait for tests, proxies and custom HTTP clients.
- Zone call pacing (one second between `domains/zones*` calls), adjustable with `set_zone_min_interval`.
- Telemetry ported from php-dna (credential masking, ignored errors, stable classification, performance sampling, minimal mode), sent in the background; the Sentry DSN is empty until the project for this library exists, so nothing is sent yet. `set_telemetry_dsn` overrides it.
- Test-mode flag accepts `bool` or strings (`"true"`, `"1"`, `"on"`, `"yes"`), credentials are trimmed.
- 39 runnable examples (`cargo run --example <name>`), one per php-dna example plus DNS zone, forwarding, transfer and utility examples.
- Conformance test replaying all 126 php-dna fixtures, plus unit tests for the HTTP layer, error mapping, telemetry and utilities.
- README in English with translations in Turkish, German, French, Italian, Dutch, Azerbaijani, Arabic and Simplified Chinese.

[1.0.0]: https://github.com/domainreseller/rust-dna/releases/tag/v1.0.0
