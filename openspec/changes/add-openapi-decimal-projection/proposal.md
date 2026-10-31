# Proposal

## Why

`morphir-openapi` 0.3.0 has no schema projection for the Morphir SDK type
`Decimal` (`morphir/SDK:decimal#decimal`). Both targets, `openapi` and
`json-schema`, report `JSC003` (unsupported form) for any IR that uses it, and the
MoonBit CLI exits 1 with `extension.operation_unsuccessful`.
`unsupported: "warn-and-skip"` only drops the affected definitions. `Decimal` is
the usual type for money and rates in Morphir models, so most finance models
cannot use the extension. The Avro extension in this repository already maps
`Decimal`. Reported in finos/morphir-extensions#20, found during the npm
requalification of the 0.3.0 bundles (finos/morphir-extensions#11) with
`typed-pricing.json` from finos/morphir-moonbit.

Tracked by bead `morphir-ltut` (label `repo:morphir-extensions`).

## What Changes

- Project `Decimal` by default to a JSON string that keeps the exact value:
  `"type": "string"`, the registered format `"decimal"` and a `pattern` that
  accepts the decimal strings that Morphir data codecs write (design.html DEC-1).
- Add the backend option `decimal_encoding` (`"string"`, the default, or
  `"number"`). `"number"` projects `Decimal` to `"type": "number"` with format
  `"decimal"` for consumers that accept binary floating point (DEC-2).
- Add the optional backend options `decimal_precision` and `decimal_scale`, with
  the names and the meaning of the Avro extension's options. When both are set,
  the string pattern limits the integer and fraction digits. They have no
  default, because a Morphir `Decimal` has no fixed precision (DEC-3).
- Both targets use the same projection. The OpenAPI 3.0 downgrade keeps the
  `Decimal` schema unchanged, because the schema uses only keywords that 3.0
  has (DEC-4).
- Add `typed-pricing.json` (IR v4, both of its types use `Decimal`) as a test
  fixture: golden documents for both targets, the guest-check parity cases and
  the MoonBit host qualification.
- Release `morphir-openapi` 0.4.0, a minor version: new behavior and new
  options, and no change for IR that 0.3.0 accepted (DEC-5).
- Other SDK types that also have no projection (`LocalDate`, `LocalTime`,
  `Month` and others) stay out of scope. A follow-up issue lists them (DEC-6).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `openapi-extension`: adds the `Decimal` projection and its three options. The
  identity requirement already names the crate version, so it does not change.

## Impact

- Code: `rust/crates/morphir-openapi-extension` (type projection, schema model,
  options), its tests, golden files and `claims/expected.json` (version).
- Guest check and qualification: `rust/crates/extension-guest-check` (one more
  fixture and option sets) and `.config/mise/tasks/qualify/moonbit` (one more
  fixture).
- No new dependency. No change to the Avro extension, the MEP surface, the
  release workflow or the bundle format.
- Users: IR that 0.3.0 rejected because of `Decimal` now generates documents.
  Output for IR without `Decimal` does not change.
