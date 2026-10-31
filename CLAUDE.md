# Claude Code instructions

Follow [AGENTS.md](AGENTS.md). It holds the development guidance, the OpenSpec and
beads workflow, and the contribution rules for this repository.

## Quick reference

- **No AI co-authors or attribution** in commits, pull requests or issues. FINOS
  EasyCLA rejects them.
- **Published crates only**: use the Morphir crates from crates.io, never a git or
  path dependency on another repository.
- **OpenSpec** changes use the shared `html-design` schema; do not edit its managed
  snapshot.
- **CI and releases** go through the build-master skill; watch your pull requests
  with the pr-monitor skill.
