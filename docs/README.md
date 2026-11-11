**English** · [Français](README.fr.md) · [Project](../README.md)

# Abcom documentation

Start with installation and the user guide. The technical references describe the current implementation; historical measurements are dated in the audit log.

| Guide | Contents |
|---|---|
| [Overview](en/01-overview.md) | Purpose, design decisions and terminology |
| [Installation](en/06-installation.md) | Build, platform packages and first run |
| [User guide](en/05-features.md) | Chats, files, notifications and appearance |
| [Troubleshooting and FAQ](en/11-troubleshooting.md) | Discovery, settings, backups and bug reports |
| [Network and security](en/03-network-and-security.md) | Ports, identity, encryption and Internet access |
| [Local storage](en/04-storage.md) | Data files, persistence, retention and migrations |
| [Development](en/07-development.md) | Build checks, contribution and CI |
| [Architecture](en/02-architecture.md) | Modules, threads, rendering and message flow |
| [Testing checklist](en/10-testing.md) | Manual scenarios and automated coverage |
| [Known limitations](en/09-limitations.md) | Current constraints and planned work |

## Maintenance references

- [Performance measurement](en/12-performance.md): reproducible scenarios, tools and interpretation.
- [Maintenance and audit log](en/08-history-and-audits.md): dated validation results and unresolved dependency advisories.

## Maintaining these guides

English and French cover the same features and limitations. Update both pages in a pair when behavior changes; keep commands, limits and test IDs aligned. Each guide links directly to its translation. Existing French filenames remain stable so source-code references keep working.

Run `python3 scripts/check_docs.py` from the repository root to check language pairs, local links, anchors and manual test IDs. Maintenance records keep their original dates and results. The changelog, license text and governance files remain authoritative in their original language; English guides link to them explicitly.
