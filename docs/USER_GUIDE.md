# Second Brain OS User Guide

Second Brain OS is a macOS local-first application for capturing Sources,
creating AI summaries, reviewing Knowledge, and exporting accepted Knowledge
to Obsidian.

## 1. First-Time Setup

1. Open **Settings**.
2. Save a DeepSeek API key and select the default model.
3. Use **Test connection** to verify the saved configuration.
4. Enter an Obsidian Vault directory and save it.
5. Review the active summary prompt. New prompt versions are optional.

The API key is stored in macOS Keychain. The Vault path and provider settings
are stored in local SQLite.

## 2. Configure DeepSeek

In **Settings > AI Provider**:

1. Keep the provider set to DeepSeek.
2. Select the default model.
3. Enter the API key.
4. Save the settings.
5. Test the connection.

When a key is already configured, leaving the key field empty preserves the
existing Keychain entry. The application never displays the saved key.

Testing the connection calls DeepSeek `GET /models`. Summarizing a Source sends
the Source text and active summary prompt to DeepSeek Chat Completions.

## 3. Configure an Obsidian Vault

In **Settings > Obsidian Vault**:

1. Enter the absolute path to an existing directory.
2. Save the path.

The directory may be saved without a `.obsidian` subdirectory, but the
application displays a warning. The application does not scan or read existing
notes.

## 4. Capture Text

Open **Inbox**, enter a note or excerpt, and select **Save**. The new Source is
stored locally with the `unprocessed` status.

## 5. Capture a PDF

Open **Inbox** and select **Select PDF / Import PDF**.

- One PDF can be selected at a time.
- The file must be 20 MiB or smaller.
- Extracted text must be 200,000 characters or fewer.
- Only extracted text and safe metadata are stored.
- The original PDF path and binary file are not persisted.

Scanned or image-only PDFs require OCR and are not supported in this MVP.

## 6. Use the Inbox

Inbox lists unprocessed Sources. You can:

- search Source text;
- open Source details;
- summarize a Source;
- create a Knowledge Draft from a successful summary;
- mark a Source as processed; or
- dismiss a Source.

Processed and dismissed Sources leave the Inbox list.

## 7. Use Source Details

Select **Open Details** on an Inbox card. The detail page shows:

- Source type and lifecycle status;
- capture and processing timestamps;
- complete Source text;
- PDF filename, size, and extraction metadata;
- latest successful or failed AI Run; and
- related Knowledge, when present.

Summary, draft creation, processed, and dismissed actions are also available
from this page.

## 8. Create an AI Summary

Select **Summarize** on an Inbox card or Source Detail page. The application
uses:

- the current active `source_summary` prompt;
- the configured DeepSeek model; and
- the API key from macOS Keychain.

Every successful or failed request is recorded as an AI Run. The API key and
raw provider response are not recorded.

## 9. Create a Knowledge Draft

After a successful summary, select **Create Knowledge Draft**. The latest
successful AI Run creates one proposed Insight. The same AI Run cannot create
multiple Knowledge nodes.

## 10. Review Knowledge

Open **Knowledge**.

- Select **Accept** to move proposed Knowledge to accepted.
- Select **Archive** to archive proposed Knowledge.
- Accepted and archived items are read-only in the current MVP.

## 11. Search Knowledge

Use the Knowledge search field and optional status/type filters. Search covers
Knowledge title and content using local SQLite matching.

## 12. Export to Obsidian

Only accepted Knowledge can be exported.

1. Configure a Vault path in Settings.
2. Open Knowledge.
3. Select **Export** on an accepted node.

Markdown is written under:

```text
SecondBrainOS/Knowledge/
```

The latest export status and path are shown on the Knowledge card. Export does
not scan the Vault or synchronize changes back into the application.

## 13. Use the Dashboard

Dashboard is the default page. It shows:

- unprocessed Inbox count;
- Knowledge totals by status;
- recent Inbox Sources;
- recent Knowledge; and
- whether an Obsidian Vault path is configured.

Use **Refresh** to reload the current local state.

## 14. Data Location and Backups

The SQLite database is normally stored at:

```text
~/Library/Application Support/com.secondbrain.os/second-brain-os.sqlite3
```

The exact application-data directory is resolved by macOS and Tauri.

Quit Second Brain OS before manually copying the database. SQLite uses WAL
mode, so copying a running database without its `-wal` and `-shm` files may
produce an incomplete backup.

Obsidian Markdown remains in the configured Vault and must be backed up using
the user's normal Vault backup process. API keys are separate Keychain items
and are not included in a database backup.

## 15. Privacy Notes

- Local data is not stored in a cloud database.
- API keys are stored in macOS Keychain.
- Source text is sent to DeepSeek only when the user requests a summary.
- PDF binaries are not stored or uploaded by Second Brain OS.
- Obsidian files are written only through explicit export actions.

