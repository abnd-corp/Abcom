# 04 — Local storage

**English** · [Français](../04-stockage.md) · [Documentation](../README.md)

## Data directory

| Platform | Default directory |
|---|---|
| Linux | `~/.local/share/abcom/` (or the XDG data directory) |
| macOS | `~/Library/Application Support/abcom/` |
| Windows | `%APPDATA%\abcom\` |

`ABCOM_INSTANCE=N` selects `abcom-N/` instead. Instance zero uses `abcom/`.

```text
abcom/
├── abcom.db           SQLite history, groups, trusted peers and preferences
├── abcom.db-wal       pending WAL transactions, when present
├── abcom.db-shm       shared-memory companion, when present
├── identity.key       private machine identity
├── media/             received attachments and outgoing folder ZIPs
├── themes/            personal themes
├── fonts/             imported/downloaded fonts
├── scratch/           temporary text attachments
├── logs/              application logs
└── *.json.bak         successfully migrated legacy data
```

The contents are local and **not encrypted at rest**. Treat the identity, history and backups as private.

## SQLite and durability

SQLite is bundled through `rusqlite`; no database server is needed. The connection uses WAL and `synchronous=NORMAL`. Runtime history writes are queued to a dedicated writer, with a final flush on exit. This is not a promise that every UI operation avoids disk I/O: theme and font files have their own persistence paths.

The writer reports failed writes and returns the last unreported error on flush. A successful message commit emits `MessagesPersisted`, allowing delivery ACKs; failed writes do not produce delivery confirmation. Graceful exit flushes pending work. An abrupt stop or power loss has different durability guarantees from a clean exit.

## Schema and migrations

| Table | Contents |
|---|---|
| `messages` | Network hash, sender/recipient, text, timestamps, media, reply and nonce |
| `messages_fts` | Contentless FTS5 index for message search |
| `reactions` | Message hash, emoji and username |
| `read_marks` | Read-through message hash per conversation |
| `groups` | Group JSON keyed by its immutable ID in the legacy `name` column |
| `peers` | Username, local alias, avatar and TOFU public key |
| `kv` | Persistent preferences |
| `receipts` | Named delivery/read receipts |
| `outbox` | Durable messages waiting for a private recipient |
| `local_media` | Original paths of files sent from this computer |

`to_user` is null for All, a username for private chat and `#<id>` for a group. Schema migrations use `PRAGMA user_version` and transactions. Version 1 introduced nonces; version 2 migrated group conversation keys and message hashes to immutable group IDs, including dependent references. The schema source is [storage.rs](../../src/app/storage.rs).

At startup, the latest 500 messages are loaded. Older history is fetched in pages of 100 using keyset pagination. The database is not limited to 500 messages. When older in-memory rows have been dropped, a missing cursor is reconstructed from the oldest remaining hash rather than mistaken for the beginning of history.

If no database exists, legacy JSON data is imported transactionally. Only successfully imported source files become `.bak`; failed imports preserve the sources. Migration backups older than 30 days are removed by maintenance.

## Settings persistence

| Preference | Storage and behavior |
|---|---|
| `appearance` | Theme, light/dark/system mode, text, fonts and online-font permission |
| `notif_preview` | `1` for message preview, `0` for discreet notifications; default `1` |
| `autostart` | Login startup; enabled on first release launch if unset |
| `media_retention_days` | Manual purge age; `0` means only incomplete/orphan cleanup |
| `media_purge_images` | Whether manual purge includes inline images |
| Language | Session-only at present; startup uses French |

Personal themes live in `themes/`, imported fonts in `fonts/`. Missing or unreadable fonts fall back to the theme font with a notice. Read-only data directories may allow a temporary appearance change without saving it; the application reports that situation.

## Attachment retention

Received files and outgoing folder ZIPs live in `media/`. Sending an existing file does not duplicate it there: `local_media` records its original location. Dated filenames make sorting received files by age possible.

**Attachments are not automatically purged.** In Settings → Storage, wait for the scan, inspect the breakdown and preview, then choose Purge now. Manual purge can remove:

- incomplete `.part` files and unreferenced files after a one-hour grace period;
- referenced attachments older than the selected age;
- inline images only when Include images is selected.

Zero days means incomplete/orphan cleanup, not deletion of every attachment. Originals belonging to the user are protected. An empty reference set against a nonempty directory does not authorize orphan deletion. Reference lookup uses the database, not only the currently visible messages. The message remains in history when its attachment is purged.

Klipy media is fetched by URL and is not stored as a normal received attachment.

## Backup, export and maintenance

Quit through the tray menu before making a simple file backup. Copy the **whole data directory**, including the identity, database, media, themes and fonts. Do not copy just a live `abcom.db` while ignoring its WAL. Restore with Abcom stopped; restoring a different identity may trigger key-change alerts on peers.

Settings → General → Data exports the current conversation as text and runs compaction (`VACUUM` and `ANALYZE`). A text export is not a complete backup and does not include all attachments or private identity data. Compaction can reclaim space after conversation deletion.
