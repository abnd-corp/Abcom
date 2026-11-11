# 09 — Known limitations and planned work

**English** · [Français](../09-limites-et-pistes.md) · [Documentation](../README.md)

These are current constraints, not delivery promises. Historical measurements and unresolved dependency advisories are recorded in [history and audits](08-history-and-audits.md).

## Security

| Limitation | Current behavior | Possible work |
|---|---|---|
| No encryption at rest | History, avatars and attachments are readable on local storage | Database/file encryption |
| First-contact trust | A previously unknown username can be announced with an attacker's own key | Verify fingerprints through a trusted channel |
| Visible discovery | UDP advertises username, presence and public keys | A shared passphrase restricts sessions, not discovery visibility |
| Fixed GIF CDN allowlist | Only approved HTTPS Klipy domains are fetched; a provider change can break rendering | Review `ALLOWED_MEDIA_URL_HOSTS` and test 7.3 |
| Session-local group authorization | Peer identity and locally known ownership are checked; events have no transferable signatures | Revisit if third-party forwarding is introduced |
| Dependency advisories | Three D-Bus maintenance advisories have explicit, approved exceptions; GTK and proc-macro-error were removed | Keep issue #59 open until all exceptions are removed |

## Groups and messaging

| Limitation | Current behavior | Possible work |
|---|---|---|
| No offline group replay | Offline group members miss messages; membership state can catch up | A group event log with member offsets |
| No group receipt semantics | Delivery/read receipts and retries apply to private messages | Define group-wide semantics before implementation |
| Group rename has no UI | Backend `Rename` and immutable IDs exist | Wire the management action |
| No online group members | The message remains in local history; the composer is still available | An explicit warning |
| Per-link group encryption | Sender encrypts separately to each member; no common group key | Revisit only if forwarding is introduced |

## Protocol and identity

Chat and media use separate TCP listeners. Multiplexing would require a deliberate protocol change. Identity belongs to a machine, not an account; moving machines changes the key unless the identity is restored. Protocol version mismatches are rejected; there is no N/N−1 negotiation. There is no server, cloud synchronization or Internet relay.

## Platforms and packaging

| Limitation | Current behavior |
|---|---|
| Static musl | A fully static GUI binary is unsupported; dynamic musl is the documented Alpine route |
| macOS packaging | No signed/notarized `.app` bundle; notifications can be attributed to the terminal |
| Windows packaging | PowerShell installer and ZIP workflow; no complete MSI installer |
| Linux tray | Requires a D-Bus session and StatusNotifierItem desktop support; the icon can await a watcher; closing exits if creation fails |
| Native cross-platform validation | Tray, notifications, pickers and accessibility need real-machine testing; compilation alone is insufficient |

## Settings and appearance

The language selector is session-only and startup uses French. Appearance, theme mode and storage preferences do persist. Personal themes and imported fonts need writable data directories; failed saves can leave changes active only for the session.

The following features are retained in the backlog, with no implementation date:

- [VS Code theme import](https://github.com/abnd-corp/Abcom/issues/55).
- [Abcom theme JSON import/export](https://github.com/abnd-corp/Abcom/issues/56).
- [Sharing themes in a conversation](https://github.com/abnd-corp/Abcom/issues/57).
- [Italic variants of the suggested fonts](https://github.com/abnd-corp/Abcom/issues/58).

They are not available yet. The issues describe their scope and expected validation.

## Deferred implementation work

The native file pickers now use an asynchronous path rather than a blocking nested event loop.

Notification action buttons remain deferred, particularly until reliable signed macOS packaging. The timeline uses custom variable-height windowing because uniform row APIs do not match Markdown/media messages. SQLite tables are not `STRICT`; changing that needs an explicit schema migration.

## Measurements and manual checks still needed

Re-measure idle GPU usage, RSS after prolonged GIF browsing and large-transfer throughput using a documented release build. Test pagination while new messages arrive. The headless P2P scenario exercises Noise and an authenticated message on a real socket, but not full GUI-process discovery. Use the [testing checklist](10-testing.md) for desktop integration.

## Tracked audit exceptions

The three D-Bus maintenance exceptions are tracked in [issue #59](https://github.com/abnd-corp/Abcom/issues/59), which must remain open while any exception exists. The full audit still blocks other advisories. CI checks the issue state. Review is due on 30 November 2026; postponing it requires a reason and a new milestone date. The exceptions do not disable accessibility.
