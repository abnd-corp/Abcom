# 09 — Limites connues et pistes

[English](en/09-limitations.md) · **Français** · [Documentation](README.fr.md)

Inventaire honnête de ce que l'application ne fait pas (ou fait imparfaitement), avec les pistes envisagées. Les pistes ne sont pas des engagements de livraison ; les anomalies constatées et les avis de dépendances sont aussi suivis ici.

## Sécurité

| Limite | Détail | Piste |
|---|---|---|
| Pas de chiffrement au repos | `abcom.db`, `media/` et les avatars sont en clair sur le disque local ; seul le transit est chiffré | SQLCipher, ou chiffrement fichier par clé dérivée de la session utilisateur |
| Découverte visible | L'annonce UDP (pseudo + empreinte de clé) est lisible par tout le LAN — c'est la fonction même de la découverte | La passphrase de salon empêche déjà un inconnu d'établir une session |
| Liste blanche du CDN de GIF codée en dur | Le chargement d'un média distant est restreint à `https://*.klipy.com` — le CDN observé est `static.klipy.com`, donc couvert. Un changement de domaine côté Klipy ferait retomber les GIF sur une carte fichier | Le domaine se corrige dans `ALLOWED_MEDIA_URL_HOSTS` ; test 7.3 du [cahier](10-cahier-de-tests.md) |
| Autorisation locale, sans signature transférable | L'auteur Noise est vérifié et les actions sont autorisées selon le propriétaire connu ; un événement n'est toutefois pas vérifiable hors de sa session d'origine | Signer les événements uniquement si un relais tiers apparaît |

## Groupes et messagerie

| Limite | Détail | Piste |
|---|---|---|
| Pas de réémission hors ligne | Un membre absent au moment d'un envoi ne recevra jamais ce message (les mutations de membres, elles, sont rattrapées par le propriétaire) | Journal par salon avec offset par membre |
| Renommage de salon sans UI | Le protocole existe (`Rename`) et le renommage ne coûte plus rien depuis le passage aux identifiants immuables, mais le bouton n'est pas câblé | Ajouter l'entrée dans le modal de gestion |
| Salon sans membre en ligne | Le message part dans le vide (historique local seulement) ; la barre de saisie reste active, contrairement au privé qui signale « hors ligne » | Avertissement visuel |
| Chiffrement de groupe par lien | Pas de clé de salon partagée : l'émetteur chiffre vers chaque membre sur sa session pair-à-pair — suffisant tant que l'émetteur relaie lui-même | Clé de groupe si le relais par un tiers apparaît |

## Protocole et transport

| Limite | Détail | Piste |
|---|---|---|
| Deux ports TCP | Les médias ont leur propre listener (`chat + 1`) avec son en-tête dédié | Multiplexer les médias sur la connexion chat unique (un octet de type de trame suffit) — différé volontairement : refonte de protocole juste après la précédente, gain purement architectural, à faire une fois le transport validé par l'usage |
| Identité liée à la machine | Une clé par machine, pas par utilisateur ; changer de machine = nouvelle identité (bandeau « clé changée » chez les pairs) | Export/import de l'identité |
| Compatibilité stricte | Le `Hello` porte une version et rejette explicitement une version incompatible ; aucune négociation N/N-1 n'est encore prévue | Ajouter des capacités négociées lorsqu'une première évolution compatible devient nécessaire |

## Plateformes et packaging

| Limite | Détail | Piste |
|---|---|---|
| Pas de binaire musl statique pris en charge | La pile graphique et les pilotes utilisent des bibliothèques dynamiques ; Alpine suit la voie musl dynamique | Valider les bibliothèques et pilotes de la cible |
| Pas de bundle macOS | Binaire nu : les notifications sont attribuées au terminal | Bundle `.app` (`com.abend.abcom`) via cargo-bundle/cargo-packager, à la première release |
| Pas d'installateur Windows complet | Le script PowerShell installe et crée les raccourcis, sans MSI | Package MSI ou ZIP signé |
| Tray Linux dépendant du shell | Nécessite D-Bus et StatusNotifierItem (GNOME peut demander une extension) ; une icône peut attendre le watcher ; si la création échoue, fermer quitte | — |
| Tray Windows/Linux non testés en réel | Implémentés mais l'environnement de dev est macOS | Passe manuelle §13 du [cahier de tests](10-cahier-de-tests.md) sur machines cibles |

## Dette identifiée, non résorbée

Décisions de maintenance conservées après réexamen de l'implémentation actuelle.

| Constat | Détail | Pourquoi pas appliqué |
|---|---|---|
| Pas d'actions dans les notifications | `notify-rust` sait poser des boutons (« Répondre ») | Sur macOS, `show()` bloque en attendant la réponse, et les boutons n'apparaissent de façon fiable que depuis un bundle `.app` signé — que nous n'avons pas encore. À reprendre avec la signature macOS |
| Fenêtrage du fil fait à la main | `ScrollArea::show_rows` suppose des lignes de hauteur uniforme, ce que nos messages n'ont pas (markdown, médias, citations, réactions) ; `show_viewport` exige des offsets connus d'avance | Reviendrait à reconstruire notre fenêtrage sous une autre forme, sans gain |
| Schéma SQLite non `STRICT` | Les colonnes n'ont pas de typage strict | Différé à une prochaine migration de schéma, pour ne pas cumuler deux réécritures de tables |

Les sélecteurs natifs ont depuis été corrigés : ils suivent une voie asynchrone, sans boucle native bloquante imbriquée. L'ancien constat sur un sélecteur seulement reporté d'une frame ne décrit plus l'implémentation.

## Réglages et apparence

- La langue est un choix de session et revient au français au démarrage ; les réglages d'apparence et de stockage sont, eux, persistés.
- Les thèmes personnels et polices importées demandent un dossier inscriptible ; un échec peut laisser le réglage appliqué seulement pour la session.
Les fonctionnalités suivantes sont retenues dans le backlog, sans date de réalisation :

- [Import de thèmes VS Code](https://github.com/abnd-corp/Abcom/issues/55).
- [Import/export de thèmes Abcom en JSON](https://github.com/abnd-corp/Abcom/issues/56).
- [Partage de thèmes dans une conversation](https://github.com/abnd-corp/Abcom/issues/57).
- [Variantes italiques des polices proposées](https://github.com/abnd-corp/Abcom/issues/58).

Elles ne sont pas encore disponibles ; les issues précisent le périmètre et la validation attendue.

## Dépendances

GTK et `proc-macro-error` ont été retirés au profit du tray KSNI. Trois avis de maintenance D-Bus font l'objet d'exceptions explicites approuvées, suivies dans l'[issue #59](https://github.com/abnd-corp/Abcom/issues/59). Elle reste ouverte tant qu'une exception subsiste ; la CI vérifie cet état. L'audit complet continue de bloquer les autres avis. Revue au 30 novembre 2026 ; un report demande une justification et une nouvelle date de jalon. L'accessibilité reste activée.

## Rendu et mesures restantes

- Mesures à refaire selon le [protocole de mesure](12-performances.md) : GPU au repos, RSS après navigation GIF intensive picker fermé, débit d'un transfert > 1 Go.
- QA manuelle du fenêtrage : compensation d'offset au chargement de 100 messages pendant qu'un message arrive.

## Outillage

- Le scénario P2P headless couvre handshake, identité et message sur une vraie socket. La découverte UDP et le cycle complet de deux processus GUI restent validés manuellement.
