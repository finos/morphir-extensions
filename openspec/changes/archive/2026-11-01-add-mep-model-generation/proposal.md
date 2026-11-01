# Proposal

## Why

`morphir-avro` 0.3.0 and `morphir-openapi` 0.4.0 claim MEP `0.1` only. The MEP
contract in finos/morphir is at `0.2.0-draft.1`, which adds the model operation
`morphir.backend.generateModel`: the host sends a model envelope (version
`0.1.0-draft.1`) in a tuple that both sides agree on in `initialize`. The MoonBit
worker host sends a model envelope only on that line. The maintainer wants every
Morphir extension on the latest MEP as soon as possible.

Decision RUN-1 of `move-avro-and-openapi-extensions` (ratified) kept the Extism ABI
and deferred MEP 0.2 to finos/morphir-extensions#18. The ABI and the MEP version
are independent, and `morphir-extension-sdk` 0.4.0 implements the model operations
next to MEP `0.1`. The Gleam extension (`extension/gleam/v0.4.0`) is the reference.

Tracked by bead `morphir-up3q` (label `repo:morphir-extensions`).

## What Changes

- Both guests claim MEP `0.2.0-draft.1` and `0.1`. A host that offers only `0.1`
  gets the `0.1` session of the released versions, with the same output (design.html
  MEP-1).
- Both guests advertise one model transport tuple: `morphir.backend.generateModel`,
  profile `morphir-ir`, codec `morphir-json`, envelope `0.1.0-draft.1`, IR
  `4.0.0`. They do not offer Ion, because the SDK has no Ion reader for a WASM
  guest (MEP-2).
- Both guests answer `morphir.backend.generateModel`. They admit the envelope with
  the SDK, then run the generation of `morphir.backend.generate` on its IR, so the
  artifacts and diagnostics are the same for the same IR and options (MEP-3).
- A request with a tuple that was not agreed, an envelope that does not match its
  tuple, or a required guarantee is refused (MEP-4).
- The capability claims (`claimsVersion` `0.1.0-draft.2`) and so the release
  descriptors list both protocol lines and the model transport.
- The guest check runs both protocol lines on the built modules, and
  `qualify:moonbit` adds `extension generate` with a model envelope (QUA-2).
- Release `morphir-avro` 0.4.0 and `morphir-openapi` 0.5.0, both minor versions
  (REL-1). The Extism ABI, the Decimal projection and its options do not change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `avro-extension`: the protocol requirement claims both MEP lines and the model
  transport; a new requirement covers model generation; the MoonBit host
  requirement adds generate on the model line.
- `openapi-extension`: the same three changes.

## Impact

- Code: `rust/crates/morphir-avro-extension` and
  `rust/crates/morphir-openapi-extension` (a `model` module, capabilities, tests,
  `claims/expected.json`, crate versions).
- Guest check and qualification: `rust/crates/extension-guest-check` (sessions on
  both lines) and `.config/mise/tasks/qualify/moonbit` (model line).
- No new dependency: the SDK admits and builds model payloads. No change to the
  release workflow or to the bundle format: `release.json` takes the claims from
  the live describe call.
- Hosts: finos/morphir's Rust host compatibility pins keep working on MEP `0.1`.
  The finos/morphir bundle pins move in a separate pull request.
