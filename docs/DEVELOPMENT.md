# Second Brain OS Development Guide

## Supported Development Platform

The current MVP supports macOS only. The Rust credential-store adapter uses
macOS Keychain and intentionally returns an error on other platforms.

## Prerequisites

- Node.js 24 or a compatible current Node.js release
- pnpm 11.5.1
- Rust stable toolchain
- Tauri 2 macOS prerequisites
- Xcode Command Line Tools for development
- Full Xcode before Phase 9B installation-package builds

Use pnpm for every frontend dependency and script. Do not mix npm, Yarn, or
Bun lock files into the repository.

## Install Dependencies

```bash
pnpm install --frozen-lockfile
```

## Run the Application

Frontend only:

```bash
pnpm dev
```

Tauri desktop application:

```bash
pnpm tauri dev
```

## Validation Commands

```bash
cd src-tauri
cargo fmt --all -- --check
cargo test --locked
cargo check --locked
cd ..
pnpm build
pnpm tauri dev
pnpm tauri info
```

Phase 9A must not run `pnpm tauri build` or generate installation packages.

## Migration Rules

Migration files are immutable after they are committed.

- Never edit an existing numbered migration.
- Add a new migration for future schema changes.
- Register each migration in the runner with a stable checksum.
- Keep migration versions strictly increasing.
- Run the migration test suite before merging a schema change.

Phase 9A does not add or modify migrations.

## Data Boundaries

### SQLite

SQLite is the source of truth for application records. The runtime database is
created in the Tauri application data directory as:

```text
second-brain-os.sqlite3
```

The database uses foreign keys, WAL mode, migration checksums, and a busy
timeout. Do not commit runtime database, WAL, or SHM files.

### macOS Keychain

DeepSeek API keys are stored by the credential-store adapter under the service:

```text
com.secondbrain.os
```

Application code must depend on the `CredentialStore` port. Never add plaintext
fallback storage, return an API key in an IPC DTO, or write a key to logs.

### Obsidian Vault

SQLite stores only the configured Vault path and export records. Markdown
exports are written under:

```text
SecondBrainOS/Knowledge/
```

Do not scan, index, watch, or copy existing Vault content without a separately
approved phase.

## Application Identity

The following values are frozen for the first MVP package:

- Product: `Second Brain OS`
- Version: `0.1.0`
- Identifier: `com.secondbrain.os`
- Package: `second-brain-os`

Changing the identifier can change the macOS application data directory and
disconnect the application from its existing Keychain service.

## Phase 9B Packaging Prerequisites

Before creating a local installation package:

1. Install the full Xcode application.
2. Run `xcode-select` and accept any required license.
3. Confirm `pnpm tauri info` reports a complete macOS toolchain.
4. Decide whether the build is unsigned local-only or Developer ID signed.
5. Configure signing and notarization only in Phase 9B.
6. Run the complete validation suite from a clean Git worktree.

## Repository Safety

Never commit:

- API keys or `.env` files;
- SQLite, WAL, or SHM files;
- Keychain exports;
- Obsidian Vault contents;
- private keys or signing certificates; or
- local packaging logs.

Test credentials must be obvious non-secret placeholders.

