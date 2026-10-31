# Spec Delta

## ADDED Requirements

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

