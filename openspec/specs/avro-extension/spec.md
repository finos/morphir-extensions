# avro-extension Specification

## Purpose
Defines the Avro extension guest that Morphir hosts load to project a Morphir
specification into Apache Avro schemas, protocols and IDL.

## Requirements

### Requirement: Avro extension identity
The guest SHALL describe itself with id `morphir-avro`, name `Morphir Avro`, the
version of its crate, the single type `backend`, license `Apache-2.0` and
homepage `https://github.com/finos/morphir-extensions`.

#### Scenario: Describe the 0.3.0 module
- **WHEN** a host sends `morphir.extension.describe` to the 0.3.0 module
- **THEN** the answer has id `morphir-avro`, version `0.3.0`, types `["backend"]` and homepage `https://github.com/finos/morphir-extensions`

### Requirement: Avro guest ABI
The module SHALL be an Extism guest (`extism-kernel-v1`). It SHALL import only
functions of `extism:host/env` and SHALL export `handle`,
`morphir_extension_info`, `morphir_extension_capabilities` and `memory`.

#### Scenario: Import surface
- **WHEN** a host classifies the released module by its imports and exports
- **THEN** every import is from `extism:host/env`, there is no WASI import, and the host classifies it as `extism-kernel-v1`

### Requirement: Avro protocol and capabilities
The guest SHALL claim MEP `0.1` only, a backend with the target `avro`, IR
versions `3` and `4`, and `generate`. It SHALL answer `morphir.backend.generate`
with plain Morphir IR JSON input.

#### Scenario: Generate from IR v4
- **WHEN** a host sends `morphir.backend.generate` with the canonical customer IR v4 fixture and target `avro`
- **THEN** the result is successful and holds the Avro artifacts for the fixture

#### Scenario: Unsupported IR version
- **WHEN** a host sends a model whose `formatVersion` is `5`
- **THEN** the guest answers with a result that has `success` false, one error diagnostic `unsupported_format_version_major` and no artifacts

### Requirement: Avro module parity
The released module SHALL produce the same artifacts and diagnostics as the native
build of the crate for every golden fixture.

#### Scenario: Golden parity
- **WHEN** the guest check runs each golden fixture through the module and through the native crate
- **THEN** the artifacts and diagnostics are equal

### Requirement: Valid Avro IDL
Avro IDL that the extension generates for the golden fixtures SHALL be accepted
by Apache Avro Tools 1.12.2.

#### Scenario: IDL goldens
- **WHEN** CI runs the Avro IDL check on the IDL goldens
- **THEN** Apache Avro Tools 1.12.2 converts each golden without an error

### Requirement: Avro runs on the MoonBit worker host
The released module SHALL run on the MoonBit worker host as a WASM worker, from
the native CLI and from the Node CLI, with a pin that names the module and its
`release.json`.

#### Scenario: Describe and generate with the MoonBit CLI
- **WHEN** a user runs `morphir extension describe` and `morphir extension generate` with plain IR JSON and such a pin
- **THEN** both commands exit 0, the recorded claims equal the live claims, and generate writes the Avro artifacts
