# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Each extension has its own version and release tag `extension/<short-id>/v<version>`.

## [Unreleased]

### Added

- `morphir-openapi` 0.4.0: a schema for the Morphir SDK `Decimal`
  (`morphir/SDK:decimal#decimal`) in both targets, `openapi` (3.1 and 3.0) and
  `json-schema` (finos/morphir-extensions#20). 0.3.0 refused such IR with `JSC003`.
  By default a `Decimal` is a string that keeps the exact value:
  `{"type": "string", "format": "decimal", "pattern": "^-?(0|[1-9][0-9]*)(\\.[0-9]+)?$"}`.
  The pattern accepts plain decimal notation, as Morphir data writers emit it, and
  rejects exponents, a `+` sign, `.5`, `1.` and leading zeros.
- `morphir-openapi` options `decimal_encoding` (`"string"`, the default, or
  `"number"` for `{"type": "number", "format": "decimal"}`), and `decimal_precision`
  with `decimal_scale` (set both; the Avro extension's meaning, no default; they
  bound the integer and fraction digits of the string pattern). Invalid
  combinations are `JSC002`.

### Changed

- The guest check and `qualify:moonbit` also run `typed-pricing-v4.json`, the IR v4
  model from finos/morphir-moonbit that uses `Decimal`.
- Archive the OpenSpec change `move-avro-and-openapi-extensions`; its specs
  `extension-bundle-release`, `avro-extension` and `openapi-extension` are now in
  `openspec/specs/`.

## morphir-avro 0.3.0 and morphir-openapi 0.3.0 - 2026-10-31

First releases from this repository: `extension/avro/v0.3.0` and
`extension/openapi/v0.3.0`, attested bundles built from `96d57db`.

### Added

- Repository setup: the shared OpenSpec `html-design` schema, pinned mise tools and
  tasks, agent instructions, the shared beads tracker configuration, the
  pr-monitor and build-master skills, and CI.
- OpenSpec change `move-avro-and-openapi-extensions`: move the Avro and the OpenAPI
  and JSON Schema extensions here from finos/morphir-rust and release them as
  attested bundles.
- `rust/`: the Avro extension (`morphir-avro-extension`) and the OpenAPI and JSON
  Schema extension (`morphir-openapi-extension`), imported with their history from
  finos/morphir-rust at `945486e`. They build against the published Morphir crates
  from crates.io. Rust CI jobs run when `rust/` or its tooling changes.
- `extension-guest-check`: a test-only Extism host that runs each built module in an
  MEP 0.1 session, compares it with the native crate and reads its claims.
- `.github/workflows/extension-release.yml`: one release workflow for every
  extension, with `release.json`, `.sha256` files and verified build provenance
  attestations; `rust/extensions.toml` declares the extensions.
- `rust:avro-idl` (Apache Avro Tools 1.12.2), `qualify:moonbit` (MoonBit worker
  host, native and Node CLI) and the manual workflow `qualify-moonbit.yml` (the
  same check on Linux, with the Python 0.4.2 bundle as a control).

### Changed

- `morphir-avro` and `morphir-openapi` 0.3.0: the homepage is
  finos/morphir-extensions, and the release files are named after the extension
  id (`morphir-avro-0.3.0.wasm`, `morphir-openapi-0.3.0.wasm`).
