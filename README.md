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

The two extensions are moving here from
[finos/morphir-rust](https://github.com/finos/morphir-rust) (OpenSpec change
[`move-avro-and-openapi-extensions`](openspec/changes/move-avro-and-openapi-extensions/proposal.md)).
Until their first release from this repository, use the `0.2.0` releases in
finos/morphir-rust.

## Development setup

Install [mise](https://mise.jdx.dev), then:

```sh
mise install                  # pinned repository tools
mise run openspec:check       # managed OpenSpec snapshot is unchanged
mise run openspec:validate    # schema and every change, strict
mise run openspec:test        # tests of the shared checker
```

See [AGENTS.md](AGENTS.md) for the OpenSpec and beads workflow and the
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
