# 03 — Network and security

**English** · [Français](../03-reseau-et-securite.md) · [Documentation](../README.md)

## Ports

| Default port | Transport | Purpose |
|---|---|---|
| 9001/UDP | Broadcast and multicast | Shared peer discovery |
| 9000/TCP | Noise-encrypted TCP | Messages, reactions, receipts, typing, avatars and groups |
| 9001/TCP | Noise-encrypted TCP | Media streaming; always chat port + 1 |

`ABCOM_INSTANCE=N` moves TCP ports by ten per instance (9010/9011, 9020/9021, …) and selects a separate `abcom-N` data directory. Discovery stays on the shared UDP port. Values are bounded to the valid port range; see [config.rs](../../src/config.rs).

On Linux with UFW, allow the required ports only on the trusted network:

```sh
sudo ufw allow 9000:9001/tcp
sudo ufw allow 9001/udp
```

## Peer discovery

Every three seconds, each instance broadcasts a signed JSON announcement and sends it to multicast group `239.255.42.98`. Multicast loopback lets instances on the same macOS computer find each other. Presence expires after six seconds; expiry checks run every two seconds. Announcements use a 1,024-byte receive buffer, with usernames limited to 64 characters. At most 512 peers are tracked.

Announcements include username, TCP port, public keys, timestamp and an Ed25519 signature. The signing identity is derived from the Noise identity with domain-separated BLAKE2s. Invalid signatures and timestamps more than 60 seconds from local time are rejected. Events are emitted on presence changes rather than on every announcement.

A valid signature proves possession of the announced key, not ownership of a person's name. It does not establish trust at first contact. Two discovery datagrams per instance every three seconds are still LAN traffic while the GUI is idle.

## Identity and trust

A machine generates an X25519 key pair in `identity.key`. Unix permissions are set to `0600` on creation and tightened on load; Windows uses a restricted ACL.

TOFU stores a peer's first public key in SQLite. Later handshakes must present that key. A changed key is rejected with an alert; trusting a replacement is an explicit user action. Verify the fingerprints in Settings → Profile through another trusted channel, especially at first contact.

## Encrypted transport

Chat uses one persistent TCP connection per peer. The Noise pattern is `Noise_XX_25519_ChaChaPoly_BLAKE2s` (`snow`); the encrypted `Hello` exchanges username, protocol version and capabilities. The expected recipient must match the authenticated remote identity. Received packet authors are checked against the session identity.

Setting `ABCOM_PASSPHRASE` enables `XXpsk3`. A domain-separated, iterated derivation runs at startup. Every participant must use the same passphrase. It restricts who can establish a session; it does not identify individual members or provide individual revocation. Distribute it outside Abcom over a trusted channel. Changing it requires updating every participant. Disclosure does not reveal old session keys.

The separate media connection uses Noise too. Files stream in chunks of 60 KB; UI progress updates are throttled to at most once per 100 ms.

## Packets and protocol version

| Packet | Purpose |
|---|---|
| `Chat` | Public, private or group message |
| `Group` | Create, membership change, rename or delete |
| `Typing` | Typing state |
| `Ack` | Private-message delivery confirmation |
| `ReadReceipt` | Private-message read confirmation |
| `Reaction` | Add or remove an emoji reaction |
| `Avatar` | Sender's avatar announcement |

Protocol version **3** uses immutable group IDs in message hashes. Incompatible peers are rejected at `Hello`; upgrade all communicating instances together.

A deterministic FNV-1a hash identifies a message from sender, recipient, epoch timestamp, content, media ID and nonce. Group recipients use `#<id>`, never the display name. This hash is not a cryptographic security boundary.

## Optional Internet access

Local discovery, chat and file transfers do not require Internet access. These features do:

| Feature | Service | What happens |
|---|---|---|
| GIF search | Klipy API | Searches require `ABCOM_KLIPY_API_KEY` |
| Displaying Klipy media | HTTPS Klipy CDN | Each receiving peer fetches the media URL |
| Online fonts, disabled by default | `https://api.fontsource.org/v1/fonts` | Opt-in catalogue GET, refreshed at most once every seven days |
| Choosing an online font | `https://cdn.jsdelivr.net/fontsource/fonts/…` | Up to four font-file GETs |

External services see the requesting IP address. Klipy search sends the search terms; online font requests send no application identity. Clicking an external link also opens the system browser.

No online-font request is sent before consent, including when opening settings. Unticking the permission or cancelling abandons downloads and ignores late results. Catalogue size is capped at 8 MB and each downloaded font at 6 MB. Font IDs are restricted to `[a-z0-9-]`, accepted catalogue licenses to OFL-1.1 and Apache-2.0. Downloaded fonts are decoded and validated before rendering and saved through an adjacent file and rename.

## Threat model and limits

Noise protects session content against passive LAN observers and tampering. TOFU detects changed keys after first contact. A shared passphrase further restricts session admission.

The following remain outside these guarantees:

- **First contact:** an attacker can claim an unfamiliar username with their own key. Compare fingerprints.
- **Local storage:** SQLite history, avatars, attachments and scratch files are not encrypted at rest. Account permissions do not protect against access by the same account, an administrator or an unprotected disk/backup.
- **Discovery metadata:** usernames, network presence and announced public keys are visible on the LAN.
- **Group events:** authorization is checked against the authenticated session and locally known ownership; events have no transferable signature for third-party verification.
- **Message IDs:** FNV-1a is predictable and must not be treated as proof of authorization.

The server caps frames and simultaneous incoming connections and applies read timeouts. Invalid packets are ignored. Remote media fetching is restricted to allowed HTTPS Klipy hosts, preventing arbitrary peer-supplied tracking URLs. Only HTTP/HTTPS Markdown links are clickable; `file:`, `smb:` and application schemes remain plain text.

Use trusted networks, a shared passphrase on shared networks and fingerprint verification for sensitive exchanges. The [audit log](08-history-and-audits.md) records unresolved dependency advisories; encryption is not a claim that every component is free of vulnerabilities.
