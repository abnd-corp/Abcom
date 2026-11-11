# 05 — User guide

**English** · [Français](../05-fonctionnalites.md) · [Documentation](../README.md)

## Conversations

Use **All** to address currently online peers, choose a peer for private chat, or create a group from the sidebar's `+` button. Groups send messages only to their members. Usernames cannot contain the reserved `#` prefix used internally for group IDs.

The timeline groups messages by author and day, with local timestamps and stable author colors in multi-user conversations. Select text by dragging. Search and older history are available locally; scrolling upwards loads older pages.

## Writing messages

Markdown supports bold, italic, strikethrough, inline/fenced code, quotes, lists, GFM tables and HTTP/HTTPS links. Other URL schemes are not clickable.

The embedded emoji picker supports categories, search and `:shortcode:` completion. Emoji-only messages use larger artwork. Hover over a message to react or reply; click a reaction to toggle it. Replies quote the original author and text and let you navigate back to the message. Typing indicators are sent to the relevant peer or group members.

| Shortcut | Action |
|---|---|
| Enter or Shift+Enter | New line |
| Cmd+Enter (macOS), Ctrl+Enter (other platforms) | Send |
| Enter with shortcode suggestions open | Insert selected emoji |
| Tab with suggestions open | Complete with the first suggestion |
| Up / Down with suggestions open | Select a suggestion |
| Option+Backspace / Ctrl+Backspace | Delete previous word |
| Option+Delete / Ctrl+Delete | Delete next word |
| Cmd+Backspace on macOS | Delete to line start |
| Option+Left/Right / Ctrl+Left/Right | Move by word |
| Cmd+Left/Right on macOS, Home/End | Line start/end |
| Cmd/Ctrl+A | Select all |
| Cmd/Ctrl+C, X, V | Copy, cut, paste |
| Shift+arrows or click | Extend selection |
| Cmd/Ctrl+F | Search |
| Escape | Close the topmost overlay |

## Delivery and read receipts

Private conversations show sent, delivered and read states. Delivery is acknowledged only after the recipient commits the message to SQLite. Reading requires the conversation to be open and the window in the foreground; focus restoration catches up receipts.

Unacknowledged private messages retry with exponential backoff from 1 second to a 32-second cap. Messages for offline private peers remain in a durable queue until acknowledged, including across restarts.

Groups and All do not provide equivalent delivery/read receipts or offline message replay. A member offline during a group message will not receive it later.

## Groups

A group has an immutable ID, a display name, an owner, an ordered member list and a creation time. Names must be nonempty, at most 50 bytes, and contain only Unicode alphanumeric characters, `-` or `_`; duplicate names are rejected using ASCII case-insensitive comparison.

| Action | Owner | Member |
|---|---|---|
| Send and receive | Yes | Yes |
| Add a known peer | Yes | No |
| Remove another member | Yes | No |
| Leave | Yes, with succession | Yes |
| Delete the group | Yes | No |

Use the group's Actions menu for membership, leaving and deletion. Leaving or being removed **deletes that group's local history**. Remaining members keep their copies. If the owner leaves, the first remaining member becomes owner. Deleting a group asks members to remove its history.

Create, add/remove member, rename and delete events are sent only to members. New members receive group state, not old messages. The owner resends current state when a known peer returns. A Create event excluding the local user, or a message for an unknown/left group, is ignored. Authorization checks use the authenticated session and locally known group ownership.

The backend supports renaming without changing message hashes; a GUI trigger is not implemented yet. The [testing checklist](10-testing.md) distinguishes backend scenarios from available UI paths.

## Files, folders and images

Attach files or folders to a private conversation or group. Folders become ZIP archives. Each transfer is limited to 2 GiB and uses 60 KB chunks. Files up to 50 MB are received automatically; larger ones require acceptance, with a 120-second timeout. Progress is shown on both sides.

Received attachments live in the application's `media/` directory. Image previews preserve aspect ratio within 360 × 300 points; click to open the viewer. Download copies an attachment to the system Downloads directory. Existing names get a suffix rather than being overwritten.

The optional Klipy picker has GIF, meme and sticker tabs, a 300 ms search debounce and paginated results. It requires `ABCOM_KLIPY_API_KEY`. GIFs are sent by URL and fetched by each recipient from an allowed Klipy CDN; they need Internet access. See [network and security](03-network-and-security.md#optional-internet-access).

## Profile and aliases

Settings → Profile shows the identity fingerprint and room-passphrase status. Choose, replace or remove an avatar. PNG/JPEG imports are normalized to 256 × 256; SVG import requires the optional `avatar-svg` build feature. Avatars are shared with peers. Contact aliases stay local and persist in SQLite.

## Notifications and resident mode

Visible-window messages can produce an internal toast and sound. A hidden or minimized window uses native system notifications. Settings → General controls message previews and login startup. Conversations can be muted; unread badges appear in the sidebar and tray.

With tray support, Close and Cmd/Ctrl+W hide the window. On macOS its Dock icon disappears too. Choose Open Abcom or Quit from the tray menu; on Windows, left-click opens the window. Without tray support, closing exits. Linux tray availability depends on the desktop environment.

First release launch enables login startup if no setting exists; debug builds do not enable it by default. Integration uses a macOS Launch Agent, Windows Run registry key or Linux XDG autostart entry.

## Settings and appearance

Settings contains Profile, General, Appearance, Storage, Credits and License. Each tab scrolls as one area and adapts to the window. The language selector switches the interface between French and English, but its choice is currently session-only.

Appearance has four sections:

- **Themes:** light, dark or system mode; ten built-ins (Classic, Lounge, Frost, Nocturne, Sand, Slate, Marshmallow, Ink, Forest and Readable) and personal themes. Mode persists across restarts.
- **Text:** global scale from 80–160%, separate interface/conversation scales, conversation line and letter spacing. Combined scale is bounded to 0.7–2. Emoji artwork does not scale with these settings.
- **Colors:** background, surfaces, text and accent, plus advanced roles (21 total). Editing a built-in creates a personal copy. Enter `#RGB` or `#RRGGBB`; live text/background contrast warns below 4.5:1 without blocking the edit. The currently displayed light/dark variant is edited.
- **Fonts:** separate interface and conversation choices, each previewed in its own font: Noto Sans, Inter, Atkinson Hyperlegible, Lexend, OpenDyslexic, Literata, Nunito and JetBrains Mono. Import `.ttf`, `.otf`, `.ttc`, `.woff` or `.woff2` up to 30 MB; decoded data is limited to 60 MB. Invalid, bitmap/color or unsupported fonts are rejected with a reason. Importing identical data selects the existing copy.

The online font catalogue is off by default. It sends no request before consent; see [Internet access](03-network-and-security.md#optional-internet-access). Themes, imported fonts and appearance settings persist in the [data directory](04-storage.md). Missing fonts fall back with a notice. A read-only directory can leave appearance changes active only for the session, with a warning.

## Storage and exports

Settings → Storage displays disk usage and a manual purge preview. Choosing an age does not schedule automatic deletion. Zero days means incomplete/orphan cleanup; inline images are included only when explicitly selected. Original files sent from your computer are protected. See [retention and backup](04-storage.md).

Settings → General → Data exports the current conversation as text or compacts the database. Export is not a full backup. Credits lists bundled assets and licenses; License displays the AGPL text at the selected text size.
