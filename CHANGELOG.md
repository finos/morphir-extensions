# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Each extension has its own version and release tag `extension/<short-id>/v<version>`.

## [Unreleased]

### Added

- Repository setup: the shared OpenSpec `html-design` schema, pinned mise tools and
  tasks, agent instructions, the shared beads tracker configuration, the
  pr-monitor and build-master skills, and CI.
- OpenSpec change `move-avro-and-openapi-extensions`: move the Avro and the OpenAPI
  and JSON Schema extensions here from finos/morphir-rust and release them as
  attested bundles.
- `rust/`: the Avro extension (`morphir-avro-extension`) and the OpenAPI and JSON
  Schema extension (`morphir-openapi-extension`), imported with their history from
  finos/morphir-rust at `945486e`. They build against the published Morphir crates
  from crates.io. Rust CI jobs run when `rust/` or its tooling changes.
