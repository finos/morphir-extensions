# openapi-extension Specification

## Purpose
Defines the OpenAPI and JSON Schema extension guest that Morphir hosts load to
project a Morphir specification into OpenAPI documents and JSON Schema 2020-12
documents.

## Requirements

### Requirement: OpenAPI extension identity
The guest SHALL describe itself with id `morphir-openapi`, name
`Morphir OpenAPI`, the version of its crate, the single type `backend`, license
`Apache-2.0` and homepage `https://github.com/finos/morphir-extensions`.

#### Scenario: Describe the 0.3.0 module
- **WHEN** a host sends `morphir.extension.describe` to the 0.3.0 module
- **THEN** the answer has id `morphir-openapi`, version `0.3.0`, types `["backend"]` and homepage `https://github.com/finos/morphir-extensions`

### Requirement: One extension with two targets
The guest SHALL claim one backend with the targets `openapi` and `json-schema`.
Both targets SHALL use the same schema projection, so a type has the same schema
in either output.

#### Scenario: Target selection
- **WHEN** a host sends `morphir.backend.generate` with target `json-schema`
- **THEN** the result holds JSON Schema 2020-12 documents and no OpenAPI document

#### Scenario: Same schema in both targets
- **WHEN** the same IR is generated with target `openapi` (default options) and with target `json-schema`
- **THEN** the component schema of each type in the OpenAPI document equals the JSON Schema of that type, apart from the base that a `$ref` is written against

### Requirement: OpenAPI guest ABI
The module SHALL be an Extism guest (`extism-kernel-v1`). It SHALL import only
functions of `extism:host/env` and SHALL export `handle`,
`morphir_extension_info`, `morphir_extension_capabilities` and `memory`.

#### Scenario: Import surface
- **WHEN** a host classifies the released module by its imports and exports
- **THEN** every import is from `extism:host/env`, there is no WASI import, and the host classifies it as `extism-kernel-v1`

### Requirement: OpenAPI protocol and capabilities
The guest SHALL claim MEP `0.1` only, IR versions `3` and `4`, and `generate`. It
SHALL answer `morphir.backend.generate` with plain Morphir IR JSON input.

#### Scenario: Generate from IR v4
- **WHEN** a host sends `morphir.backend.generate` with the canonical customer IR v4 fixture and target `openapi`
- **THEN** the result is successful and holds one OpenAPI document

### Requirement: Valid documents
OpenAPI documents that the extension generates for the golden fixtures SHALL
validate against the OpenAPI 3.0 or 3.1 schema of their version, and JSON Schema
documents SHALL validate against the JSON Schema 2020-12 metaschema.

#### Scenario: Golden documents
- **WHEN** the crate tests validate each golden document
- **THEN** every document is valid for its schema

### Requirement: OpenAPI module parity
The released module SHALL produce the same artifacts and diagnostics as the native
build of the crate for every golden fixture and for both targets.

#### Scenario: Golden parity
- **WHEN** the guest check runs each golden fixture through the module and through the native crate
- **THEN** the artifacts and diagnostics are equal

### Requirement: OpenAPI runs on the MoonBit worker host
The released module SHALL run on the MoonBit worker host as a WASM worker, from
the native CLI and from the Node CLI, with a pin that names the module and its
`release.json`.

#### Scenario: Describe and generate with the MoonBit CLI
- **WHEN** a user runs `morphir extension describe` and `morphir extension generate --target json-schema` with plain IR JSON and such a pin
- **THEN** both commands exit 0, the recorded claims equal the live claims, and generate writes the JSON Schema documents
