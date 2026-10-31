# Tasks

Tracking: bead `morphir-ukvn.1.7` in the shared tracker (label
`repo:morphir-extensions`). Each numbered group lands as one pull request against
`main`, in order. Damian's blanket approval of 2026-10-31 covers the merges, the
tags `extension/avro/v0.3.0` and `extension/openapi/v0.3.0` and the follow-up pull
requests in FUP-1.

## 1. Repository setup (PR 1)

- [x] 1.1 Add the shared OpenSpec bundle (`openspec/config.yaml`, `openspec/schemas/html-design`, `openspec/extensions.lock.json`, `scripts/openspec-ext.mbtx`, its feature file) and the `openspec:*` tasks. Verify: `mise run openspec:check`, `mise run openspec:validate` and `mise run openspec:test` pass with MoonBit 0.10.18.
- [x] 1.2 Add `.config/mise/config.toml` with the pins from design.html SET-1. Verify: `mise install` in a clean checkout installs node, OpenSpec, bun, actionlint and MoonBit `0.10.18+6c83a7f62`, and no Rust.
- [x] 1.3 Add `AGENTS.md`, `CLAUDE.md`, `CHANGELOG.md`, `.beads/config.yaml` and `.beads/.gitignore`, the pr-monitor skill (`skills-lock.json`) and the build-master skill and agent with a repository reference for this repository. Verify: `.beads/config.yaml` names the shared tracker as in finos/morphir-python, `bd where` resolves it, and every skill link resolves.
- [x] 1.4 Replace the blueprint `README.md` with a description of the repository and its extensions. Verify: the README has no blueprint tokens.
- [x] 1.5 Add `.github/workflows/ci.yml` with the `changes`, `openspec`, `actionlint` and `ci-ok` jobs (design.html CI-1). Verify: `actionlint` passes and the PR shows the jobs green.
- [x] 1.6 Add this change and validate it. Verify: `openspec validate move-avro-and-openapi-extensions --strict` passes.

## 2. Import the crates with their history (PR 2)

- [x] 2.1 Record the finos/morphir-rust import revision in the PR description. Verify: no commit on finos/morphir-rust `main` touches either crate after it when PR 2 is ready.
- [x] 2.2 Run `git filter-repo` for each crate with the path rename to `rust/crates/<crate>/` and the message callback from design.html HIS-1. Verify: no AI co-author trailers, every commit has an `Imported-from:` trailer, and the counts are 14 (Avro) and 32 (OpenAPI).
- [x] 2.3 Merge each filtered history with `--allow-unrelated-histories --no-ff`. Verify: `git log --follow rust/crates/morphir-avro-extension/src/lib.rs` shows the imported commits.
- [x] 2.4 Add `rust/Cargo.toml` (workspace package values from finos/morphir-rust, repository URL of this repository), `rust/rust-toolchain.toml`, `rust/Cargo.lock` and `/rust/target/` in `.gitignore`. Change the path dependencies to the crates.io versions from design.html DEP-1. Verify: `cargo build` and `cargo build --target wasm32-unknown-unknown` pass in `rust/`.
- [x] 2.5 Copy `canonical-customer-v4.json` into each crate's `tests/fixtures/`, change the two `include_str!` paths, and change the README links to absolute finos/morphir-rust links. Verify: `cargo test --locked` passes all imported tests with no file outside `rust/`.
- [x] 2.6 Add `.config/mise/config.rust.toml` and the tasks `rust:fmt`, `rust:lint`, `rust:test` and `rust:wasm` (build, `wasm-tools validate`, import check). Verify: `MISE_ENV=rust mise run rust:test` and `rust:wasm` pass, and plain `mise install` still installs no Rust.
- [x] 2.7 Add the `rust` and `wasm` jobs to `ci.yml` behind the change selection. Renovate's `config:recommended` already updates `rust/` (Cargo), the mise pins and the actions, so no new rule is needed. Verify: the PR shows the jobs green and `ci-ok` passes; `npx --yes --package renovate -- renovate-config-validator` passes.
- [x] 2.8 Add a "Rust extensions" section to `AGENTS.md` and `README.md` and an `Added` entry in `CHANGELOG.md`. Verify: the documented commands run as written.
- [x] 2.9 Merge PR 2 with a merge commit. Verify: `git log main -- rust/crates/morphir-openapi-extension` shows the imported commits.

