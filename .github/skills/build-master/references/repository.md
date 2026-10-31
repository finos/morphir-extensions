# Morphir Extensions build context

Read root `AGENTS.md`. This checkout carries the complete build-master skill, so contributors and hosted agents do not need a personal installation. Edit `.github/skills/build-master/`; `.agents/skills/`, `.claude/skills/` and `.grok/skills/` contain discovery links.

## Find the execution owners

- `.github/workflows/ci.yml` owns routine CI. Its `changes` job selects the path-scoped jobs from the changed files. Its `ci-ok` job always runs, needs every other job and is the one check to require; a skipped job must not block a merge.
- `.github/workflows/extension-release.yml` (when present) owns the release of every extension. A tag `extension/<short-id>/v<version>` selects one extension from `rust/extensions.toml`. The check job refuses a wrong version, a tag that is not on `main` and an existing release before any build. The module is built once and every later job uses that artifact.
- `.config/mise/config.toml` pins the repository tools. Language tools are in their own environment (`.config/mise/config.rust.toml`, `MISE_ENV=rust`), so the OpenSpec and lint jobs do not install Rust.
- The OpenSpec checks (`mise run openspec:check`, `openspec:validate`, `openspec:test`) need the pinned MoonBit toolchain only for the shared snapshot checker.

This is a navigation map, not an assertion that all existing placements are optimal. Use current files and run evidence to decide.

## Scarce runners

Treat macOS runners and non-standard Windows labels as scarce shared capacity. The extension guests are portable Wasm, so pull-request and release jobs run on Linux. Platform qualification on the MoonBit worker host (native and Node) runs by hand before a tag and is recorded in the pull request that prepares the tag. Send every decision that adds such a runner to the build-master.

## Repository constraints

Keep workflow steps declarative and put logic that needs tests in mise tasks or in the bundle script under `.github/scripts/`, with its own tests. Pin every action by major version and every tool in mise.

Run `actionlint` and the checks for the files you changed before pushing. Use the canonical instructions in `AGENTS.md` for the exact current commands.

Keep human-reviewed designs in OpenSpec `design.html` files and the shared beads workflow. A short review can return directly to its coordinator without creating a report. Scope, tracker updates, merges, releases and deployment authority follow repository instructions and the user's existing authorization.
