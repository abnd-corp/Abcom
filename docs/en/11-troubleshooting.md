# 11 — Troubleshooting and FAQ

**English** · [Français](../11-depannage.md) · [Documentation](../README.md)

## Computers cannot discover each other

1. Check that both use the same LAN. Guest Wi-Fi, client isolation, a VPN or separate subnets can prevent discovery.
2. Allow Abcom through the system firewall on the trusted network. Defaults are UDP 9001 for discovery and TCP 9000/9001 for exchanges.
3. Choose different usernames and check protocol compatibility.
4. If `ABCOM_PASSPHRASE` is set, use the same value on both sides without disclosing it in a bug report.
5. Try `make run2` on one computer to distinguish an application issue from network restrictions.

The [network guide](03-network-and-security.md) explains ports and announcements. Discovery is periodic; allow a few seconds after startup.

## The application disappeared when I closed the window

With tray support, Close hides the window. Use the macOS menu-bar icon or Windows/Linux tray icon to open Abcom or actually quit. Without an available tray, Close exits.

## Settings do not survive restart

- Quit through the tray menu, then start the same instance.
- Check `ABCOM_INSTANCE`: each number uses a different data directory.
- Check write permissions and save warnings. Appearance can remain active for a session even when saving fails.
- Themes, text sizes, fonts, light/dark mode and purge settings persist. **Language is session-only** and currently returns to French.
- Deleted or unreadable fonts fall back to the theme font with a notice.

See [storage and preferences](04-storage.md).

## A private message is waiting

An offline peer can receive the message when it returns. The private queue is persistent and unacknowledged messages retry. Check key-change alerts and peer availability too. Groups and All have no message catch-up for absent recipients.

## A contact's key changed

Reinstallation, a different instance directory or impersonation can cause this. Compare the new fingerprint with your contact through a trusted channel before accepting it. Blindly deleting trust data would bypass this protection.

## GIFs or online fonts do not work

GIFs require Internet access and a Klipy key. Each recipient fetches the GIF from an allowed CDN; a disallowed URL becomes a file card. The font catalogue requires consent in Settings → Appearance → Fonts. Embedded and locally imported fonts work offline. See [optional network access](03-network-and-security.md#optional-internet-access).

## Storage keeps growing

Attachments are not purged automatically. In Settings → Storage, wait for the scan, choose an age and read the preview before purging. Zero days means incomplete/orphan cleanup, not deleting everything. Inline images are kept unless their checkbox is selected. The history message remains after its attachment is deleted.

After deleting conversations, compact the database in Settings → General → Data to reclaim space. See [retention](04-storage.md#attachment-retention).

## Back up or move to another computer

Actually quit Abcom and copy the **whole data directory**, not only `abcom.db`. It also holds the identity key, media, themes and fonts. Keep the backup private. A live backup must account for SQLite WAL; stopping the application first is the simple file-copy procedure.

Restore while Abcom is stopped. A conversation text export is not a complete backup. See [platform-specific paths](04-storage.md#data-directory).

## Why does building fail on Linux?

Install the development packages listed in [installation](06-installation.md#linux-build-dependencies). `--no-default-features` removes tray and sound, but the application is still graphical. Windowing and a compatible driver remain necessary. Fully static musl binaries are unsupported.

## Does Abcom work without Internet?

Yes for local discovery, conversations, files, embedded fonts and local imports. GIFs, the online font catalogue and external links have separate Internet access. Messaging needs no central server.

## Are the data encrypted?

Chat and transfer sessions use Noise. Presence announcements are visible on the LAN. History, media and backups are not encrypted at rest. Verify first-contact identity for sensitive exchanges. Read the [threat model](03-network-and-security.md#threat-model-and-limits).

## Report a problem

In an [issue](https://github.com/abnd-corp/Abcom/issues), include version/commit, OS and architecture, build configuration, minimal reproduction steps, expected behavior and actual behavior. For layout problems, add window size, pixel density, theme and text size.

Logs are in the data directory's `logs/`; a panic may produce `last-panic.txt`. Review and redact excerpts before sharing. Never attach `.env`, `identity.key`, the conversation database or a complete backup. Hide private names and addresses in screenshots.
