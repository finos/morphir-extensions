# Proposal

## Why

The Avro extension (`morphir-avro`) and the OpenAPI and JSON Schema extension
(`morphir-openapi`) live in finos/morphir-rust and are released from there. The
Rust CLI is being sunset and the MoonBit CLI (finos/morphir-moonbit) is now the
main Morphir CLI. Extensions leave the CLI and live in the repository that owns
them. These two extensions do not belong to one language repository, so they move
to finos/morphir-extensions, the home of general-purpose Morphir extensions. From
there they are released as attested bundles that a GitHub-hosted repository index
can admit.

This change supersedes bead `morphir-ukvn.1.7` (A5) in the shared Morphir
tracker. That bead moved the two extensions to finos/morphir.

## What Changes

- Set up this repository like the other Morphir repositories: the shared OpenSpec
  `html-design` schema with its locked snapshot, a pinned
  `.config/mise/config.toml` with mise tasks (MoonBit `0.10.18+6c83a7f62` for the
  shared snapshot checker), AGENTS.md and CLAUDE.md, the beads configuration for
  the shared tracker, the pr-monitor and build-master skills, and CI.
- Import `crates/morphir-avro-extension` (14 commits) and
  `crates/morphir-openapi-extension` (32 commits) from finos/morphir-rust with
  their git history, under `rust/crates/`, in one Cargo workspace. The crate names
  do not change. The OpenAPI extension moves as one extension with its two targets
  `openapi` and `json-schema`; it is not split.
- Use only published Morphir crates from crates.io: `morphir-core` 0.2.0,
  `morphir-extension-sdk` 0.4.0, `morphir-projection` 0.1.0 (and `morphir-json`
  0.2.0 through them). No new crate release is necessary (design.html DEP-1).
- Keep each extension as an Extism guest (`extism-kernel-v1`) that speaks MEP
  `0.1`. The MoonBit worker host and the Rust CLI both run this guest.
- Keep the extension ids `morphir-avro` and `morphir-openapi`, the short ids
  `avro` and `openapi` and the tag scheme `extension/<short-id>/v<version>`. The
  first release from this repository of each extension is `0.3.0`. The extension
  homepage changes to finos/morphir-extensions. The artifact file names follow the
  extension id: `morphir-avro-0.3.0.wasm` and `morphir-openapi-0.3.0.wasm`.
- Add change-scoped Rust CI and one release workflow for every extension in this
  repository. It builds the bundle once, tests it, writes `release.json`
  (descriptor `2.0.0-draft.2`, claim set `0.1.0-draft.2`) and a `.sha256` file for
  each artifact, attests each artifact with `actions/attest@v4`, verifies the
  attestations and attaches the files to a GitHub release.
- Qualify each release bundle on the MoonBit worker host (native CLI and Node
  worker thread) and on the Rust CLI host compatibility check before its tag.
- Describe the follow-up changes in finos/morphir-rust (remove the crates and
  their release tasks) and finos/morphir (bundle pins, tests, docs and the bead).
  Make them only as this change lists them.

## Capabilities

### New Capabilities

- `extension-bundle-release`: the released bundle of any extension in this
  repository. It covers the files in a release, the descriptor and its claims,
  checksums, attestations, the tag scheme and the checks before publication.
- `avro-extension`: the Avro guest that hosts load. It covers its identity, its
  guest ABI, the MEP operations it answers and the parity of the module with the
  native build.
- `openapi-extension`: the OpenAPI and JSON Schema guest that hosts load. It
  covers the same points for its two targets.

### Modified Capabilities

None. This repository has no specs yet.

## Tracking

- Shared beads tracker, label `repo:morphir-extensions`. Bead
  `morphir-ukvn.1.7` records this destination and the decisions in design.html.
- Damian gave blanket approval for this work on 2026-10-31 (one OpenAPI
  extension, self-approval of this change, merges after green CI, crate
  publishing if necessary and the two extension tags). Decisions that need him
  go to his main session and are marked in design.html as "decided by agents,
  pending Damian review". Damian ratified all three (RUN-1, ID-1, QUA-1) on
  2026-10-31.
- Precedent: finos/morphir-gleam change `move-gleam-extension`.

## Impact

- New: `openspec/`, `.config/mise/` (`config.toml`, `config.rust.toml`, tasks),
  `AGENTS.md`, `CLAUDE.md`, `CHANGELOG.md`, `.beads/`, skill and agent files,
  `rust/` (workspace, two imported crates, a test-only guest check crate),
  `.github/workflows/ci.yml`, `.github/workflows/extension-release.yml`,
  `.github/scripts/extension-bundle/`.
- Changed: `README.md` (replace the blueprint text), `.gitignore`,
  `renovate.json`.
- New dependencies: Rust 1.98.1 with the `wasm32-unknown-unknown` target,
  `wasm-tools`, Java for the Avro IDL check (Apache Avro Tools 1.12.2), bun for
  bundle assembly, the `extism` crate for the guest check (test only).
- Other repositories: finos/morphir-rust and finos/morphir change only in the
  follow-up pull requests listed in design.html FUP-1. finos/morphir-moonbit does
  not change.
