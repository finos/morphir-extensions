// Command line for the release workflow and the rust:bundle task.
//
//   bun cli.ts select <tag>
//       Check a release tag against rust/extensions.toml and the crate version.
//       Prints short-id, version, crate, extension-id, module and checks as
//       key=value lines (for $GITHUB_OUTPUT).
//   bun cli.ts list
//       Print the short id of each extension, one per line.
//   bun cli.ts info <short-id>
//       Print the same lines for one extension, without a tag.
//   bun cli.ts bundle <short-id> --wasm <file> --claims <file> --out <dir> --git-commit <sha>
//       Check the claims and write the four bundle files.

import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import {
  crateVersion,
  findExtension,
  parseTag,
  readBytes,
  readExtensions,
  writeBundle,
} from "./bundle.ts";

const root = resolve(import.meta.dir, "../../..");
const rust = join(root, "rust");

function flag(args: string[], name: string): string {
  const at = args.indexOf(`--${name}`);
  const value = at >= 0 ? args[at + 1] : undefined;
  if (!value) throw new Error(`--${name} is required`);
  return value;
}

function extensions() {
  return readExtensions(readFileSync(join(rust, "extensions.toml"), "utf8"));
}

function versionOf(crate: string): string {
  return crateVersion(readFileSync(join(rust, "crates", crate, "Cargo.toml"), "utf8"));
}

function print(shortId: string, version: string): void {
  const extension = findExtension(extensions(), shortId);
  const lines = {
    "short-id": shortId,
    version,
    crate: extension.crate,
    "extension-id": extension.extensionId,
    module: extension.module,
    checks: extension.checks.join(" "),
  };
  for (const [key, value] of Object.entries(lines)) console.log(`${key}=${value}`);
}

function select(tag: string): void {
  const { shortId, version } = parseTag(tag);
  const crate = versionOf(findExtension(extensions(), shortId).crate);
  if (crate !== version) {
    throw new Error(`tag ${tag} has version ${version}, but the crate version is ${crate}`);
  }
  print(shortId, version);
}

function info(shortId: string): void {
  print(shortId, versionOf(findExtension(extensions(), shortId).crate));
}

function bundle(args: string[]): void {
  const [shortId] = args;
  if (!shortId || shortId.startsWith("--")) throw new Error("the short id is required");
  const extension = findExtension(extensions(), shortId);
  const out = resolve(flag(args, "out"));
  const files = writeBundle(out, {
    extension,
    version: versionOf(extension.crate),
    gitCommit: flag(args, "git-commit"),
    wasm: readBytes(resolve(flag(args, "wasm"))),
    claims: JSON.parse(readFileSync(resolve(flag(args, "claims")), "utf8")),
    expectedClaims: JSON.parse(
      readFileSync(join(rust, "crates", extension.crate, "claims", "expected.json"), "utf8"),
    ),
  });
  console.log(`bundle ${extension.extensionId} in ${out}:`);
  for (const file of files) console.log(`  ${file}`);
}

try {
  const [command, ...args] = process.argv.slice(2);
  if (command === "select" && args.length === 1) select(args[0]);
  else if (command === "info" && args.length === 1) info(args[0]);
  else if (command === "list" && args.length === 0)
    for (const extension of extensions()) console.log(extension.shortId);
  else if (command === "bundle") bundle(args);
  else throw new Error("usage: cli.ts select <tag> | cli.ts list | cli.ts info <short-id> | cli.ts bundle <short-id> --wasm --claims --out --git-commit");
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
