// Assembles the release bundle of one extension of this repository: the module,
// release.json and a .sha256 file for each (OpenSpec spec extension-bundle-release).
//
// The claims come from a live describe call to the exact module bytes; the
// caller passes them in. This file only checks and writes. When the MoonBit CLI
// has `morphir extension publish --stage`, that command replaces this script.

import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

export const SCHEMA_VERSION = "2.0.0-draft.2";
export const CLAIMS_VERSION = "0.1.0-draft.2";

export interface Extension {
  shortId: string;
  crate: string;
  extensionId: string;
  module: string;
  checks: string[];
}

export interface BundleInput {
  extension: Extension;
  version: string;
  gitCommit: string;
  wasm: Uint8Array;
  claims: unknown;
  expectedClaims: unknown;
}

export function sha256(bytes: Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}

/** Read every extension that rust/extensions.toml declares. */
export function readExtensions(toml: string): Extension[] {
  const parsed = Bun.TOML.parse(toml) as { extensions?: Record<string, Record<string, unknown>> };
  const table = parsed.extensions ?? {};
  return Object.entries(table).map(([shortId, entry]) => {
    const text = (key: string): string => {
      const value = entry[key];
      if (typeof value !== "string" || value === "") {
        throw new Error(`extensions.${shortId}.${key} must be a non-empty string`);
      }
      return value;
    };
    const checks = entry.checks ?? [];
    if (!Array.isArray(checks) || !checks.every((check) => typeof check === "string")) {
      throw new Error(`extensions.${shortId}.checks must be a list of task names`);
    }
    if (!/^[a-z][a-z0-9-]*$/.test(shortId)) {
      throw new Error(`short id ${JSON.stringify(shortId)} must be lowercase letters, digits and -`);
    }
    return {
      shortId,
      crate: text("crate"),
      extensionId: text("extension-id"),
      module: text("module"),
      checks: checks as string[],
    };
  });
}

/** Find the extension that a short id names, or fail with the known ids. */
export function findExtension(extensions: Extension[], shortId: string): Extension {
  const found = extensions.find((extension) => extension.shortId === shortId);
  if (!found) {
    const known = extensions.map((extension) => extension.shortId).join(", ");
    throw new Error(`no extension has the short id ${JSON.stringify(shortId)} (known: ${known})`);
  }
  return found;
}

/** Split a release tag extension/<short-id>/v<version>. */
export function parseTag(tag: string): { shortId: string; version: string } {
  const match = /^extension\/([a-z][a-z0-9-]*)\/v(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)$/.exec(tag);
  if (!match) {
    throw new Error(`tag ${JSON.stringify(tag)} is not extension/<short-id>/v<version>`);
  }
  return { shortId: match[1], version: match[2] };
}

/** Read the package version from a crate's Cargo.toml. */
export function crateVersion(cargoToml: string): string {
  const parsed = Bun.TOML.parse(cargoToml) as { package?: { version?: unknown } };
  const version = parsed.package?.version;
  if (typeof version !== "string") {
    throw new Error("the crate has no literal package.version");
  }
  return version;
}

function canonical(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") {
    const entries = Object.entries(value as Record<string, unknown>).sort(([a], [b]) =>
      a < b ? -1 : a > b ? 1 : 0,
    );
    return `{${entries.map(([k, v]) => `${JSON.stringify(k)}:${canonical(v)}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

/** Check the live claims against the extension, the version and the committed claims. */
export function checkClaims(input: BundleInput): void {
  const claims = input.claims as {
    claimsVersion?: unknown;
    extension?: { id?: unknown; version?: unknown };
  };
  if (claims?.claimsVersion !== CLAIMS_VERSION) {
    throw new Error(`claimsVersion must be ${CLAIMS_VERSION}, not ${JSON.stringify(claims?.claimsVersion)}`);
  }
  if (claims.extension?.id !== input.extension.extensionId) {
    throw new Error(
      `the module describes ${JSON.stringify(claims.extension?.id)}, not ${input.extension.extensionId}`,
    );
  }
  if (claims.extension?.version !== input.version) {
    throw new Error(
      `the module describes version ${JSON.stringify(claims.extension?.version)}, not ${input.version}`,
    );
  }
  if (canonical(input.claims) !== canonical(input.expectedClaims)) {
    throw new Error(
      `the claims of the module differ from claims/expected.json of ${input.extension.crate}`,
    );
  }
}

export function fileNames(extension: Extension, version: string) {
  const base = `${extension.extensionId}-${version}`;
  return { wasm: `${base}.wasm`, descriptor: `${base}.release.json` };
}

/** The release.json document for the bundle. */
export function descriptor(input: BundleInput): Record<string, unknown> {
  if (!/^[0-9a-f]{40}$/.test(input.gitCommit)) {
    throw new Error(`git commit ${JSON.stringify(input.gitCommit)} is not 40 lowercase hex digits`);
  }
  return {
    schemaVersion: SCHEMA_VERSION,
    extensionId: input.extension.extensionId,
    shortId: input.extension.shortId,
    version: input.version,
    gitCommit: input.gitCommit,
    artifacts: [
      {
        runtime: "wasm",
        filename: fileNames(input.extension, input.version).wasm,
        sha256: sha256(input.wasm),
        claims: input.claims,
      },
    ],
  };
}

function checksumLine(bytes: Uint8Array, name: string): string {
  return `${sha256(bytes)}  ${name}\n`;
}

/** Check the input and write the four bundle files to `out`. Returns their names. */
export function writeBundle(out: string, input: BundleInput): string[] {
  checkClaims(input);
  const names = fileNames(input.extension, input.version);
  const release = new TextEncoder().encode(`${JSON.stringify(descriptor(input), null, 2)}\n`);
  mkdirSync(out, { recursive: true });
  const files: [string, Uint8Array | string][] = [
    [names.wasm, input.wasm],
    [`${names.wasm}.sha256`, checksumLine(input.wasm, names.wasm)],
    [names.descriptor, release],
    [`${names.descriptor}.sha256`, checksumLine(release, names.descriptor)],
  ];
  for (const [name, content] of files) writeFileSync(join(out, name), content);
  return files.map(([name]) => name);
}

/** Read a file as bytes. */
export function readBytes(path: string): Uint8Array {
  return new Uint8Array(readFileSync(path));
}
