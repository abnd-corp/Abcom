# 04 — Stockage local

[English](en/04-storage.md) · **Français** · [Documentation](README.fr.md)

## Répertoire de données

Toutes les données vivent dans le répertoire de données de la plateforme ([config.rs](../src/config.rs), via `dirs::data_dir()`) :

| Plateforme | Chemin |
|---|---|
| Linux | `~/.local/share/abcom/` |
| macOS | `~/Library/Application Support/abcom/` |
| Windows | `%APPDATA%\abcom\` |

En mode multi-instance (`ABCOM_INSTANCE=N`), chaque instance utilise `abcom-N/` pour ne pas partager ses données.

Contenu :

```
abcom/
├── abcom.db          # base SQLite : messages, groupes, pairs, préférences…
├── abcom.db-wal/-shm # fichiers du mode WAL
├── identity.key      # paire X25519 de la machine (0600)
├── media/            # pièces jointes reçues et ZIP de dossiers envoyés
├── themes/           # thèmes personnels
├── fonts/            # polices importées ou téléchargées
├── scratch/          # pièces jointes texte temporaires
├── logs/             # journaux
└── *.json.bak        # anciens fichiers JSON, conservés après migration
```

## La base SQLite

Un seul fichier, `abcom.db`, ouvert en mode WAL avec `synchronous=NORMAL` (bon compromis durabilité/latence pour un chat local). Le crate est `rusqlite` en feature `bundled` : SQLite est embarqué dans le binaire, aucune dépendance système.

**Les écritures d’historique sont déportées du thread UI.** Les thèmes et polices ont leurs propres chemins de persistance ; toutes les écritures de la base passent par un canal vers un thread de stockage dédié ([app/storage.rs](../src/app/storage.rs), commandes `StorageCmd`). L'ajout d'un message coûte un `INSERT` hors du thread de rendu ; un flush final est effectué à la fermeture de l'application.

**Une écriture qui échoue est remontée.** Le `Flush` ne se contente pas d'accuser le passage : il rend la dernière erreur d'écriture survenue depuis le flush précédent. Un disque plein ou une base en lecture seule est donc journalisé à la fermeture (`historique non sauvegardé`) au lieu de laisser l'utilisateur quitter en croyant ses messages enregistrés.

**Un message n'est acquitté qu'une fois commité.** Après un lot d'insertions réussi, le thread de stockage émet `MessagesPersisted` ; c'est ce signal qui libère les accusés de réception retenus par l'interface. Une écriture qui échoue ne l'émet jamais, donc l'expéditeur ne reçoit rien et réémet — plutôt que de croire son message livré alors qu'il a disparu.

### Schéma

| Table | Contenu |
|---|---|
| `messages` | Hash réseau, expéditeur, destinataire, contenu, dates, média, réponse et nonce |
| `messages_fts` | Index FTS5 de recherche, sans seconde copie du texte |
| `reactions` | Hash du message, emoji et utilisateur |
| `read_marks` | Hash « lu jusqu'à » par conversation, pas un compteur |
| `groups` | JSON du groupe indexé par son ID immuable dans la colonne historique `name` |
| `peers` | Pseudo, alias, avatar et clé publique épinglée |
| `kv` | Préférences persistées |
| `receipts` | Accusés nominatifs de livraison et lecture |
| `outbox` | File durable des messages privés en attente |
| `local_media` | Chemin d'origine des fichiers envoyés |

`to_user` vaut `NULL` pour « Tous », un pseudo pour le privé et `#<id>` pour un salon. Le schéma exécutable est dans [storage.rs](../src/app/storage.rs).


Les évolutions de schéma utilisent `PRAGMA user_version` et des migrations transactionnelles. Une migration échouée laisse la version et le schéma précédents intacts.

| Version | Contenu |
|---|---|
| 1 | Colonne `nonce` sur `messages` |
| 2 | Salons désignés par un identifiant immuable : `to_user` passe de `#<nom>` à `#<id>`, les hashs de messages concernés sont recalculés et `reactions`, `receipts` et `read_marks` sont reportés dessus |

### Chargement et pagination

Au démarrage, seule une fenêtre récente est chargée en mémoire (500 messages, `INITIAL_WINDOW`). Le fil en affiche une centaine ; remonter dans l'historique déclenche des requêtes keyset (`WHERE id < :oldest ORDER BY id DESC LIMIT 100`) qui étendent la fenêtre. L'historique complet reste en base sans limite de taille — l'ancien plafond arbitraire de 500 messages a disparu avec les fichiers JSON.

La fenêtre mémoire est elle-même bornée : au-delà, les plus anciens messages sont drainés et leurs rowids deviennent inconnus. Ce cas est distinct de « tout l'historique est déjà chargé » — les confondre arrêtait définitivement le chargement vers le haut, au moment précis où il restait le plus à charger. Le curseur perdu se redérive alors du hash du plus ancien message encore en mémoire (`HistoryCursor::BeforeHash`).

### Migration depuis les fichiers JSON

Si `abcom.db` n'existe pas au démarrage et que d'anciens fichiers JSON sont présents (`messages.json`, `reactions.json`, `read_counts.json`, `groups.json`, `peer_records.json`, `peer_avatars.json`), leur contenu est importé transactionnellement. Seuls les fichiers importés avec succès sont renommés en `.bak` ; une erreur conserve les sources actives et annule les écritures partielles.

