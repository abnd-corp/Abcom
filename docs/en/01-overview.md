# 01 — Project overview

**English** · [Français](../01-presentation.md) · [Documentation](../README.md)

## What Abcom is

Abcom is a native messenger for a local network: an office, home, event or phone hotspot. Each computer runs the same program. There is no central server, account service or registration. Instances discover each other automatically when the network permits it.

Three principles guide the project:

1. **Local operation.** Messages and file transfers work without Internet access. Klipy GIFs and the opt-in online font catalogue are separate Internet features.
2. **Resident application.** The application can remain reachable in the system tray or macOS menu bar. Idle performance measurements are historical observations, not hardware requirements or current guarantees; see [history and audits](08-history-and-audits.md).
3. **Encrypted connections.** Chat and file transfers use authenticated Noise sessions. Discovery announcements remain visible on the LAN, and local history is not encrypted at rest. See [network and security](03-network-and-security.md).

## How it works

Every three seconds, an instance sends signed UDP presence announcements by broadcast and multicast. Peers that have not announced themselves for six seconds become offline.

Messages use direct, persistent TCP connections. A Noise XX handshake establishes session keys using X25519; ChaCha20-Poly1305 protects the frames. The first public key associated with a peer is remembered (TOFU). A changed key is rejected until the user explicitly trusts its replacement.

Every participant keeps a local SQLite history. There is no central copy or central backup.

## Features

- Public “All” conversation, private conversations and member-only groups.
- Markdown, emojis, reactions, quoted replies and typing indicators.
- Private-message delivery and read receipts, retries and a durable offline queue.
- Files and ZIP-packaged folders up to 2 GiB, with confirmation above 50 MB.
- Image previews, a viewer and optional Klipy GIFs, memes and stickers.
- Native notifications, unread badges and configurable login startup.
- Ten light/dark themes, custom themes, text sizing and local font imports.

With tray support, closing the window hides it; use the tray's Quit action to exit. Release builds enable login startup on the first launch unless a preference already exists. [Installation](06-installation.md) explains minimal builds without tray or sound.

## Design decisions

**Rust, Tokio and egui.** One native application combines asynchronous networking with an immediate-mode GUI. Electron, Python/Node and Qt/GTK application frameworks were considered and not selected. The actual binary size depends on features and build versions.

**LAN peer-to-peer.** UDP discovery and direct TCP connections avoid a central discovery or message service. There is no Internet relay or automatic cross-subnet discovery.

**SQLite.** A dedicated writer and WAL replace the early implementation's complete JSON rewrites. Messages, groups, reactions, identities of trusted peers, avatars and preferences live in `abcom.db`.

**Noise with persistent connections.** The former plaintext, per-packet transport was replaced before a public compatibility commitment. Noise provides peer keys and forward secrecy without a public-key infrastructure.

**One resident process.** Networking, storage and GUI communicate through channels. A separate daemon was considered; it would add IPC and lifecycle complexity. Hiding the GUI releases rendering resources while the same process continues handling events.

**Stable group IDs.** A group's name is a mutable label. Its immutable ID enters conversation keys and message hashes, so renaming does not detach history, reactions or read markers. The backend supports renaming; the settings UI for it is still missing.

## Terminology

| Term | Meaning |
|---|---|
| Peer | Another instance, identified by a username and public key |
| Discovery | Periodic signed UDP presence announcement |
| All | Public conversation sent to online peers |
| Group | A member-only conversation, internally `#<id>` |
| Owner | Group creator or successor; manages membership and deletion |
| ACK / read receipt | Private-message delivery / read confirmation |
| Noise XX | Handshake with static peer identities and ephemeral session keys |
| TOFU | Trust on first use: remember the first key and require it later |
| Fingerprint | BLAKE2s digest of a public key, displayed in Settings → Profile |
| Room passphrase | Optional shared `ABCOM_PASSPHRASE` required for a connection |
| Tray | Resident icon in the system tray or macOS menu bar |
| History window | Messages currently loaded in memory; older ones remain in SQLite |
