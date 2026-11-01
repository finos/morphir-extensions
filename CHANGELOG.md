# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Each extension has its own version and release tag `extension/<short-id>/v<version>`.

## [Unreleased]

### Changed

- Archive the OpenSpec change `add-mep-model-generation`; its requirements are now
  in `openspec/specs/avro-extension` and `openspec/specs/openapi-extension`.
- Archive the OpenSpec change `add-openapi-decimal-projection`; its `Decimal`
  requirements are now in `openspec/specs/openapi-extension`.
- Archive the OpenSpec change `move-avro-and-openapi-extensions`; its specs
  `extension-bundle-release`, `avro-extension` and `openapi-extension` are now in
  `openspec/specs/`.

## morphir-avro 0.4.0 and morphir-openapi 0.5.0 - 2026-11-01

Releases `extension/avro/v0.4.0` and `extension/openapi/v0.5.0`, attested bundles built
from `a73cda0` (finos/morphir-extensions#25). Qualified on the MoonBit worker host,
native and npm entry, before and after the tags.

| File | SHA-256 |
| --- | --- |
| `morphir-avro-0.4.0.wasm` | `7f86a1983028d1e7d6b03de4d553417b05f8fc8eb47e20ab922830b0daecdd5f` |
| `morphir-avro-0.4.0.release.json` | `926647e8ca0df3d76206fe8d1744c377cf9e1dbdea5c8494d5fadc2dcb991447` |
| `morphir-openapi-0.5.0.wasm` | `691bb4892c1eaba11b5dbddb06a3217b7430e5c31c0239e8440ad4081c78da4d` |
| `morphir-openapi-0.5.0.release.json` | `5385f37f1de928f3c48b19ee561a8ac39840cfcef1cf6fe6b49a7571ec3d4bff` |

### Added

- MEP `0.2.0-draft.1` for both extensions (finos/morphir-extensions#18, OpenSpec
  change `add-mep-model-generation`). The guests claim `0.2.0-draft.1` and `0.1`
  and answer `morphir.backend.generateModel` for the one model transport tuple
  `morphir-ir` / `morphir-json` / envelope `0.1.0-draft.1` / IR `4.0.0`, with the
  artifacts and diagnostics of `morphir.backend.generate` for the envelope's IR.
  They offer no Ion tuple: the SDK has no Ion reader for a WASM guest. An
  unagreed tuple, an envelope that does not match its tuple or digest, and a
  required guarantee are refused with a JSON-RPC error. A host that offers only
  `0.1` gets the `0.1` session, unchanged. The Extism ABI, the projections and
  their options do not change.
- The guest check runs an MEP `0.2.0-draft.1` session on the built modules, and
  `qualify:moonbit` also generates from a model envelope of each IR v4 fixture.

## morphir-openapi 0.4.0 - 2026-10-31

Release `extension/openapi/v0.4.0`, an attested bundle built from `166ffce`
(finos/morphir-extensions#22). Qualified on the MoonBit worker host, native and npm
entry, before the tag.

### Added

- A schema for the Morphir SDK `Decimal`
  (`morphir/SDK:decimal#decimal`) in both targets, `openapi` (3.1 and 3.0) and
  `json-schema` (finos/morphir-extensions#20). 0.3.0 refused such IR with `JSC003`.
  By default a `Decimal` is a string that keeps the exact value:
  `{"type": "string", "format": "decimal", "pattern": "^-?(0|[1-9][0-9]*)(\\.[0-9]+)?$"}`.
  The pattern accepts plain decimal notation, as Morphir data writers emit it, and
  rejects exponents, a `+` sign, `.5`, `1.` and leading zeros.
- Options `decimal_encoding` (`"string"`, the default, or
  `"number"` for `{"type": "number", "format": "decimal"}`), and `decimal_precision`
  with `decimal_scale` (set both; the Avro extension's meaning, no default; they
  bound the integer and fraction digits of the string pattern). Invalid
  combinations are `JSC002`.

### Changed

- The guest check and `qualify:moonbit` also run `typed-pricing-v4.json`, the IR v4
  model from finos/morphir-moonbit that uses `Decimal`.

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
