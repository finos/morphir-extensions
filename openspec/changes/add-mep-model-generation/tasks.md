# Tasks

Bead `morphir-up3q` (label `repo:morphir-extensions`): group 1 is
`morphir-up3q.1`, groups 2 to 4 are `morphir-up3q.2`, group 5 is `morphir-up3q.3`.

## 1. OpenSpec change

- [x] 1.1 Write the proposal, the `avro-extension` and `openapi-extension` delta
  specs, design.html and this task list; verify with
  `mise run openspec:validate`.

## 2. Tests first

- [x] 2.1 Add native tests in each crate for the claims (both protocol lines, the one
  JSON tuple), for negotiation through the SDK session (a `0.1`-only host, a
  `0.2.0-draft.1` host with an Ion-first offer), for `generateModel` against the
  goldens and against `generate`, and for the refusals (unagreed tuple, wrong envelope
  version, wrong digest, required guarantee); verify that they fail before 3.1.

## 3. Implementation

- [x] 3.1 Add `src/model.rs` to both crates and set `model_transport` (MEP-1 to
  MEP-5); verify with `MISE_ENV=rust mise run rust:test`.
- [x] 3.2 Set the crate versions to avro 0.4.0 and openapi 0.5.0 and update
  `claims/expected.json` (REL-1); verify with the claims tests.

## 4. Guest check, qualification and documentation

- [x] 4.1 Add a `0.2.0-draft.1` session to `extension-guest-check`: negotiation,
  `generateModel` parity with `generate` on the module, refusals (QUA-2); keep the
  `0.1` session; verify with `MISE_ENV=rust mise run rust:guest-check`.
- [x] 4.2 Add the model line to `qualify:moonbit` (QUA-2); verify on a local bundle
  with the native and npm CLI of finos/morphir-moonbit `main`. Passed for both local
  bundles on CLI `40623b8` (avro 12 of 12, openapi 22 of 22 commands).
- [x] 4.3 Update `CHANGELOG.md`, the crate READMEs and AGENTS.md; verify with
  `MISE_ENV=rust mise run rust:fmt`, `rust:lint`, `rust:test`, `rust:wasm`,
  `rust:guest-check` and `rust:avro-idl`.

## 5. Release preparation

- [x] 5.1 Qualify the PR CI `bundles-dry-run` bundles of `avro` and `openapi` on the
  MoonBit worker host, native and npm entry, with `mise run qualify:moonbit`; verify
  that both records pass and put them in the pull request. Passed on CI run
  41267575321 bundles, CLI finos/morphir-moonbit `40623b8`: avro 12 of 12, openapi 22
  of 22 commands (finos/morphir-extensions#25, comment 6552688313).
- [ ] 5.2 After the merge (Damian's approval), run `extension-release.yml` as a dry
  run for `extension/avro/v0.4.0` and `extension/openapi/v0.5.0`, qualify those
  bundles the same way, and record the evidence.

## Workflow follow-up

- Push the tags `extension/avro/v0.4.0` and `extension/openapi/v0.5.0` under
  Damian's approval and record the releases in `CHANGELOG.md`.
- Archive the change after the releases, and close bead `morphir-up3q` and
  finos/morphir-extensions#18.
