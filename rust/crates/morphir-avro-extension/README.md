# Morphir Avro extension

`morphir-avro-extension` is a portable Morphir Extension Protocol backend for
Apache Avro. It accepts Morphir IR v3 and v4 and returns Avro artifacts through
the typed `GenerateRequest { ir, options }` operation. The crate is implemented
and tested in this repository and released as an attested bundle.

The native library contains the pure normalization, projection, and rendering
pipeline. The `wasm32-unknown-unknown` build adds a thin MEP guest. The guest
does not read files, write files, or use the network. A Morphir host reads IR,
calls the guest, validates its returned relative artifact paths, and publishes
the files.

## Build and test

From the root of finos/morphir-extensions:

```console
MISE_ENV=rust mise run rust:test
MISE_ENV=rust mise run rust:wasm
MISE_ENV=rust mise run rust:guest-check   # the module answers as the native crate
MISE_ENV=rust mise run rust:avro-idl      # Apache Avro Tools 1.12.2 accepts the IDL goldens
MISE_ENV=rust mise run rust:bundle avro   # a local bundle in rust/target/bundle/avro
```

Or with Cargo, from `rust/`:

```console
cargo test -p morphir-avro-extension
cargo build --release -p morphir-avro-extension --target wasm32-unknown-unknown
```

## Backend contract

The extension advertises target `avro`, Morphir IR versions `3` and `4`, and the
MEP backend `generate` capability. Generation supports three projections:

- `schemas` emits public types without messages.
- `protocol-entry-points` emits declared v4 Application entry points.
- `protocol-public` emits every public value specification.

JSON output uses `.avsc` for schemas and `.avpr` for protocols. IDL output uses
`.avdl` for every projection. Defaults are JSON, schemas, self-contained
dependencies, inline aliases, strict unsupported-form handling, logical types
enabled, decimal precision 38, and decimal scale 10.

Value bodies never enter the projection model. A public zero-argument value can
become a constant message in `protocol-public`, but the message contains only
`morphir.value-kind = constant` metadata. It never contains or evaluates the
constant value.

## Releases

From `0.3.0`, this extension is released from finos/morphir-extensions by the tag
`extension/avro/v<crate-version>`. Each release holds `morphir-avro-<version>.wasm`,
`morphir-avro-<version>.release.json`, a `.sha256` file for each, and build
provenance attestations. The repository README describes the release steps and
how to verify a download.

`claims/expected.json` holds the claims that the module must describe. Change it
together with any change to the extension identity or capabilities;
`mise run rust:guest-check` fails otherwise.

Releases up to `extension/avro/v0.2.0` stay in
[finos/morphir-rust](https://github.com/finos/morphir-rust/releases), with the
release contract described in its
[`docs/contributors/design/extensions/README.md`](https://github.com/finos/morphir-rust/blob/945486e1320385e22986ef937db82e344b179d05/docs/contributors/design/extensions/README.md)
and its
[packaging contract](https://github.com/finos/morphir-rust/blob/945486e1320385e22986ef937db82e344b179d05/docs/extensions/capability-claims.md#packaging-wasm-extensions).
