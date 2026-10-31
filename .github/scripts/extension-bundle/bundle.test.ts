import { describe, expect, test } from "bun:test";
import { mkdtempSync, readFileSync, readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import {
  type BundleInput,
  crateVersion,
  descriptor,
  findExtension,
  parseTag,
  readExtensions,
  sha256,
  writeBundle,
} from "./bundle.ts";

const root = resolve(import.meta.dir, "../../..");
const extensions = readExtensions(readFileSync(join(root, "rust/extensions.toml"), "utf8"));
const avro = findExtension(extensions, "avro");
const commit = "0123456789abcdef0123456789abcdef01234567";

function claims(id = "morphir-avro", version = "0.3.0") {
  return {
    claimsVersion: "0.1.0-draft.2",
    protocolVersions: ["0.1"],
    extension: { id, name: "Morphir Avro", version, types: ["backend"] },
    capabilities: { backend: { targets: ["avro"], irVersions: ["3", "4"], generate: true } },
  };
}

function input(overrides: Partial<BundleInput> = {}): BundleInput {
  return {
    extension: avro,
    version: "0.3.0",
    gitCommit: commit,
    wasm: new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0]),
    claims: claims(),
    expectedClaims: claims(),
    ...overrides,
  };
}

describe("rust/extensions.toml", () => {
  test("declares both extensions with their crates and modules", () => {
    expect(extensions.map((e) => e.shortId).sort()).toEqual(["avro", "openapi"]);
    expect(avro).toEqual({
      shortId: "avro",
      crate: "morphir-avro-extension",
      extensionId: "morphir-avro",
      module: "morphir_avro_extension.wasm",
      checks: ["rust:avro-idl"],
    });
    expect(findExtension(extensions, "openapi").extensionId).toBe("morphir-openapi");
  });

  test("each crate's version is readable and its expected claims agree", () => {
    for (const extension of extensions) {
      const version = crateVersion(
        readFileSync(join(root, "rust/crates", extension.crate, "Cargo.toml"), "utf8"),
      );
      const expected = JSON.parse(
        readFileSync(join(root, "rust/crates", extension.crate, "claims/expected.json"), "utf8"),
      );
      expect(expected.extension.id).toBe(extension.extensionId);
      expect(expected.extension.version).toBe(version);
    }
  });

  test("an unknown short id names the known ones", () => {
    expect(() => findExtension(extensions, "protobuf")).toThrow(/protobuf.*known: avro, openapi/);
  });
});

describe("release tags", () => {
  test("split into short id and version", () => {
    expect(parseTag("extension/avro/v0.3.0")).toEqual({ shortId: "avro", version: "0.3.0" });
    expect(parseTag("extension/openapi/v1.0.0-rc.1")).toEqual({
      shortId: "openapi",
      version: "1.0.0-rc.1",
    });
  });

  test("refuse other shapes", () => {
    for (const tag of ["v0.3.0", "extension/avro/0.3.0", "extension/Avro/v0.3.0", "extension/avro/v0.3"]) {
      expect(() => parseTag(tag)).toThrow(/extension\/<short-id>\/v<version>/);
    }
  });
});

describe("bundle", () => {
  test("writes exactly the four files with matching checksums", () => {
    const out = mkdtempSync(join(tmpdir(), "bundle-"));
    const written = writeBundle(out, input());
    expect(written).toEqual([
      "morphir-avro-0.3.0.wasm",
      "morphir-avro-0.3.0.wasm.sha256",
      "morphir-avro-0.3.0.release.json",
      "morphir-avro-0.3.0.release.json.sha256",
    ]);
    expect(readdirSync(out).sort()).toEqual([...written].sort());
    for (const name of ["morphir-avro-0.3.0.wasm", "morphir-avro-0.3.0.release.json"]) {
      const line = readFileSync(join(out, `${name}.sha256`), "utf8");
      expect(line).toBe(`${sha256(new Uint8Array(readFileSync(join(out, name))))}  ${name}\n`);
    }
  });

  test("release.json describes the module", () => {
    const out = mkdtempSync(join(tmpdir(), "bundle-"));
    const bundle = input();
    writeBundle(out, bundle);
    const release = JSON.parse(readFileSync(join(out, "morphir-avro-0.3.0.release.json"), "utf8"));
    expect(release).toEqual({
      schemaVersion: "2.0.0-draft.2",
      extensionId: "morphir-avro",
      shortId: "avro",
      version: "0.3.0",
      gitCommit: commit,
      artifacts: [
        {
          runtime: "wasm",
          filename: "morphir-avro-0.3.0.wasm",
          sha256: sha256(bundle.wasm),
          claims: claims(),
        },
      ],
    });
  });

  test("refuses claims that differ from the committed claims", () => {
    const changed = claims();
    changed.capabilities.backend.targets.push("protobuf");
    expect(() => writeBundle(mkdtempSync(join(tmpdir(), "bundle-")), input({ claims: changed }))).toThrow(
      /differ from claims\/expected.json/,
    );
  });

  test("refuses a module of another extension or version", () => {
    const out = mkdtempSync(join(tmpdir(), "bundle-"));
    expect(() => writeBundle(out, input({ claims: claims("morphir-openapi") }))).toThrow(/morphir-openapi/);
    expect(() => writeBundle(out, input({ claims: claims("morphir-avro", "0.2.0") }))).toThrow(/0\.2\.0/);
  });

  test("refuses a git commit that is not 40 hex digits", () => {
    expect(() => descriptor(input({ gitCommit: "HEAD" }))).toThrow(/40 lowercase hex/);
  });
});
