# AGENTS.md

Guidance for people and AI coding agents working on this repository.

## What this is

`rust-dna` is the Rust SDK for the Domain Name API reseller REST API (crate `domainnameapi`). It mirrors the
[`dna` CLI](https://github.com/domainreseller/cli): the same operations, groups, safety classes, environment
variables, retry rules and error codes. It has nothing to do with php-dna or the SOAP API.

## Commands

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                 # unit, fake-API, spec and doc tests; none calls the real API
cargo build --examples
# without a local toolchain:
docker run --rm -v "$PWD":/src -w /src rust:1.83 sh -c 'rustup component add rustfmt clippy; cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo test'
```

Everything must pass on the MSRV (1.83, `rust-version` in `Cargo.toml`) and on the latest stable. Keep `Cargo.lock`
resolvable on the MSRV (`CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=fallback cargo update` with a newer cargo).

## Layout

| path | content |
|---|---|
| `src/client.rs` | `Client`, `ClientBuilder`, `Environment`: sending, retries, pacing, error parsing, `list_all` |
| `src/request.rs` | `Request`, `PagedRequest`, `Endpoint`, `OperationSafety`, `Query`, `JsonBody`, `Validator`, `Page`, `Empty` |
| `src/error.rs` | `Error`, `ErrorKind`, `codes` |
| `src/de.rs` | lenient decoding and the `model!`, `int_enum!`, `str_enum!` macros (internal) |
| `src/account.rs`, `contacts.rs`, `domains/`, `dns.rs`, `dnssec.rs`, `ssl.rs`, `hosting.rs`, `orders.rs`, `products.rs` | one module per CLI group: models, enums, request types, `XxxClient` |
| `tests/common/` | the fake HTTP API every integration test uses |
| `tests/spec.rs` | the spec coverage test; `tests/testdata/openapi.json` is the API document the CLI embeds |

## Adding or changing an operation

1. Copy the new `internal/openapi/openapi.json` from the CLI repository to `tests/testdata/openapi.json` and add the
   operation to `CLI_OPERATIONS` in `tests/spec.rs` (from the CLI's `internal/ops/registry.go`). The test fails until
   the request type exists.
2. Add a request type named after the CLI command (`dna domain renew-check` → `DomainRenewCheckRequest`) in the area
   module: required inputs in `new(...)`, optional ones as chained setters, `#[non_exhaustive]`, documented fields.
   Implement `Request`: `endpoint()` with the CLI's method, path, safety and idempotency, plus `query()`, `body()`
   and `validate()` as needed. Wire names live only in `query()`/`body()`.
3. Declare answer models with `model!` (lenient, `#[non_exhaustive]`) and enums with `int_enum!`/`str_enum!`.
4. Add the area client method, a case in `cases()` of `tests/spec.rs` (every optional value set), a line in
   `tests/areas.rs`, and a behaviour test.

## Rules

- **Safety class** is a product decision and must match the CLI's registry: `Billable` charges the balance,
  `Destructive` deletes or is hard to undo, `Write` is free and reversible, `Read` has no side effects even when the
  method is POST or PUT. Call `.idempotent()` only where the CLI marks the operation idempotent.
- **Never retry a billable or non-idempotent request after a network error or a 5xx answer.** Only 429 is safe for
  them. Do not weaken the retry rules in `src/client.rs`.
- **`"success": false` with HTTP 200 is returned as it is** (as in the Java SDK); document the `success` field.
- **No telemetry.** Nothing may leave the user's process except the API requests they make.
- **Dependencies stay small**: `reqwest` (rustls, no default features), `serde`, `serde_json`, `tokio` (timers only),
  `futures-util`, `chrono`, `base64`, `httpdate`. No OpenSSL.
- **Tests never call the real API.** Use `tests/common::FakeApi`.
- **Public API:** `#![deny(missing_docs)]`; every public item has rustdoc. Error codes, `ErrorKind` values and safety
  classes are a contract.
- **README translations:** `README.md` is the source; keep the eight `README-XX.md` translations in sync.
- **Style:** English for code, comments and docs; messages say what happened and what to do next.
- **Commits:** imperative subject line. Do not add `Co-Authored-By` trailers for AI tools.
