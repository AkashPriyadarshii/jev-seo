#!/usr/bin/env node
// Minimal launcher: resolves the prebuilt jev-seo binary for this
// platform (real GitHub release artifacts only) and execs it.
// No MCP reimplementation. Env and args pass straight through.
"use strict";

const { spawnSync, spawn } = require("node:child_process");
const { createWriteStream, existsSync, mkdirSync, chmodSync } = require("node:fs");
const { tmpdir, homedir } = require("node:os");
const { join } = require("node:path");
const https = require("node:https");

const VERSION = "0.1.2";
const REPO = "AkashPriyadarshii/jev-seo";

// Only targets with real published release assets. darwin/x64 is
// deliberately absent: no x86_64-apple-darwin archive exists.
const TARGETS = {
  "linux:x64": { target: "x86_64-unknown-linux-gnu", ext: "tar.gz" },
  "linux:arm64": { target: "aarch64-unknown-linux-gnu", ext: "tar.gz" },
  "darwin:arm64": { target: "aarch64-apple-darwin", ext: "tar.gz" },
  "win32:x64": { target: "x86_64-pc-windows-msvc", ext: "zip", exe: true },
};

function platformTarget() {
  const key = `${process.platform}:${process.arch}`;
  const entry = TARGETS[key];
  if (!entry) {
    const supported = Object.keys(TARGETS).join(", ");
    console.error(
      `jev-seo: unsupported platform ${key}. Supported: ${supported}. ` +
        `Override with JEV_SEO_BIN=/path/to/jev-seo if you built from source.`
    );
    process.exit(1);
  }
  return entry;
}

function binName(entry) {
  return entry.exe ? "jev-seo.exe" : "jev-seo";
}

function cacheDir() {
  const base =
    process.env.JEV_SEO_CACHE ||
    process.env.XDG_CACHE_HOME ||
    join(homedir(), ".cache");
  return join(base, "jev-seo", VERSION);
}

function resolveLocalBin(entry) {
  const override = process.env.JEV_SEO_BIN;
  if (override) {
    if (existsSync(override)) return override;
    console.error(`jev-seo: JEV_SEO_BIN=${override} not found.`);
    process.exit(1);
  }
  const candidates = [
    join(__dirname, "..", "bin", binName(entry)),
    join(cacheDir(), binName(entry)),
  ];
  for (const p of candidates) if (existsSync(p)) return p;
  return null;
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const req = https.get(
      url,
      { headers: { "User-Agent": "jev-seo-npm" } },
      (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          res.resume();
          return download(res.headers.location, dest).then(resolve, reject);
        }
        if (res.statusCode !== 200) {
          res.resume();
          return reject(new Error(`download failed: ${url} -> HTTP ${res.statusCode}`));
        }
        const out = createWriteStream(dest);
        res.pipe(out);
        out.on("finish", () => resolve(dest));
        out.on("error", reject);
      }
    );
    req.on("error", reject);
  });
}

function extract(archive, destDir, entry) {
  const tool = entry.exe ? "unzip" : "tar";
  const args = entry.exe
    ? ["-o", "-j", archive, "-d", destDir]
    : ["-xzf", archive, "-C", destDir];
  const r = spawnSync(tool, args, { stdio: "inherit" });
  if (r.status !== 0) {
    console.error(`jev-seo: failed to extract ${archive} (needs ${tool} on PATH).`);
    process.exit(1);
  }
}

async function ensureBin(entry) {
  const local = resolveLocalBin(entry);
  if (local) return checkBin(local);
  const archive = `jev-seo-${entry.target}.${entry.ext}`;
  const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${archive}`;
  const dir = cacheDir();
  mkdirSync(dir, { recursive: true });
  const tmpFile = join(tmpdir(), archive);
  console.error(`jev-seo: fetching ${url}`);
  try {
    await download(url, tmpFile);
  } catch (e) {
    console.error(`jev-seo: ${e.message}`);
    console.error(`jev-seo: network needed on first run, or set JEV_SEO_BIN.`);
    process.exit(1);
  }
  extract(tmpFile, dir, entry);
  const bin = join(dir, binName(entry));
  if (!existsSync(bin)) {
    console.error(`jev-seo: archive ${archive} did not contain ${binName(entry)}.`);
    process.exit(1);
  }
  if (!entry.exe) {
    try {
      chmodSync(bin, 0o755);
    } catch {}
  }
  return checkBin(bin);
}

/// The download can succeed yet the binary still not run here (musl
/// systems vs glibc builds, corrupt cache). Fail clearly instead of
/// leaking a system relocation error to the agent.
function checkBin(bin) {
  const r = spawnSync(bin, ["--version"], { stdio: "pipe" });
  if (r.status === 0) return bin;
  console.error(
    `jev-seo: binary at ${bin} failed to execute. ` +
      `Prebuilt releases need glibc; musl/Alpine systems must build from ` +
      `source (cargo build) and set JEV_SEO_BIN=/path/to/jev-seo. ` +
      `Delete the cache dir above to force a fresh download.`
  );
  process.exit(1);
}

async function main() {
  const entry = platformTarget();
  const bin = await ensureBin(entry);
  // Bare `npx -y jev-seo` starts the MCP server over stdio.
  const args = process.argv.slice(2);
  const child = spawn(bin, args.length ? args : ["mcp"], { stdio: "inherit" });
  child.on("exit", (code, signal) => {
    if (signal) {
      console.error(`jev-seo: binary killed by ${signal}`);
      process.exit(1);
    }
    process.exit(code ?? 1);
  });
  child.on("error", (e) => {
    console.error(`jev-seo: cannot execute ${bin}: ${e.message}`);
    process.exit(1);
  });
}

main();