## 3. Guest check, release workflow and 0.3.0 (PR 3)

- [x] 3.1 Add `rust/extensions.toml` with the two extensions (design.html LAY-1). Verify: the bundle script test reads both entries.
- [x] 3.2 Add the test-only crate `extension-guest-check` (Extism host, MEP 0.1 session, golden parity, claims output). Verify: the parity tests pass for both modules, and the binary prints claims that equal a describe call.
- [x] 3.3 Set the homepage of both extensions to finos/morphir-extensions and the crate versions to `0.3.0`; add `claims/expected.json` to each crate and the task `rust:guest-check`. Verify: changing a target without updating `claims/expected.json` makes the task fail.
- [x] 3.4 Port the Avro IDL check as the task `rust:avro-idl` (Apache Avro Tools 1.12.2 with its SHA-512, Java from `config.rust.toml`) and add the `avro-idl` CI job. Verify: the job passes on the IDL goldens.
- [x] 3.5 Add the bun bundle script `.github/scripts/extension-bundle/` and the task `rust:bundle`; it writes the four files of the extension-bundle-release spec and runs `sha256sum --check`. Verify: `bun test` checks the file set and the descriptor fields, and the `wasm` job uploads a dry-run bundle for each extension.
- [x] 3.6 Add `.github/workflows/extension-release.yml` with the check, build, test and publish jobs and the permissions from design.html REL-1, with the stage step and the publish job kept separate for the later `morphir extension publish` switch. Verify: `actionlint` passes.
- [ ] 3.7 Add the task `qualify:moonbit` (design.html QUA-1). Verify: it runs against a local build of the dry-run bundles with a MoonBit CLI from finos/morphir-moonbit `main` and writes its JSON record.
- [x] 3.8 Document the release steps, the bundle files and the `gh attestation verify` command in `README.md`, and add `Changed` entries in `CHANGELOG.md`. Verify: the verify command names the workflow path.

## 4. Releases

- [ ] 4.1 After PR 3 merges, run the release workflow from `main` with `publish: false` for both tags. Verify: all jobs pass and each bundle artifact matches the extension-bundle-release spec; a run with a wrong version fails in check.
- [ ] 4.2 Run `qualify:moonbit` on both dry-run bundles (native and Node CLI) and the finos/morphir host compatibility test with the dry-run modules. Verify: the records show exit 0 for describe and generate; post them in the PR or issue that prepares the tags.
- [ ] 4.3 Push `extension/avro/v0.3.0` and `extension/openapi/v0.3.0` on `main`. Verify: each release has the four files, `sha256sum --check` passes, and `gh attestation verify` passes for the module and `release.json`. Send the release URLs to Damian's main session.
- [ ] 4.4 Run `mise run openspec:validate` and `openspec validate move-avro-and-openapi-extensions --strict`. Verify: both pass.

## 5. Follow-up in other repositories (FUP-1)

- [ ] 5.1 finos/morphir: move the `avro` and `openapi` bundle pins to finos/morphir-extensions 0.3.0, change the two generate tests to use the pinned bundles, and update `docs/generate/avro.md`, `json-schema.md`, `openapi.md` and the release-manager skill. Verify: its CI, including host compatibility, passes.
- [ ] 5.2 finos/morphir-rust: remove the two crates and their release tasks, registry entries, CI references and host-native fixtures, and leave pointers. Verify: its CI passes and the 0.2.0 releases still exist.
- [ ] 5.3 Close bead `morphir-ukvn.1.7` with a pointer to this change and the releases. Verify: `bd show morphir-ukvn.1.7` shows it closed.

## Workflow follow-up

- Archive this change after the releases and the follow-up pull requests merge,
  following AGENTS.md.
- Ask Damian to review the decisions marked "decided by agents, pending Damian
  review" in design.html.
