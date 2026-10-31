# AGENTS.md

AI coding assistants (Claude, Copilot, Cursor and others) follow this guidance in
the finos/morphir-extensions repository.

## Project overview

**finos/morphir-extensions** holds general-purpose
[Morphir](https://morphir.finos.org) extensions: extensions that do not belong to
one language repository, such as schema and interface backends. Each extension is
released from here as an attested bundle that a Morphir CLI can verify and install.

- The main Morphir CLI is the MoonBit CLI,
  [finos/morphir-moonbit](https://github.com/finos/morphir-moonbit). Its worker host
  runs any extension that conforms to the Morphir Extension Protocol (MEP) and to a
  supported runtime. The Rust CLI in
  [finos/morphir-rust](https://github.com/finos/morphir-rust) is being sunset.
- The protocol and IR specifications are in
  [finos/morphir](https://github.com/finos/morphir) (`spec/mep`, `spec/ir`).
- An extension that belongs to one language (for example Gleam, Elm or Python)
  lives in that language's repository, not here.

## Repository structure

```
morphir-extensions/
├── openspec/            # OpenSpec changes and specs (shared html-design schema)
├── .config/mise/        # Tool pins and file-based tasks
│   ├── config.toml      # Repository tools (OpenSpec, MoonBit checker, bun, actionlint)
│   └── tasks/           # mise tasks
├── rust/                # Rust extensions (one Cargo workspace)
├── .github/             # Workflows, skills and agent profiles
├── .beads/              # Configuration for the shared beads tracker
├── docs/, website/      # FINOS project website
└── AGENTS.md            # This file
```

### Rust extensions

`rust/` is one Cargo workspace. Each crate under `rust/crates/` is one extension:

| Crate | Extension id | Targets |
| --- | --- | --- |
| `morphir-avro-extension` | `morphir-avro` | `avro` |
| `morphir-openapi-extension` | `morphir-openapi` | `openapi`, `json-schema` |

The crates and their history come from finos/morphir-rust (OpenSpec change
`move-avro-and-openapi-extensions`). Each builds to an Extism guest
(`wasm32-unknown-unknown`) that speaks MEP 0.1. Rust tools are in
`.config/mise/config.rust.toml`; select them with `MISE_ENV=rust`, so jobs that do
not build Rust do not install it.

```sh
MISE_ENV=rust mise run rust:fmt     # --fix to format
MISE_ENV=rust mise run rust:lint    # clippy, host and wasm32
MISE_ENV=rust mise run rust:test
MISE_ENV=rust mise run rust:wasm    # build, validate, import check
```

A guest may import only `extism:host/env`. `rust:wasm` fails on any other import.

## Development

```sh
mise install                  # repository tools
mise run openspec:check       # managed OpenSpec snapshot is unchanged
mise run openspec:validate    # schema and every change, strict
mise run openspec:test        # tests of the shared checker
actionlint                    # GitHub Actions workflows
```

### Dependencies

- Use the published Morphir crates from crates.io. If an extension needs an
  unpublished change, have finos/morphir-rust publish it first. Do not add a git
  or path dependency on another repository.
- Pin every tool in `.config/mise/`. The MoonBit toolchain is `0.10.18+6c83a7f62`;
  do not add pins or workarounds for an older version.

## OpenSpec workflow

Use the shared `html-design` schema for new change proposals. Start with the
`openspec-propose` skill, then use `openspec-apply-change`,
`openspec-update-change`, `openspec-sync-specs` and `openspec-archive-change` as
the change progresses. `openspec-explore` supports investigation before a
proposal.

Design artifacts are standalone `design.html` files with inline CSS and SVG. The
generated `openspec-*` skills say `design.md`; in this repository that file is
`design.html`. Keep Morphir context and FINOS contribution policy in
`openspec/config.yaml` and this file.

The schema, templates, checker and Gherkin feature are managed snapshots from
`moonrockz/mission-control`'s `apps/openspec-ext`, locked by
`openspec/extensions.lock.json`. Do not edit those files locally. Run
`mise run openspec:check`, `mise run openspec:validate` and
`mise run openspec:test` after OpenSpec changes. Run `mise run openspec:setup` to
regenerate the six core skills without changing personal OpenSpec settings.

## Issue tracking with beads

Work is tracked with [beads](https://github.com/steveyegge/beads) (`bd`) in the
**shared Morphir tracker**, not in a database of this repository. The database
lives in finos/morphir and syncs over its `refs/dolt/data` ref.
`.beads/config.yaml` points `bd` at it, so you do not need a finos/morphir
checkout.

```sh
bd bootstrap          # first time on a machine: clones the shared database
bd dolt pull          # before you start work
bd dolt push          # after you change issues
```

Run `bd bootstrap` in the main checkout before you open a worktree or start an
agent session. An empty local database hides the shared one; use `bd count` to
check before you move one aside.

Issues whose work happens here carry the label `repo:morphir-extensions`:

```sh
bd ready --label repo:morphir-extensions        # unblocked work
bd list --label repo:morphir-extensions --all   # everything, including closed
bd show <id> --long                             # description, design, notes and comments
```

## Decisions

When a decision needs the maintainer and an agent makes it under delegated
authority, record it in the change's `design.html` as "decided by agents, pending
Damian review" and as a comment on the bead, then continue.

## CI and releases

- Send every change to workflows, checks, matrices, caching or release steps
  through the build-master skill (`.github/skills/build-master/`).
- `.github/workflows/ci.yml` selects jobs from the changed paths. Its `ci-ok` job
  always runs and is the check to require.
- Each extension releases from a tag `extension/<short-id>/v<version>` on `main`.
  Create a tag only with maintainer approval. Never replace an existing release
  or tag.
- After you open or push to a pull request, watch it with the pr-monitor skill
  until it merges or closes.

## Git workflow

### Commit messages

Use conventional commits:

```
feat(avro): add logical type for decimal
fix(openapi): keep nullable references in 3.0 output
ci: verify release attestations
docs: describe the release bundle
```

Branches: `feature/<topic>`, `fix/<topic>`, `docs/<topic>`, `ci/<topic>`.

### CHANGELOG

Follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Add each change
under `[Unreleased]` in `CHANGELOG.md` with one of `Added`, `Changed`,
`Deprecated`, `Removed`, `Fixed` or `Security`.

## Critical compliance requirement

> **Do not add AI assistants (Claude, Copilot and others) as co-authors of commits,
> and do not add AI attribution to commits, pull requests or issues.**

FINOS EasyCLA checks every commit and blocks a pull request with such a line. Only
human developers are commit authors.

Incorrect:

```
Co-authored-by: Claude <claude@anthropic.com>
```

## Session completion checklist

- [ ] The checks for the files you changed pass (see Development).
- [ ] `CHANGELOG.md` is updated.
- [ ] Changes are committed with a descriptive message and pushed.
- [ ] The bead is updated and `bd dolt push` succeeded.

Work is not complete until `git push` succeeds.
