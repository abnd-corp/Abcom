# 08 — Maintenance and audit log

**English** · [Français](../08-historique-et-audits.md) · [Documentation](../README.md)

Results below are dated and do not certify every version or OS. Current procedures live in the [measurement protocol](12-performance.md) and [testing checklist](10-testing.md).

## Historical milestones

- April 2026: first LAN messenger with plaintext JSON and complete file-history rewrites.
- June 2026: modularization, real-socket tests and CI; 164 tests after the technical audit.
- July 2026: rendering caches, SQLite WAL, persistent Noise connections, resident mode and groups. The macOS measurements then reported about 0.2% CPU, 155 MB RSS and 8 threads after optimization; these are not guaranteed current budgets.
- August 2026: immutable group IDs, protocol 3, post-commit ACKs and durable private queues. The automatic attachment quota was removed after a deletion incident; purge is now manual.

Superseded execution plans and documents have been removed from the current tree. Still-applicable decisions are described in the [overview](01-overview.md#design-decisions) and [architecture](02-architecture.md).

## 12 November 2026 — Final icon checks and GIF button hover

The actual GIF widget test reproduces a width changing from 33 to 31 points on hover and lettering moving by one point. The explicit border was applied after egui 0.37 compensated its margins. The button now uses the same border in this calculation for every state. The test compares its frame and lettering over 24 frames, at the centre and near the edge, in light/dark themes, at 1×, 1.5× and 2×, with the picker open or closed. The old code fails; the fix passes. Tracking: [#64](https://github.com/abnd-corp/Abcom/issues/64).

Repeated checks: 650 tests passed, alongside formatting, strict Clippy, EN/FR documentation, minimal compilation, Rust 1.95 and the release build. Native diagnostics again measure an 18-point tray in all three states and a visible Dock width of 80.7% outside a bundle, 81.2% inside one. All 124 exports, 15 ICO representations, the ICNS structure and Linux launcher are checked; Windows/Linux graphical execution still needs testing on those systems.

Two 1280 × 640 px EN/FR sharing cards use the new logo. The generator checks dimensions, opacity and the size limit of 1 MB. Both were visually inspected; GitHub activation instructions are in the [branding guide](../../assets/branding/README.md#github-social-preview). Their presence in the repository does not change GitHub’s Social preview setting.

## 12 November 2026 — Dock icon after eframe rendering

New screenshots showed that fixing the ICNS files was insufficient. eframe 0.37 reapplied the unpadded window icon during `pre_update`, after the startup AppKit assignment. The earlier check validated artwork and tray geometry, not this lifecycle stage, so it did not prove the displayed Dock size.

Handy declares its ICNS in the bundle and retains that resource when returning to Regular activation policy. Abcom now follows this approach: an explicit empty macOS window icon disables eframe replacement; the bundle restores its original ICNS. A bare executable launched with `cargo run` uses the same embedded ICNS as a fallback. Windows/Linux keep their colour window icon.

The new `check_macos_dock_after_frame` diagnostic runs eframe, waits three frames, then reads and exports the actual `NSApplication` icon. `--reproduce-old` deliberately enables the faulty path and must fail; normal mode must pass, including inside an `.app` bundle. These runtime measurements replace PNG-only inspection for Dock validation.

Results: the old path fails at 97.3% visible width; the fixed path passes at 80.7% from the bare executable and 81.2% from the bundle. Images come from the native icon after three eframe frames. Local checks: 649 tests passed, strict Clippy, minimal compilation, Rust 1.95 and release build passed.

## 12 November 2026 — Native size correction

User screenshots revealed an oversized tray and Dock despite valid bitmaps. tray-icon treats the PNG as a native image capped at 22 points; cropping amplified the problem. The fix sets an 18-point AppKit image backed by a 36 px bitmap in a system square slot, reapplied after each badge change. The native diagnostic measures 18 × 18 pt / 36 × 36 px and a 22 × 22 pt button on this Mac, in normal/unread/read states.

The ICNS body is reduced to 824/1024 for macOS only. Apple Settings, Terminal and the corrected export have a visible width of 206/256 at alpha > 128. Windows gets 15 ICO sizes and a tray sized using its DPI metrics; API types are checked against bindings 0.61.2, without Windows execution. Linux keeps sizing under its SNI panel’s control. Visual testing on other desktops remains necessary.

## 12 November 2026 — Native icons

The colour logo is embedded in the window and desktop launchers. macOS uses a high-resolution Dock icon and a menu-bar template retained when changing the unread badge. Windows gets a multi-resolution ICO embedded through the build-only `winresource` dependency. Linux installers put the PNG in hicolor. The macOS packaging script and release workflow produce an `.app` bundle with ICNS and metadata.

Local macOS validation: 645 tests passed, alongside strict Clippy, compilation without default features, Rust 1.95, release and the complete audit. The native AppKit probe decoded the ICNS and created the template tray with unread/read transitions; no failure or update error occurred. `iconutil` reads ten standard/Retina representations; all six ICO entries and bundle metadata are checked. Actual light/dark and Retina rendering, plus Windows/Linux integration, remain in L9/M7/W7. This local validation does not replace CI on the other platforms.

## 11 November 2026 — KSNI tray and tracked exceptions

The AppIndicator/GTK backend is replaced by KSNI in tray-icon 0.26.1, with default features disabled. GTK, libappindicator and proc-macro-error have disappeared from the lockfile. The GTK thread and 200 ms polling are removed; menus and badge updates use the KSNI worker. The workaround for inherited Snap GTK variables is no longer needed and was removed.

Three explicit, approved exceptions cover only RUSTSEC-2026-0370, 0371 and 0372. Their rationale and [issue #59](https://github.com/abnd-corp/Abcom/issues/59) are recorded in `deny.toml`. The issue stays open until all exceptions are removed. CI checks that invariant and now runs the complete audit; closing the issue while an exception remains fails the check. Review is due 30 November 2026, with a reason and rescheduled milestone if needed. No vulnerability advisory is exempted.

An explicit Linux test creates the tray and changes its badge on a private D-Bus session without GTK; CI runs it with `dbus-run-session`. This does not validate an actual desktop panel, which remains in the manual checklist. GTK/AppIndicator system packages have been removed from CI and the testing image.

Local validation on macOS: 642 all-feature tests and 640 tests without default features passed, with no failures. Formatting, strict Clippy, the release build, EN/FR documentation checks, compilation with Rust 1.95.0 and the complete `cargo deny --locked check` audit pass. The Linux D-Bus test is prepared but was not run locally; an actual Linux desktop session remains to be checked. The tracking guard was checked with the issue open and a simulated closed state, which fails as intended.

## 11 November 2026 — Dependency update

All direct dependencies were raised to their latest compatible stable versions and the lockfile was resolved again. Main upgrades: eframe/egui 0.37, rusqlite 0.42, ZIP 9.5.1, rfd 0.18, tray-icon 0.26.1, resvg 0.50, rodio 0.23 and skrifa 0.50.

At this stage the selected AppIndicator backend used GTK 0.18.2; GTK 0.19 conflicted at native-library linkage. The later KSNI migration described above removes that chain. egui's monochrome emoji fonts are explicitly enabled to retain previously supplied glyphs. Identifier, accessibility-role and scroll-compensation APIs were adapted, along with ZIP filename reading. The retention duration control now has an accessible label.

The refreshed RustSec audit reports no vulnerability or I/O-safety advisory for the resolved graph. **It still fails on four unmaintained dependencies**, without any exception added to `deny.toml`:

| Dependency | Remaining path | Advisory |
|---|---|---|
| `proc-macro-error 1.0.4` | GTK 0.18 and Linux tray | RUSTSEC-2024-0370 |
| `zbus_names 4.3.5` | AccessKit/AT-SPI and Linux accessibility | RUSTSEC-2026-0371 |
| `zvariant 5.15.1` | AccessKit/AT-SPI and Linux accessibility | RUSTSEC-2026-0370 |
| `zvariant_derive 5.15.1` | Same chain | RUSTSEC-2026-0372 |

The latest upstream dependencies still retain these components. Removing the four advisories needs upstream work or a dedicated adaptation; disabling tray or accessibility to hide the audit would change the product. The earlier audit figures below describe the state before this update.

### Update validation

| Check | Result |
|---|---|
| All-target/all-feature Clippy, `-D warnings` | Passed |
| All-feature tests | 639 passed, no failures |
| Tests without default features | 639 passed, no failures |
| Release build | Passed |
| Formatting and EN/FR documentation checks | Passed |
| All-target/all-feature compilation with Rust 1.95.0 | Passed; MSRV retained |
| Licenses, sources and bans | Passed |
| Complete RustSec audit | Fails only on the four maintenance advisories above |

## 11 November 2026 — Settings review after Appearance

Local work resumed from `feature/apparence` at `b05a022`, without a push. Comparison with `dev` showed grids without column width limits and a license display fixed at 12 points. Global text wrapping introduced with Appearance reduced early grid columns to egui's 40-point minimum. Storage had only been exercised before its disk-usage reply; previous geometry checks ignored noninteractive text.

Corrections bound columns in General, Storage and Credits, wrap storage actions, use one scroll area per tab and keep separate tab positions. License text follows the general text size, reflows paragraphs and preserves every word; `LICENSE` is unchanged.

Six regression tests cover French/English labels and controls, 560 × 360 and 860 × 600 windows, 100/160% text, readable columns, license scaling and reachable purge preferences with saving. The existing appearance matrix now uses loaded storage data with gigabytes and thousands of files. Off-screen GPU captures were inspected; this is not native-window or screen-reader validation.

### Local checks on macOS, Rust 1.99.1

| Check | Result |
|---|---|
| `cargo fmt --all --check`, `git diff --check` | Passed |
| `cargo clippy --all-targets --all-features --locked -- -D warnings` | Passed |
| `cargo test --all-features --locked` | 639 passed: 631 library, 2 binary, 1 P2P, 5 interaction tests |
| `cargo check --all-targets --no-default-features --locked` | Passed |
| `cargo test --no-default-features --locked` | 639 passed |
| `cargo build --release --locked` | Passed |
| `cargo deny --locked --offline check licenses sources bans` | Passed, with nonblocking duplicate-version warnings |
| `cargo deny --locked check advisories` | Failed; see below |

The first sandboxed network run failed to open local sockets (`Operation not permitted`); the complete suite then passed with the required access. GPU capture also needed access beyond the sandbox. Native Windows/Linux checks and Rust 1.95 MSRV validation were not run locally. At that validation point, there was no real-window, native-picker or screen-reader run; use tests 12.25–12.32 for desktop verification. Later launching the application for a user test is not a completed manual test report.

### Dependency audit remains failing

The refreshed RustSec database at revision `fd6f39814c4d3c359fdbe223d0280dec4933f78f` reported **16 errors**: 11 vulnerability advisories, one I/O-safety advisory and four unmaintained packages. Yanked `clipboard-win 5.4.1` added a warning. The issue therefore extends beyond that one package.

These dependency versions predated the settings correction; that correction did not change the manifest or lockfile. This list is not an exploitability assessment for Abcom. Dedicated upgrades and renewed checks remain necessary.

| Locked dependency | cargo-deny category | Advisory |
|---|---|---|
| `libsqlite3-sys 0.38.2` | vulnerability | [RUSTSEC-2026-0459](https://rustsec.org/advisories/RUSTSEC-2026-0459) |
| `proc-macro-error 1.0.4` | unmaintained | [RUSTSEC-2024-0370](https://rustsec.org/advisories/RUSTSEC-2024-0370) |
| `rustls 0.23.43` | vulnerability | [RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285) |
| `rustls 0.23.43` | vulnerability | [RUSTSEC-2026-0346](https://rustsec.org/advisories/RUSTSEC-2026-0346) |
| `rustls 0.23.43` | vulnerability | [RUSTSEC-2026-0348](https://rustsec.org/advisories/RUSTSEC-2026-0348) |
| `rustls 0.23.43` | vulnerability | [RUSTSEC-2026-0349](https://rustsec.org/advisories/RUSTSEC-2026-0349) |
| `rustls-webpki 0.103.13` | vulnerability | [RUSTSEC-2026-0382](https://rustsec.org/advisories/RUSTSEC-2026-0382) |
| `tokio 1.53.1` | unsound | [RUSTSEC-2026-0386](https://rustsec.org/advisories/RUSTSEC-2026-0386) |
| `webpki-roots 1.0.9` | vulnerability | [RUSTSEC-2026-0381](https://rustsec.org/advisories/RUSTSEC-2026-0381) |
| `zbus_names 4.3.4` | unmaintained | [RUSTSEC-2026-0371](https://rustsec.org/advisories/RUSTSEC-2026-0371) |
| `zip 8.6.0` | vulnerability | [RUSTSEC-2026-0367](https://rustsec.org/advisories/RUSTSEC-2026-0367) |
| `zip 8.6.0` | vulnerability | [RUSTSEC-2026-0368](https://rustsec.org/advisories/RUSTSEC-2026-0368) |
| `zip 8.6.0` | vulnerability | [RUSTSEC-2026-0369](https://rustsec.org/advisories/RUSTSEC-2026-0369) |
| `zip 8.6.0` | vulnerability | [RUSTSEC-2026-0429](https://rustsec.org/advisories/RUSTSEC-2026-0429) |
| `zvariant 5.13.1` | unmaintained | [RUSTSEC-2026-0370](https://rustsec.org/advisories/RUSTSEC-2026-0370) |
| `zvariant_derive 5.13.1` | unmaintained | [RUSTSEC-2026-0372](https://rustsec.org/advisories/RUSTSEC-2026-0372) |
