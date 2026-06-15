# Second Brain OS

Second Brain OS is a local-first AI second brain desktop application built
with React, TypeScript, Tauri 2, Rust, and SQLite.

The MVP turns captured text and text-based PDFs into searchable Sources,
AI-generated summaries, reviewed Knowledge, and Markdown files exported to an
Obsidian Vault.

## MVP Features

- Text Capture
- PDF Capture with local text extraction
- Inbox lifecycle and Source search
- Source Detail view
- DeepSeek AI Summary with persisted AI Run history
- Knowledge Draft creation and review
- Knowledge filtering and search
- Obsidian Vault configuration and Markdown export
- Dashboard overview

## Platform Support

The current MVP supports macOS only.

- API keys are stored in macOS Keychain.
- Phase 9B packaging requires the full Xcode application.
- Windows and Linux credential-store support has not been implemented.

## Local-First Data Boundaries

Application data is stored locally in SQLite under the Tauri application data
directory for `com.secondbrain.os`. On macOS, the database is normally located
under:

```text
~/Library/Application Support/com.secondbrain.os/second-brain-os.sqlite3
```

The exact directory is resolved by macOS and Tauri at runtime.

- Source text, PDF-extracted text, AI Runs, Knowledge, prompts, settings, and
  export records are stored in SQLite.
- PDF binaries are not copied into SQLite or the application data directory.
- DeepSeek API keys are stored in macOS Keychain, not SQLite.
- Accepted Knowledge is written as Markdown only when the user explicitly
  exports it to the configured Obsidian Vault.
- The application does not scan or synchronize the Vault.

## DeepSeek Network Calls

Most application workflows are local. Network access occurs when the user:

- tests the DeepSeek connection with `GET /models`; or
- requests a Source summary with `POST /chat/completions`.

The selected Source text, active prompt, configured model, and API key are sent
to DeepSeek for a summary request. API keys are never returned to the frontend
or stored in SQLite.

## Prerequisites

- macOS
- Node.js 24 or a compatible current Node.js release
- pnpm 11.5.1
- Rust stable toolchain
- Tauri 2 macOS prerequisites
- Full Xcode before building local installation packages in Phase 9B

## Install Dependencies

```bash
pnpm install --frozen-lockfile
```

## Run Locally

Run the frontend only:

```bash
pnpm dev
```

Run the desktop application:

```bash
pnpm tauri dev
```

## Validation

```bash
cd src-tauri
cargo fmt --all -- --check
cargo test --locked
cargo check --locked
cd ..
pnpm build
pnpm tauri info
```

Do not use npm or modify an existing migration file.

## Documentation

- [User Guide](docs/USER_GUIDE.md)
- [Development Guide](docs/DEVELOPMENT.md)

## Current Limitations

- PDF OCR and scanned-image extraction are not supported.
- URL, image, and audio Sources are not supported.
- PDF preview and page rendering are not supported.
- RAG and vector search are not supported.
- Knowledge relationships are not supported.
- Obsidian bidirectional synchronization and Vault scanning are not supported.
- Batch export is not supported.