## Préférences (table `kv`)

| Clé ou réglage | Valeurs et comportement |
|---|---|
| `appearance` | Thème, mode clair/sombre/système, texte, polices et accord réseau |
| `notif_preview` | `1` aperçu / `0` discret ; défaut `1` |
| `autostart` | `1` / `0` ; activé au premier lancement release si absent |
| `media_retention_days` | Âge pour la purge manuelle ; `0` nettoie seulement les déchets |
| `media_purge_images` | Inclusion des images du fil dans cette purge |
| Langue | Choix de session uniquement ; le démarrage utilise le français |

Les thèmes personnels vivent dans `themes/`, les polices importées dans `fonts/`. Une police absente ou illisible retombe sur celle du thème avec un avis. Un dossier non inscriptible peut laisser une apparence appliquée seulement pour la session, avec un avertissement.

## Le dossier `media/`

`media/` ne contient que ce que nous **recevons**, plus les archives ZIP des dossiers que nous envoyons — celles-ci n'ayant pas d'original ailleurs. Un fichier que nous envoyons n'y est pas recopié : l'original reste là où l'utilisateur l'a rangé, et la table `local_media` note son chemin pour que notre propre fil garde vignette et ouverture. Dupliquer des gigaoctets d'un document déjà présent sur la machine ne servait à rien.

Chaque fichier porte un identifiant unique préfixé par la date locale de l'envoi (`2026-08-12_143005-000123-rapport.pdf`) : un tri par nom dans l'explorateur les range donc par date, ce qui compte puisque le ménage est manuel.

**Aucune purge automatique.** Rien n'est jamais supprimé sans un clic sur « Purger maintenant » dans Paramètres → Stockage. Un plafond de 2 Go tournait auparavant au démarrage et toutes les 15 minutes ; il a détruit 285 Mo de pièces jointes référencées le 12 août 2026 et a été retiré, avec son déclencheur périodique. La purge manuelle enlève :

- les **transferts inachevés** (`.part`) et les **orphelins** (plus aucun message ne les référence), passé une heure de grâce — ce délai protège une réception en cours et le bref instant où un média est écrit avant que son message n'atteigne la base ;
- les pièces jointes plus vieilles que la durée demandée, en jours, `0` ne purgeant que les déchets ci-dessus ;
- les **images affichées dans le fil** seulement si la case est cochée : les effacer laisse des trous dans les conversations, là où un fichier ne perd qu'un bouton de téléchargement. Seuls les formats réellement peints comptent comme images (`kind = image`) — un HEIC est un `file`.

Un aperçu chiffre ce qui partirait avant tout clic. Deux garde-fous ferment le reste : un ensemble de références vide devant un dossier plein n'autorise aucune suppression d'orphelin, et rien de ce qui appartient à l'utilisateur (nos envois, dont `media_path` rend le chemin d'origine) ne peut être effacé par l'application.

La purge passe par le thread de stockage : la liste des médias encore référencés est une requête SQL, et l'historique en mémoire n'en est qu'une fenêtre — s'en servir supprimerait les fichiers des messages plus anciens.

Un pair ne peut écrire sans confirmation que jusqu'au seuil d'acceptation (50 Mo, cf. [05](05-fonctionnalites.md)), ce qui borne aussi ce que le cache peut absorber entre deux passages.

Les GIF Klipy ne sont jamais stockés : ils voyagent par URL et chaque pair les charge depuis le CDN — dont l'hôte est restreint par une liste blanche, cf. [03](03-reseau-et-securite.md).

## Tables ajoutées le 8 août 2026

| Table | Contenu | Cycle de vie |
|---|---|---|
| `receipts` | Accusés nominatifs livré/lu (`message_hash`, `username`, `kind`) | Insertion idempotente. Effacer une conversation emporte immédiatement les accusés de ses messages — les laisser derrière les rendait orphelins jusqu'au prochain démarrage, où seule une purge de rattrapage les ramassait. Idem pour `reactions` et `read_marks` |
| `outbox` | Messages en attente d'un destinataire hors ligne (`hash`, `to_peer`, `message`) | Vidée pair par pair à leur reconnexion |

**Maintenance.** Paramètres → Général → Données propose la compaction
(`VACUUM` + `ANALYZE`) — la base ne rend jamais seule l'espace des
conversations effacées — et l'export texte de la conversation courante. Les
sauvegardes `*.json.bak` de la migration JSON → SQLite sont supprimées au-delà
de 30 jours.

## Sauvegarde et restauration

Quitter réellement Abcom par le menu du tray, puis copier le **répertoire de données complet**, avec l'identité, la base, les médias, les thèmes et les polices. Une copie de la seule base ouverte peut omettre les transactions présentes dans `abcom.db-wal`. Restaurer avec l'application arrêtée ; une identité différente provoquera des alertes de changement de clé chez les pairs.

L'export texte n'est pas une sauvegarde complète. Les données et sauvegardes ne sont pas chiffrées au repos. Le mode SQLite `synchronous=NORMAL` et le flush à la fermeture ne donnent pas à un arrêt brutal ou à une coupure d'alimentation les mêmes garanties qu'à une sortie normale.
