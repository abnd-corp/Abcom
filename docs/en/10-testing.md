# 10 — Testing checklist

**English** · [Français](../10-cahier-de-tests.md) · [Documentation](../README.md)

The automated suite covers application logic, real-socket networking and simulated GUI interaction. This checklist covers actual desktops, real LAN conditions, native integrations and filesystem behavior. An item listed here is **not** a claim that it passed.

## Setup

```sh
make run2
make rung
make run-multi N=4
```

Each instance has separate data (`abcom`, `abcom-1`, `abcom-2`, …). Ports are 9000/9001 TCP for instance zero, 9010/9011 for instance one, and so on; discovery is shared UDP 9001. GIFs require a local `ABCOM_KLIPY_API_KEY`.

Use disposable instance directories for destructive tests. Back up before any migration or identity reset. The base locations are in [storage](04-storage.md#data-directory). Never delete production data simply to reset a test.

Record prerequisites, actions, expected/observed result, OS, commit and build configuration. A mismatch or `tracing::error!` is a failure. **P0** blocks a release, **P1** is visible breakage, **P2** is usability. Stop processes through Quit when a clean restart is required; closing normally hides the window.

## 1. Discovery and presence

| ID | Action | Expected | Priority |
|---|---|---|---|
| 1.1 | Start `make run2` | Both peers online within 5 seconds | P0 |
| 1.2 | Quit Bob | Alice marks Bob offline within about 10 seconds; contact remains | P0 |
| 1.3 | Restart Bob | Online again, history intact | P0 |
| 1.4 | Switch Wi-Fi to Ethernet | New address accepted; sending resumes | P1 |
| 1.5 | Move the clock back 2 minutes; wait 10 seconds | Existing peers do not disappear; restore clock afterwards | P1 |

## 2. Messages

| ID | Action | Expected | Priority |
|---|---|---|---|
| 2.1 | Alice sends privately to Bob | Timestamped delivery within 1 second | P0 |
| 2.2 | Send in All | Every online peer receives | P0 |
| 2.3 | Send in a group | Only group members receive | P0 |
| 2.4 | Keep Carol outside the group | No message or error for Carol | P0 |
| 2.5 | Paste over 4,000 characters | Collapsed message with Show more | P2 |
| 2.6 | Paste about 1 MB of text | Text attachment, no UI freeze | P1 |
| 2.7 | Exceed the composer limit | Red counter; sending refused | P2 |
| 2.8 | Reply to a message | Author/text quote on both peers | P1 |
| 2.9 | Disconnect Bob during send, then reconnect | Only one visible copy | P0 |

## 3. Delivery and read receipts

These scenarios use **private conversations**. Group receipts are not implemented.

| ID | Action | Expected | Priority |
|---|---|---|---|
| 3.1 | Send with Bob's window in background | Alice sees delivered | P0 |
| 3.2 | Bob opens the conversation in foreground | Alice sees read | P0 |
| 3.3 | Leave conversation open in background, receive, refocus without switching chats | Read receipt within 1 second | P0 |
| 3.4 | Receive while in tray, then restore | Read receipt after restoration | P1 |
| 3.5 | Repeatedly change foreground focus | No duplicate receipt emissions in logs | P2 |
| 3.6 | Send in a group | No private delivery/read receipt shown | P1 |
| 3.7 | Make recipient database unwritable, then send | No ACK for an uncommitted message; sender retries | P0 |

## 4. Groups

| ID | Action | Expected | Priority |
|---|---|---|---|
| 4.1 | Alice creates a group with Bob | Visible on both | P0 |
| 4.2 | Try spaces, over 50 bytes or duplicate names | Refused | P1 |
| 4.3 | Add Carol | Group arrives; Carol's history starts empty | P0 |
| 4.4 | Remove Carol | Group disappears for Carol | P0 |
| 4.5 | Bob leaves | Bob's local history removed; Alice's retained | P0 |
| 4.6 | Owner leaves | First remaining member succeeds | P1 |
| 4.7 | Owner deletes | Group removed for members that receive the event | P0 |
| 4.8 | Non-owner attempts deletion | Refused | P0 |
| 4.9 | Backend rename with history, reaction and reply present | Name changes; immutable ID, history, reactions and read marker remain intact | P0 |
| 4.10 | Restart after backend rename | Name and history remain consistent | P0 |

**4.9/4.10:** there is no GUI rename action yet. Validate through the group/storage regression tests (`cargo test app::groups` and the storage tests); a full two-window manual rename scenario remains pending UI wiring. Private-message receipts are not a group-test prerequisite.

## 5. Reactions

| ID | Action | Expected | Priority |
|---|---|---|---|
| 5.1 | React | Visible on both peers | P0 |
| 5.2 | Click same reaction again | Removed on both | P0 |
| 5.3 | Alice and Bob use same emoji | Count is 2 | P1 |
| 5.4 | Restart | Reactions persist | P0 |
| 5.5 | Add over 32 distinct emojis | Bounded growth | P2 |

## 6. Media

| ID | Action | Expected | Priority |
|---|---|---|---|
| 6.1 | Send PNG/JPEG | Clickable preview and viewer | P0 |
| 6.2 | Send PDF | Downloadable file card | P0 |
| 6.3 | Send folder | Valid ZIP received | P1 |
| 6.4 | Archive a folder containing unreadable data | Explicit error, not a silently incomplete ZIP | P1 |
| 6.5 | Download received media | Saved to Downloads without freezing | P0 |
| 6.6 | Download twice | Suffix `(1)`; no overwrite | P1 |
| 6.7 | Receive hostile `../../evil.txt` name | Sanitized name confined to Downloads | P0 |
| 6.8 | Send about 10 MB | Automatic acceptance | P0 |
| 6.9 | Send over 50 MB | Recipient acceptance prompt | P0 |
| 6.10 | Refuse offer | Sender sees refusal; no completed file written | P0 |
| 6.11 | Quit sender during transfer | Interrupted state; no completed partial attachment | P0 |
| 6.12 | Select 50 files | Responsive interface and bounded preparation workers | P1 |

## 7. GIFs and emojis

| ID | Action | Expected | Priority |
|---|---|---|---|
| 7.1 | Open GIF / Memes / Stickers | Three populated tabs | P1 |
| 7.2 | Search and scroll | Results and pagination | P1 |
| 7.3 | Send GIF | Animated on recipient | P0 |
| 7.4 | Run without Klipy key | Clear explanation, no crash | P2 |
| 7.5 | Picker and shortcode completion | Correct insertion | P1 |
| 7.6 | Enter, remain near the GIF button edge and leave; light/dark, 100/150/200% scaling, picker open then closed | Frame and lettering stay still; tooltip and click work | P2 |

If 7.3 produces a file card, review the HTTPS Klipy host restriction and the actual provider URL. Do not widen the allowlist without checking the new host.

## 8. Markdown and links

| ID | Action | Expected | Priority |
|---|---|---|---|
| 8.1 | Bold, italic, strike and inline code | Correct rendering | P1 |
| 8.2 | Code blocks, quotes, lists and GFM table | Correct rendering | P1 |
| 8.3 | HTTP/HTTPS link | Clickable | P1 |
| 8.4 | Received `file:` and `smb:` links | Plain text, not clickable | P0 |

## 9. Conversations and history

| ID | Action | Expected | Priority |
|---|---|---|---|
| 9.1 | Receive in closed conversation; open it | Badge increases, then clears | P0 |
| 9.2 | Pin peer and group; restart | Pins remain at top | P2 |
| 9.3 | Mute group | No sound/notification for it | P2 |
| 9.4 | Search a term | Results navigate to messages | P1 |
| 9.5 | Scroll old history | Loads in pages of 100 | P0 |
| 9.6 | Generate over 600 messages, then scroll up | Pagination continues after memory trimming | P0 |
| 9.7 | Clear conversation | Messages, reactions and receipts removed together | P1 |
| 9.8 | Inspect another conversation after 9.7 | Unchanged | P0 |
| 9.9 | Export | Text file saved; success only after writing | P1 |
| 9.10 | Export to unwritable directory | Explicit error; no false success | P1 |

## 10. Security

| ID | Action | Expected | Priority |
|---|---|---|---|
| 10.1 | Observe TCP chat traffic | No readable message text | P0 |
| 10.2 | First connection | Key pinned; fingerprint visible | P0 |
| 10.3 | Reset Bob's disposable identity, restart | Alice rejects changed key and alerts | P0 |
| 10.4 | Verify and explicitly accept new key | Connection resumes | P1 |
| 10.5 | Different room passphrases | No session established | P0 |
| 10.6 | Identical passphrase | Normal connection; release startup under 1 second | P0 |
| 10.7 | Inspect Unix key permissions | `0600` | P0 |
| 10.8 | Change key permissions to `0644`, restart | Tightened to `0600` | P1 |
| 10.9 | Connect incompatible protocol version | Clean rejection | P0 |
| 10.10 | Try duplicate usernames | No silently mixed histories; investigate identity alerts | P1 |

Use disposable identities. Passphrase derivation is much slower in debug; use release for 10.6. Timing is a target to measure, not a previously certified result on every machine.

## 11. Persistence and robustness

| ID | Action | Expected | Priority |
|---|---|---|---|
| 11.1 | Quit and restart | History, groups, aliases, avatars and receipts retained | P0 |
| 11.2 | Make test DB unwritable, send, quit | Save failure logged, not silent | P0 |
| 11.3 | Send while Bob is offline, then reconnect | Private message delivered | P0 |
| 11.4 | Send offline, kill Alice, restart both | Durable private queue still delivers | P0 |
| 11.5 | Compact database | Space reclaimed where possible, no lost history | P2 |
| 11.6 | Leave many attachments for over 15 minutes | No automatic attachment purge | P1 |
| 11.7 | Preview and manually purge by age | Only eligible files removed; inline images protected unless selected | P1 |
| 11.8 | Send existing file; inspect sender's media directory | Original is not duplicated | P2 |
| 11.9 | Set `ABCOM_INSTANCE=999999` | Valid bounded port, no panic | P2 |
| 11.10 | Migrate pre-group-ID data from backup | History, reactions, receipts and read markers preserved | P0 |

For 11.10, use a disposable copy of a real older database containing a group and related messages, reactions and receipts. Back it up, upgrade, then exercise backend rename preservation (4.9). Protocol version 3 requires updating all communicating peers. Filesystem permission tests must actually deny writes; running as an administrator can invalidate the setup, and SQLite WAL behavior must be accounted for.

## 12. Interface and system

| ID | Action | Expected | Priority |
|---|---|---|---|
| 12.1 | Light / dark / system | Readable throughout | P1 |
| 12.2 | French / English | Interface labels switch; language remains session-only | P1 |
| 12.3 | Choose, change, remove avatar | Changes reach peers | P1 |
| 12.4 | Set contact alias | Local and persistent | P2 |
| 12.5 | Close with tray active | App stays running in tray | P0 |
| 12.6 | Open from tray | Window and state restored | P0 |
| 12.7 | Receive while hidden | Native notification | P1 |
| 12.8 | Receive outside active conversation | Sound; silent for active conversation | P2 |
| 12.9 | Unread messages | Tray badge | P2 |
| 12.10 | Enable startup, log out/in | App starts | P2 |
| 12.11 | Resize narrow/wide | No overlap | P1 |
| 12.12 | Choose dark, quit, restart | Dark mode retained | P1 |
| 12.13 | All ten themes, both modes | Readable; no stale colors | P1 |
| 12.14 | Global text at 160% | UI/chat grow; emoji artwork unchanged | P1 |
| 12.15 | Conversation 140%, interface 80% | Independent scales | P1 |
| 12.16 | Edit a built-in color | Personal copy; original intact; live contrast | P1 |
| 12.17 | Equal text/background colors | Warning below 4.5:1 without blocking edits | P1 |
| 12.18 | Each of eight suggested fonts | Correct messages, accents and “Łódź” | P1 |
| 12.19 | Import TTF / OTF / TTC | Family name, applied font, persistence | P1 |
| 12.20 | Import WOFF / WOFF2 | Same behavior | P1 |
| 12.21 | Rename text file to TTF and import | Reasoned rejection, no change or crash | P0 |
| 12.22 | Delete imported font, restart | Theme fallback with notice | P1 |
| 12.23 | Online-font permission off | No request, including opening settings | P0 |
| 12.24 | Enable, search Lexend, select | Download, apply, then work offline | P1 |
| 12.25 | 560 × 360 window, 160% text | Settings fit; scrolling reaches everything | P1 |
| 12.26 | Move between Retina and ordinary screen | Sharp text and stable layout | P2 |
| 12.27 | VoiceOver / NVDA in Appearance | Controls named; theme selection announced | P2 |
| 12.28 | Native font picker on each OS | No freeze; cancelling changes nothing | P0 |
| 12.29 | Emojis in timeline, input and reactions at 80/160% across themes | Stable sizes and alignment | P1 |
| 12.30 | Loaded storage, FR/EN, 100/160%, small window, scroll to actions | Readable columns and reachable controls | P1 |
| 12.31 | General and all credits through palettes at 100/160% | Readable aligned values; single scroll area | P1 |
| 12.32 | License at 100/160%, scroll to end | Scaled text, natural wrapping, no horizontal clipping | P1 |

## 13. Platform-specific checks

| ID | Linux check | Expected |
|---|---|---|
| L1 | Run binary installer | Executable and working menu launcher; no new systemd service |
| L2 | Start from application menu | Correct absolute executable path |
| L3 | Re-login after installation | One instance; startup through XDG, no legacy service |
| L4 | Wayland and X11 | Tray, notifications and focus tested on both |
| L5 | GNOME / KDE / XFCE | Tray works where supported; GNOME may need an extension |
| L6 | Minimal build without GTK/ALSA | Builds; no tray/sound; closing exits |
| L7 | Dynamic musl/Alpine | Builds with documented packages and runs with a GUI/driver |
| L8 | KSNI startup and watcher arrival | On a private D-Bus session create the tray, change the badge and quit; on a real desktop test Open/Quit, hide/restore and a watcher appearing after startup |
| L9 | Linux branding | Installed launcher uses the new logo; colour tray and unread badge remain visible on the target panel |
| L10 | AppImage | Download/verify, launch with and without FUSE, check autostart path, tray, audio and LAN exchanges on the target desktop |
| L11 | Flatpak | Install bundle, launch, attach/export through portals, verify isolated data and no misleading autostart; test tray/notifications/LAN, then uninstall retaining data |

| ID | macOS check | Expected |
|---|---|---|
| M1 | Hide | Dock icon removed, menu-bar icon remains |
| M2 | Restore | Dock icon returns |
| M3 | Two local peers | Discover through multicast loopback |
| M4 | First network access | System firewall authorization handled |
| M5 | Refocus open conversation | Read receipt sent |
| M6 | Inspect `identity.key` | Owner-only permissions |
| M7 | Native icons | New Finder/Dock logo in the bundle; hide/restore retains it; menu-bar template readable on light/dark backgrounds and Retina, with/without unread messages Also run `cargo run --locked --example check_macos_dock_after_frame` to inspect the icon after framework rendering. |
| M8 | Homebrew | Install, reinstall and uninstall through the local tap; verify actual launch, signature, autostart path and data retention; distinguish local builds from quarantined downloads |

| ID | Windows check | Expected |
|---|---|---|
| W1 | Hide | Taskbar entry removed; tray menu works |
| W2 | Restore | Window visible on screen |
| W3 | Inspect key ACL | Restricted to the intended account |
| W4 | Allow in firewall | Discovery works |
| W5 | Send file with spaces in path | Correct transfer |
| W6 | Refocus restored window | Read receipt sent |
| W7 | Native icons | Explorer, shortcuts and taskbar use the new logo at different scales; colour tray has a visible badge |
| W8 | WinGet | Validate the local manifest, install/reinstall, verify Start menu and launch, then uninstall; data retained, shortcut and autostart removed |

## 14. Minimum pre-release pass

Exercise 1.1, 2.1, 2.3, 3.3, 6.1, 6.8, 6.9, 7.3, 10.3, 10.5, 11.1, 11.10, 12.5 and 12.6 on the target systems. Backend rename checks 4.9/4.10 remain mandatory; do not record a nonexistent GUI path as passed. Add appearance and loaded-storage checks when those areas change.

## 15. Automated coverage

The suite covers deterministic hashes, group-ID migration, URL filtering, peer expiry, reaction bounds, durable private queues, pagination, deletion cascades, Noise framing/handshakes, signed discovery, traversal rejection, appearance persistence, font validation, simulated online consent/errors/cancellation, theme contrast and whole-application layout matrices.

Loaded storage labels are checked in French/English at 100/160%, as are reachable purge preferences and license scaling/word preservation. Simulated density checks do not replace a physical display or a real screen reader.

```sh
make check
cargo test --release --all-features --locked
```
