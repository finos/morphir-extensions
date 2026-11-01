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
The guest SHALL claim MEP `0.2.0-draft.1` and `0.1`, IR versions `3` and `4`,
`generate`, and a model transport with the one tuple
`morphir.backend.generateModel` / `morphir-ir` / `morphir-json` /
`0.1.0-draft.1` / `4.0.0`. It SHALL answer `morphir.backend.generate` with plain
Morphir IR JSON input on either line.

#### Scenario: Generate from IR v4
- **WHEN** a host sends `morphir.backend.generate` with the canonical customer IR v4 fixture and target `openapi`
- **THEN** the result is successful and holds one OpenAPI document

#### Scenario: Host that offers only MEP 0.1
- **WHEN** a host sends `morphir.initialize` with protocol versions `["0.1"]` and no model transport
- **THEN** the session runs on `0.1`, the initialize result has no model transport, and `morphir.backend.generateModel` is refused as not negotiated

#### Scenario: Host that offers MEP 0.2.0-draft.1
- **WHEN** a host offers protocol versions `["0.2.0-draft.1", "0.1"]` with the Ion tuple first and the JSON tuple second
- **THEN** the session runs on `0.2.0-draft.1` and the agreed model transport holds only the JSON tuple

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

#### Scenario: Generate from a model envelope with the MoonBit CLI
- **WHEN** a user runs `morphir extension generate --target openapi` with a `morphir-ir` / `morphir-json` model envelope of an IR v4 fixture and such a pin
- **THEN** the command exits 0 on protocol `0.2.0-draft.1` with the JSON tuple, the process ends without a kill, and the files equal those of generate with the plain IR

### Requirement: Decimal projects to an exact decimal string
By default the extension SHALL project the SDK type `morphir/SDK:decimal#decimal`
to a JSON string schema with format `decimal` and the anchored pattern
`^-?(0|[1-9][0-9]*)(\.[0-9]+)?$`, so that the schema accepts the decimal strings
that Morphir data codecs write and keeps the exact value. The schema SHALL NOT
accept a JSON number.

#### Scenario: Default Decimal schema
- **WHEN** a record field of type `Decimal` is generated with default options
- **THEN** the field schema is `{"type": "string", "format": "decimal", "pattern": "^-?(0|[1-9][0-9]*)(\\.[0-9]+)?$"}`

#### Scenario: Canonical decimal strings are valid
- **WHEN** the strings `"0"`, `"-0.05"`, `"33500"`, `"12.34"` and `"10.50"` are validated against the default Decimal schema
- **THEN** every string is valid

#### Scenario: Other spellings are invalid
- **WHEN** the values `"1e3"`, `"+1"`, `".5"`, `"1."`, `"007"`, `""` and the JSON number `12.5` are validated against the default Decimal schema
- **THEN** every value is invalid

### Requirement: Decimal number encoding option
The option `decimal_encoding` SHALL accept `"string"` (the default) and
`"number"`. With `"number"` the extension SHALL project `Decimal` to
`{"type": "number", "format": "decimal"}`. Any other value SHALL be a `JSC002`
invalid option.

#### Scenario: Number encoding
- **WHEN** a record field of type `Decimal` is generated with `decimal_encoding: "number"`
- **THEN** the field schema is `{"type": "number", "format": "decimal"}`

#### Scenario: Unknown encoding
- **WHEN** a host sends `decimal_encoding: "float"`
- **THEN** the generation fails with a `JSC002` diagnostic and no artifacts

### Requirement: Decimal precision and scale options
The options `decimal_precision` and `decimal_scale` SHALL have the meaning of the
Avro extension's options of the same name and SHALL have no default. When both are
set, the string pattern SHALL allow at most `decimal_precision - decimal_scale`
integer digits and at most `decimal_scale` fraction digits. A `JSC002` SHALL
report one without the other, a precision outside 1 to 1000, a scale above the
precision, or either with `decimal_encoding: "number"`.

#### Scenario: Bounded pattern
- **WHEN** a `Decimal` field is generated with `decimal_precision: 10` and `decimal_scale: 2`
- **THEN** the strings `"12345678.90"` and `"-0.5"` are valid against its schema and the strings `"123456789.0"` and `"1.234"` are invalid

#### Scenario: Zero scale
- **WHEN** a `Decimal` field is generated with `decimal_precision: 5` and `decimal_scale: 0`
- **THEN** the string `"12345"` is valid against its schema and the strings `"123456"` and `"1.5"` are invalid

#### Scenario: Invalid precision and scale
- **WHEN** a host sends only `decimal_precision`, or a `decimal_scale` above `decimal_precision`, or `decimal_precision: 0`, or `decimal_precision` and `decimal_scale` with `decimal_encoding: "number"`
- **THEN** the generation fails with a `JSC002` diagnostic and no artifacts

### Requirement: Decimal schema in every output
The `Decimal` schema SHALL be the same in the JSON Schema 2020-12 documents, the
OpenAPI 3.1 document and the OpenAPI 3.0 document. An optional `Decimal`
(`Maybe Decimal`) SHALL keep the same keywords and SHALL also accept `null`.

#### Scenario: OpenAPI 3.0
- **WHEN** a record with a `Decimal` field and a `Maybe Decimal` field is generated with target `openapi` and `version: "3.0"`
- **THEN** the `Decimal` field schema equals the 3.1 schema, the `Maybe Decimal` field schema has the same keywords and `"nullable": true`, and the document validates against the OpenAPI 3.0 schema

#### Scenario: Typed pricing fixture
- **WHEN** `typed-pricing.json` (IR v4, from finos/morphir-moonbit, whose two types use `Decimal`) is generated with target `openapi` and with target `json-schema`
- **THEN** both generations succeed with no diagnostics, and every document validates against its metaschema

### Requirement: OpenAPI model generation
On a session that agreed the JSON tuple, the guest SHALL answer
`morphir.backend.generateModel` for both targets with the artifacts and
diagnostics that `morphir.backend.generate` gives for the envelope's IR, target
and options.

#### Scenario: Generate from a model envelope
- **WHEN** a host sends `morphir.backend.generateModel` with a JSON envelope of the typed-pricing IR v4 fixture, for target `openapi` and for target `json-schema`
- **THEN** each result equals the result of `morphir.backend.generate` for that IR and matches the reviewed typed-pricing golden documents

### Requirement: OpenAPI refuses unagreed model input
The guest SHALL refuse `morphir.backend.generateModel` with a JSON-RPC error and
no generation when the request tuple was not agreed, when the envelope does not
match its tuple or its payload digest, or when the request requires a guarantee.

#### Scenario: Unagreed tuple
- **WHEN** a host sends `morphir.backend.generateModel` with the tuple of IR `3.0.0`
- **THEN** the guest answers a JSON-RPC error and no result

#### Scenario: Wrong envelope
- **WHEN** a host sends an envelope whose `envelopeVersion` is `0.2.0`, or whose payload SHA-256 does not match its content
- **THEN** the guest answers a JSON-RPC error and no result
