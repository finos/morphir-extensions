# Spec Delta

## MODIFIED Requirements

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

## ADDED Requirements

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
