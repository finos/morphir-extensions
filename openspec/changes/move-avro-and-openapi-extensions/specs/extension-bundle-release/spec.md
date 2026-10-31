# Spec Delta

## Purpose

Defines the attested bundle that this repository publishes for each extension, so
that a repository index and the Morphir CLI can verify and install it.

## ADDED Requirements

### Requirement: Release tag selects one extension
A release SHALL start from a tag `extension/<short-id>/v<version>` on a commit that
is reachable from `main`. The short id SHALL name an extension that this
repository declares, and the version SHALL equal the version of that extension's
crate. The workflow SHALL refuse the release before it builds anything otherwise.

#### Scenario: Tag version differs from the crate version
- **WHEN** the tag is `extension/avro/v0.3.1` and the Avro crate version is `0.3.0`
- **THEN** the check job fails and no build, attestation or release occurs

#### Scenario: Tag commit is not on main
- **WHEN** the tag points to a commit that is not reachable from `main`
- **THEN** the check job fails and no release occurs

#### Scenario: Unknown short id
- **WHEN** the tag is `extension/protobuf/v0.1.0` and no extension has the short id `protobuf`
- **THEN** the check job fails and names the short id

### Requirement: Release files
A published release SHALL hold exactly four files: `<extension-id>-<version>.wasm`,
`<extension-id>-<version>.release.json` and one `.sha256` file for each of them.
Each `.sha256` file SHALL be in `sha256sum` format and name its file without a
directory.

#### Scenario: Avro 0.3.0 release
- **WHEN** the release for `extension/avro/v0.3.0` is published
- **THEN** it holds `morphir-avro-0.3.0.wasm`, `morphir-avro-0.3.0.wasm.sha256`, `morphir-avro-0.3.0.release.json` and `morphir-avro-0.3.0.release.json.sha256`, and `sha256sum --check` passes on the two `.sha256` files

### Requirement: Release descriptor
`release.json` SHALL have `schemaVersion` `2.0.0-draft.2`, the `extensionId`,
`shortId` and `version` of the extension, the 40-hex `gitCommit` of the tag, and
one artifact entry with `runtime` `wasm`, the artifact `filename`, its `sha256`
and the `claims` of the exact module bytes.

#### Scenario: Descriptor matches the module
- **WHEN** a reader computes the SHA-256 of the released `.wasm`
- **THEN** it equals the `sha256` of the single artifact entry in `release.json`, and the entry's `filename` names that file

### Requirement: Claims come from the released bytes
The claims in `release.json` SHALL be read from a live describe call to the exact
module that is released, with claim set version `0.1.0-draft.2`. They SHALL equal
the expected claims committed for that extension. The release SHALL fail if they
differ.

#### Scenario: Capability change without an expected-claims update
- **WHEN** a change adds a target to an extension but does not update its committed expected claims
- **THEN** CI and the release workflow fail on the claims check

### Requirement: Artifact attestations
Each released `.wasm` and `release.json` SHALL have a build provenance
attestation from the workflow `.github/workflows/extension-release.yml` of
finos/morphir-extensions. The workflow SHALL verify each attestation before it
creates the GitHub release.

#### Scenario: Verify a downloaded module
- **WHEN** a user runs `gh attestation verify morphir-avro-0.3.0.wasm --repo finos/morphir-extensions --signer-workflow finos/morphir-extensions/.github/workflows/extension-release.yml`
- **THEN** the verification passes

### Requirement: Releases are not replaced
The workflow SHALL NOT replace or add files to an existing GitHub release. A
release SHALL NOT be marked as the latest release of the repository.

#### Scenario: Release already exists
- **WHEN** the release for the tag already exists
- **THEN** the publish job fails and the existing release does not change

### Requirement: Dry run
A manual run with `publish` set to false SHALL run every job except attestation
and publication, and SHALL upload the assembled bundle as a workflow artifact.

#### Scenario: Dry run of the OpenAPI extension
- **WHEN** a maintainer runs the release workflow by hand for `extension/openapi/v0.3.0` with `publish: false`
- **THEN** the run uploads the four bundle files as an artifact and creates no attestation, tag or release
