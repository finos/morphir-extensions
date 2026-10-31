# Tasks

Bead `morphir-ltut` (label `repo:morphir-extensions`): group 1 is
`morphir-ltut.1`, groups 2 to 4 are `morphir-ltut.2`, group 5 is `morphir-ltut.3`.

## 1. OpenSpec change

- [x] 1.1 Write the proposal, the `openapi-extension` delta spec, design.html and
  this task list; verify with `mise exec -- openspec validate --all --strict`.

## 2. Options

- [x] 2.1 Add `decimal_encoding`, `decimal_precision` and `decimal_scale` to
  `SchemaOptions` with their `JSC002` rules (design.html DEC-2, DEC-3); verify with
  unit tests in `tests/options.rs` for the defaults, the spellings and every
  rejected combination.

## 3. Projection and rendering

- [x] 3.1 Add `Schema::Decimal` and project `morphir/SDK:decimal#decimal` to it
  (SCH-1); verify with projection unit tests for the default, the number encoding
  and the bounds.
- [x] 3.2 Render the string and number forms and the bounded patterns (DEC-1,
  DEC-3); verify with unit tests that validate the instances of the spec scenarios
  against the rendered schema.
- [x] 3.3 Pin the OpenAPI 3.0 output for `Decimal` and `Maybe Decimal` (DEC-4);
  verify with a downgrade test and 3.0 metaschema validation.

## 4. Fixture, goldens, guest check and documentation

- [x] 4.1 Copy `typed-pricing.json` into the crate fixtures and the guest-check
  fixtures; add golden documents for `json-schema`, `openapi` 3.1 and 3.0 with
  metaschema validation (TST-1); verify with `cargo test -p morphir-openapi-extension`
  and the fixture test of `extension-guest-check`.
- [x] 4.2 Add the fixture and Decimal option sets to the guest-check parity test and
  the fixture to `qualify:moonbit`; verify with `MISE_ENV=rust mise run rust:guest-check`.
- [x] 4.3 Set the crate version to 0.4.0, update `claims/expected.json`, the
  `CHANGELOG.md` entry and the option documentation (DEC-5); verify with
  `MISE_ENV=rust mise run rust:fmt`, `rust:lint`, `rust:test`, `rust:wasm` and
  `rust:guest-check`.
- [x] 4.4 Open a follow-up issue that lists the other SDK types without a projection,
  with the evidence of DEC-6; verify the issue URL is in the pull request
  (finos/morphir-extensions#21).

## 5. Release preparation

- [x] 5.1 Qualify the PR CI `bundles-dry-run` bundle of `openapi` on the MoonBit
  worker host, native and npm entry, with `mise run qualify:moonbit`; verify that the
  record passes and put it in the pull request. Passed, 14 of 14 commands, CLI
  finos/morphir-moonbit `405f990`
  (finos/morphir-extensions#22, comment 6545265403).
- [x] 5.2 After the merge, run `extension-release.yml` as a dry run for
  `extension/openapi/v0.4.0`, qualify that bundle the same way, and record the
  evidence; verify that the record passes. Dry run 41214127298 built `166ffce`;
  qualification passed, 14 of 14 commands, with module bytes equal to the PR bundle
  (finos/morphir-extensions#22, comment 6545365065).

## Workflow follow-up

- Push the tag `extension/openapi/v0.4.0` under Damian's approval and record the
  release in `CHANGELOG.md`. Done: tag pushed 2026-10-31, release run 41214792996.
- Archive the change after the release, and close bead `morphir-ltut` and
  finos/morphir-extensions#20.
