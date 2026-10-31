[![FINOS - Incubating](https://cdn.jsdelivr.net/gh/finos/contrib-toolbox@master/images/badge-incubating.svg)](https://community.finos.org/docs/governance/lifecycle-stages/incubating)

# Morphir Extensions

General-purpose extensions for [Morphir](https://morphir.finos.org). An extension
here does not belong to one language repository: it projects a Morphir model into a
schema or interface format, for example. Each extension is released from this
repository as an attested bundle that a Morphir CLI can verify and install.

The main Morphir CLI is the MoonBit CLI,
[finos/morphir-moonbit](https://github.com/finos/morphir-moonbit). It runs any
extension that conforms to the Morphir Extension Protocol (MEP).

## Extensions

| Extension | Id | Targets | Release tags |
| --- | --- | --- | --- |
| Avro | `morphir-avro` | `avro` | `extension/avro/v*` |
| OpenAPI and JSON Schema | `morphir-openapi` | `openapi`, `json-schema` | `extension/openapi/v*` |

The two extensions moved here from
[finos/morphir-rust](https://github.com/finos/morphir-rust) (OpenSpec change
[`move-avro-and-openapi-extensions`](openspec/changes/move-avro-and-openapi-extensions/proposal.md)).
Their first release from this repository is `0.3.0`. Releases up to `0.2.0` stay in
finos/morphir-rust. Each module is an Extism guest (`extism-kernel-v1`) for MEP
`0.1`; the MoonBit CLI runs it from its native and its Node CLI.

## Releases

Each release of an extension is a GitHub release with four files:

| File | Content |
| --- | --- |
| `<extension-id>-<version>.wasm` | The guest module |
| `<extension-id>-<version>.wasm.sha256` | Its SHA-256, in `sha256sum` format |
| `<extension-id>-<version>.release.json` | Descriptor `2.0.0-draft.2` with the claims of the module |
| `<extension-id>-<version>.release.json.sha256` | Its SHA-256 |

The module and `release.json` have build provenance attestations. To verify a
download:

```sh
gh release download extension/avro/v0.3.0 --repo finos/morphir-extensions --dir avro
cd avro
sha256sum --check ./*.sha256
for file in morphir-avro-0.3.0.wasm morphir-avro-0.3.0.release.json; do
  gh attestation verify "$file" --repo finos/morphir-extensions \
    --signer-workflow finos/morphir-extensions/.github/workflows/extension-release.yml
done
```

To release an extension (maintainers only):

1. Set the crate version, update `claims/expected.json` of the crate and
   `CHANGELOG.md`, and merge to `main`.
2. Run the release workflow as a dry run:
   `gh workflow run extension-release.yml -f tag=extension/<short-id>/v<version> -f publish=false`.
3. Download its bundle artifact and qualify it on the MoonBit worker host:
   `mise run qualify:moonbit -- --bundle <dir> --native <morphir.exe> --node <morphir.js> --cli-commit <sha>`.
4. Push the tag `extension/<short-id>/v<version>` on `main`. The workflow checks
   the version and the commit, builds the module once, tests it, writes the
   bundle, attests it, verifies the attestations and creates the release. It
   never replaces an existing release.

A new extension needs only an entry in `rust/extensions.toml` and a
`claims/expected.json`; the workflow does not change.

## Development setup

Install [mise](https://mise.jdx.dev), then:

```sh
mise install                  # pinned repository tools
mise run openspec:check       # managed OpenSpec snapshot is unchanged
mise run openspec:validate    # schema and every change, strict
mise run openspec:test        # tests of the shared checker
```

The Rust extensions in `rust/` use their own mise environment:

```sh
MISE_ENV=rust mise install
MISE_ENV=rust mise run rust:test    # unit, golden and integration tests
MISE_ENV=rust mise run rust:wasm    # build and check the Wasm guests
MISE_ENV=rust mise run rust:guest-check
```

See [AGENTS.md](AGENTS.md) for all tasks, the OpenSpec and beads workflow and the
contribution rules.

## Contributing

For questions, bugs or feature requests, open an
[issue](https://github.com/finos/morphir-extensions/issues).

To submit a contribution:

1. Fork it (<https://github.com/finos/morphir-extensions/fork>)
2. Create your feature branch (`git checkout -b feature/fooBar`)
3. Read our [contribution guidelines](CONTRIBUTING.md) and [Community Code of Conduct](https://www.finos.org/code-of-conduct)
4. Commit your changes (`git commit -am 'Add some fooBar'`)
5. Push to the branch (`git push origin feature/fooBar`)
6. Create a new Pull Request

_NOTE:_ Commits and pull requests to FINOS repositories are accepted only from
contributors who have signed a CLA through
[EasyCLA](https://community.finos.org/docs/governance/Software-Projects/easycla).
Read [FINOS Contribution Requirements](https://community.finos.org/docs/governance/Software-Projects/contribution-compliance-requirements)
before you contribute.

*Questions about CLA, DCO, or EasyCLA? Email [help@finos.org](mailto:help@finos.org)*

## License

Copyright 2026 FINOS

Distributed under the [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0).

SPDX-License-Identifier: [Apache-2.0](https://spdx.org/licenses/Apache-2.0)
