# Cahier de tests

[English](en/10-testing.md) · **Français** · [Documentation](README.fr.md)

Tests manuels d'abcom. La suite automatique (`make check`, plus de 600 tests) couvre la
logique, les sockets réelles et les interactions simulées ; ce cahier couvre le réseau réel entre postes,
l'interface, le système de fichiers et les trois OS.

## Comment lancer

```bash
make run2        # 2 fenêtres : alice, bob
make rung        # 3 fenêtres : alice, bob, carol
make run-multi N=4
```

Chaque instance a son `ABCOM_INSTANCE` (ports 9000, 9010, 9020…) et son propre
répertoire de données. Les GIF exigent `ABCOM_KLIPY_API_KEY` dans `.env`.

Utiliser des dossiers d'instances **jetables** pour les tests destructifs et sauvegarder avant migration ou changement d'identité. Ne jamais effacer les données d'usage pour réinitialiser un test. Les chemins sont dans le [guide de stockage](04-stockage.md#répertoire-de-données).

Une fermeture par la croix masque normalement la fenêtre : utiliser « Quitter » pour tester un arrêt réel. Consigner OS, commit, options de compilation et résultat observé ; une ligne dans ce cahier ne signifie pas que le test est passé.

## Conventions

Chaque test : **préconditions → actions → attendu**. Un test échoue si
l'attendu diffère, *ou* si un `tracing::error!` apparaît dans la console.

Priorités : **P0** bloque une livraison, **P1** défaut visible, **P2** confort.

---

# 1. Découverte et présence

| # | Test | Actions | Attendu | P |
|---|---|---|---|---|
| 1.1 | Découverte mutuelle | `make run2` | Chacun voit l'autre en ligne (pastille verte) sous 5 s | P0 |
| 1.2 | Déconnexion | Fermer bob | Alice le passe hors ligne sous ~10 s, la carte reste | P0 |
| 1.3 | Retour | Relancer bob | Repasse en ligne, l'historique est intact | P0 |
| 1.4 | Changement d'IP | Basculer de Wi-Fi à Ethernet | La nouvelle adresse est prise en compte, l'envoi repart | P1 |
| 1.5 | Horloge reculée | Reculer l'horloge système de 2 min, attendre 10 s | **Les pairs restent en ligne.** Régression : ils disparaissaient tous d'un coup | P1 |

> 1.5 vérifie le débordement corrigé dans `discovery.rs`. Remettre l'horloge à
> l'heure après le test.

# 2. Messages

| # | Test | Actions | Attendu | P |
|---|---|---|---|---|
| 2.1 | Privé | Alice → bob | Reçu sous 1 s, horodaté | P0 |
| 2.2 | Diffusion « Tous » | Alice écrit dans « Tous » | Tous les pairs en ligne reçoivent | P0 |
| 2.3 | Salon | Écrire dans un groupe | Seuls les membres reçoivent | P0 |
| 2.4 | Non-membre | Carol hors du groupe | Ne reçoit rien, pas d'erreur | P0 |
| 2.5 | Message long | Coller > 4 000 caractères | Le fil se replie avec « Afficher la suite » | P2 |
| 2.6 | Collage énorme | Coller ~1 Mo de texte | Converti en pièce jointe `.txt`, pas de gel | P1 |
| 2.7 | Plafond de saisie | Dépasser la limite | Compteur rouge, envoi refusé | P2 |
| 2.8 | Réponse | Répondre à un message | Citation affichée des deux côtés | P1 |
| 2.9 | Doublon | Couper le réseau de bob à l'envoi, le rétablir | Le message n'apparaît qu'**une** fois chez bob | P0 |

# 3. Accusés de réception et de lecture

| # | Test | Actions | Attendu | P |
|---|---|---|---|---|
| 3.1 | Livraison | Alice → bob (fenêtre bob en arrière-plan) | Alice voit « reçu » | P0 |
| 3.2 | Lecture | Bob ouvre la conversation | Alice voit « lu » | P0 |
| 3.3 | **Retour du focus** | Fenêtre de bob **déjà ouverte sur la conversation privée** mais en arrière-plan. Alice envoie. Bob **clique sur sa fenêtre sans changer de conversation** | Alice voit « lu » sous 1 s. Régression : il fallait sortir du salon et y revenir | P0 |
| 3.4 | Retour du tray | Bob replie dans le tray, alice envoie, bob restaure | Accusé de lecture émis à la restauration | P1 |
| 3.5 | Pas de réémission | Alt-tab plusieurs fois sur une conversation lue | Aucun accusé réémis (vérifier les logs) | P2 |
| 3.6 | Portée des accusés | Envoyer dans un salon | Aucun accusé privé de livraison/lecture affiché : les accusés de groupe ne sont pas implémentés | P1 |
| 3.7 | **Accusé après commit** | Rendre la base non inscriptible (voir §11.2), alice → bob | Bob **n'acquitte pas**, alice réémet. Rien n'est annoncé « reçu » à tort | P0 |

# 4. Groupes

| # | Test | Actions | Attendu | P |
|---|---|---|---|---|
| 4.1 | Création | Alice crée « equipe » avec bob | Le salon apparaît chez les deux | P0 |
| 4.2 | Nom invalide | Espaces, > 50 octets, doublon | Refusé avec message | P1 |
| 4.3 | Ajout de membre | Alice ajoute carol | Carol reçoit le salon **et son historique reste vide** | P0 |
| 4.4 | Exclusion | Alice exclut carol | Le salon disparaît chez carol | P0 |
| 4.5 | Départ volontaire | Bob quitte | Historique local effacé chez bob, conservé chez alice | P0 |
| 4.6 | Succession | Le propriétaire part | Le premier membre restant devient propriétaire | P1 |
| 4.7 | Suppression | Alice supprime le salon | Disparaît chez tous les membres | P0 |
| 4.8 | Non-propriétaire | Bob tente de supprimer | Refusé | P0 |

| ID | Test | Actions | Attendu | P |
|---|---|---|---|---|
| 4.9 | Renommage backend | Renommer avec historique, réaction et réponse existants | Nom modifié ; ID, historique, réactions et repère de lecture intacts | P0 |
| 4.10 | Renommage puis redémarrage | Recharger après 4.9 | Nom et historique cohérents | P0 |

**4.9/4.10 :** le déclencheur GUI de renommage n'existe pas encore. Valider les tests de groupes et de stockage (`cargo test app::groups`, puis les tests de stockage) ; le parcours manuel à deux fenêtres reste à faire quand le bouton sera câblé. Aucun accusé privé n'est un prérequis de ce test de groupe.

# 5. Réactions

| # | Test | Actions | Attendu | P |
|---|---|---|---|---|
| 5.1 | Ajout | Réagir à un message | Visible des deux côtés | P0 |
| 5.2 | Bascule | Recliquer le même emoji | Retiré des deux côtés | P0 |
| 5.3 | Cumul | Alice et bob même emoji | Compteur à 2 | P1 |
| 5.4 | Persistance | Redémarrer | Réactions conservées | P0 |
| 5.5 | Plafond | Empiler > 32 emojis distincts sur un message | Plafonné, pas de croissance infinie | P2 |

# 6. Médias

| # | Test | Actions | Attendu | P |
|---|---|---|---|---|
| 6.1 | Image | Envoyer un PNG/JPG | Vignette cliquable, visionneuse plein écran | P0 |
| 6.2 | Fichier | Envoyer un PDF | Carte fichier téléchargeable | P0 |
| 6.3 | Dossier | Envoyer un dossier | Reçu en `.zip` ouvrable | P1 |
| 6.4 | Dossier illisible | Dossier contenant un fichier sans droit de lecture | **Erreur remontée** — pas de ZIP silencieusement incomplet | P1 |
| 6.5 | Téléchargement | Télécharger un média reçu | Écrit dans Téléchargements, **sans geler la fenêtre** | P0 |
| 6.6 | Nom en double | Télécharger deux fois | Suffixe « (1) », rien d'écrasé | P1 |
| 6.7 | Nom hostile | Pair envoyant `../../evil.txt` | Écrit dans Téléchargements uniquement, nom réduit | P0 |
| 6.8 | **Sous le seuil** | Envoyer un fichier de ~10 Mo | Accepté **sans confirmation** | P0 |
| 6.9 | **Au-dessus du seuil** | Envoyer un fichier > 50 Mo | Bandeau d'acceptation chez le destinataire | P0 |
| 6.10 | Refus | Refuser l'offre | Annoté « refusé » chez l'émetteur, rien écrit | P0 |
| 6.11 | Interruption | Fermer l'émetteur en cours de transfert | Carte « interrompu », **aucun fichier partiel** dans `media/` | P0 |
| 6.12 | Envoi massif | Sélectionner 50 fichiers d'un coup | **Interface fluide**, envois séquencés. Régression : un thread OS par fichier | P1 |

> 6.8 et 6.9 encadrent le seuil abaissé de 1 Gio à 50 Mo.

# 7. GIF et emojis

| # | Test | Actions | Attendu | P |
|---|---|---|---|---|
| 7.1 | Sélecteur | Ouvrir GIF / Mèmes / Stickers | Trois onglets peuplés | P1 |
| 7.2 | Recherche | Chercher un terme | Résultats, pagination au défilement | P1 |
| 7.3 | **Envoi et rendu** | Envoyer un GIF | **S'anime chez le destinataire** | P0 |
| 7.4 | Sans clé API | Retirer `ABCOM_KLIPY_API_KEY` | Message clair, pas de crash | P2 |
| 7.5 | Emojis | Sélecteur + `:shortcode` | Insertion et complétion | P1 |
| 7.6 | Survol du bouton GIF | Entrer, rester près du bord et ressortir, en clair/sombre aux échelles 100/150/200 %, sélecteur ouvert puis fermé | Cadre et texte immobiles ; infobulle et clic fonctionnels | P2 |

> **7.3 est critique.** Le filtre d'URL n'autorise que `https://*.klipy.com` ;
> le CDN observé est `static.klipy.com`, donc couvert. Si un GIF s'affiche en
> carte fichier au lieu de s'animer, c'est que Klipy a changé de domaine :
> l'ajouter à `ALLOWED_MEDIA_URL_HOSTS` dans `src/message/media.rs`.

# 8. Markdown et liens

| # | Test | Actions | Attendu | P |
|---|---|---|---|---|
| 8.1 | Formatage | `**gras**`, `_italique_`, `~~barré~~`, `` `code` `` | Rendus | P1 |
| 8.2 | Blocs | Bloc de code, citation, listes, tableau GFM | Rendus | P1 |
| 8.3 | Lien légitime | `[site](https://example.com)` | Cliquable | P1 |
| 8.4 | **Schéma hostile** | Recevoir `[Rapport](file:///etc/passwd)` puis `[doc](smb://serveur/partage)` | **Affichés en texte brut, non cliquables** | P0 |

# 9. Conversations et historique

| # | Test | Actions | Attendu | P |
|---|---|---|---|---|
| 9.1 | Non-lus | Recevoir sur une conversation fermée | Badge incrémenté, remis à zéro à l'ouverture | P0 |
| 9.2 | Épinglage | Épingler un pair et un salon | Remontent en tête, persistent après redémarrage | P2 |
| 9.3 | Sourdine | Mettre un salon en muet | Aucun son ni notification pour lui | P2 |
| 9.4 | Recherche | Rechercher un terme | Résultats, navigation vers le message | P1 |
| 9.5 | Pagination | Remonter dans un long historique | Charge par pages de 100 | P0 |
| 9.6 | **Pagination après débordement** | Générer > 600 messages dans une session (`scripts/seed-demo.py`), puis **remonter** | **Le chargement continue.** Régression : il s'arrêtait définitivement | P0 |
| 9.7 | Effacer l'historique | Effacer une conversation | Messages **et** leurs réactions/accusés partent immédiatement | P1 |
| 9.8 | Effacement ciblé | Après 9.7, vérifier une autre conversation | Intacte | P0 |
| 9.9 | Export | Exporter une conversation | Fichier `.txt` écrit, notification **après** écriture réelle | P1 |
| 9.10 | Export impossible | Exporter vers un dossier en lecture seule | **Erreur affichée**, pas de faux succès | P1 |

# 10. Sécurité

| # | Test | Actions | Attendu | P |
|---|---|---|---|---|
| 10.1 | Chiffrement | Observer le trafic (`tcpdump`, port 9000) | Aucun texte lisible | P0 |
| 10.2 | TOFU | Première connexion | Clé épinglée, empreinte visible dans Paramètres | P0 |
| 10.3 | **Changement de clé** | Supprimer `identity.key` de bob, relancer | Alice **refuse** et alerte sur l'usurpation possible | P0 |
| 10.4 | Ré-appairage | Accepter explicitement la nouvelle clé | Connexion rétablie | P1 |
| 10.5 | Passphrase | Lancer deux instances avec des passphrases différentes | **Aucune connexion** | P0 |
| 10.6 | Passphrase identique | Même passphrase des deux côtés | Connexion normale, démarrage < 1 s | P0 |
| 10.7 | Permissions de la clé | `ls -l identity.key` (Unix) | `-rw-------` (0600) | P0 |
| 10.8 | Permissions héritées | `chmod 644 identity.key`, relancer | **Resserré à 0600 au chargement** | P1 |
| 10.9 | Version de protocole | Un pair en version antérieure | Connexion refusée proprement | P0 |
| 10.10 | Usurpation de pseudo | Deux instances du même pseudo | Pas de mélange d'historique | P1 |

> 10.6 vérifie que l'étirement de la passphrase (~30 ms en release) ne se voit
> pas. En build debug il prend ~1 s : c'est normal, ne pas tester en debug.

# 11. Persistance et robustesse

| # | Test | Actions | Attendu | P |
|---|---|---|---|---|
| 11.1 | Redémarrage | Fermer et relancer | Historique, groupes, alias, avatars, accusés conservés | P0 |
| 11.2 | **Base non inscriptible** | `chmod 444 abcom.db`, écrire, quitter | Erreur journalisée `historique non sauvegardé` — pas de silence | P0 |
| 11.3 | File hors ligne | Bob éteint, alice envoie, bob revient | Le message est livré | P0 |
| 11.4 | **File durable** | Bob éteint, alice envoie, **tuer alice brutalement** (`kill -9`), relancer alice, rallumer bob | Le message part quand même. Régression : il était perdu | P0 |
| 11.5 | Compaction | Paramètres → compacter | Base réduite, aucune perte | P2 |
| 11.6 | Cache média | Laisser tourner > 15 min avec beaucoup de médias | Aucun fichier ne disparaît : plus aucune purge automatique | P1 |
| 11.7 | Purge manuelle | Paramètres → Stockage, saisir une durée, purger | L'aperçu annonce le volume, seuls les fichiers plus vieux partent, les images du fil sont épargnées si la case est décochée | P1 |
| 11.8 | Envoi sans doublon | Envoyer un fichier, regarder `media/` chez l'émetteur | Aucune copie créée ; l'original reste seul sur le disque | P2 |
| 11.9 | Instance hors plage | `ABCOM_INSTANCE=999999` | Démarre sur un port valide, **sans panique** | P2 |
| 11.10 | Migration | Reprendre une sauvegarde de version antérieure selon la procédure ci-dessous | Historique et références conservés | P0 |

## 11.10 — Migration depuis une version antérieure (P0)

À faire **une fois**, avec une vraie base d'avant la refonte des groupes.

1. Partir d'une installation antérieure contenant un salon, des messages, des
   réactions et des accusés.
2. Sauvegarder le répertoire de données.
3. Installer cette version, lancer.

**Attendu** : historique, réactions, accusés et repères de lecture intacts, le
salon garde son nom. Ensuite, appliquer 4.9 (renommage) : tout doit tenir.

> ⚠️ **Le protocole passe en version 3.** Un pair resté en version antérieure ne
> se connectera plus. Le déploiement doit se faire sur toutes les machines en
> même temps.

# 12. Interface et système

| # | Test | Actions | Attendu | P |
|---|---|---|---|---|
| 12.1 | Thèmes | Basculer clair / sombre / système | Tout reste lisible, aucun texte invisible | P1 |
| 12.2 | Langue | Basculer FR / EN | Tous les libellés suivent ; la langue reste un choix de session | P1 |
| 12.3 | Avatar | Choisir, changer, retirer | Propagé aux pairs | P1 |
| 12.4 | Alias | Renommer un contact | Local, persistant | P2 |
| 12.5 | Tray | Fermer la fenêtre | Repli dans le tray, l'app tourne | P0 |
| 12.6 | Restauration | Cliquer l'icône du tray | Fenêtre restaurée, état intact | P0 |
| 12.7 | Notification native | Message reçu fenêtre repliée | Notification système | P1 |
| 12.8 | Son | Message reçu hors conversation active | Son joué ; silence si conversation ouverte | P2 |
| 12.9 | Badge | Messages non lus | Badge sur l'icône du tray | P2 |
| 12.10 | Autostart | Activer, redémarrer la session | L'app démarre | P2 |
| 12.11 | Redimensionnement | Fenêtre très étroite / très large | Pas de chevauchement | P1 |
| 12.12 | Mode conservé | Choisir Sombre, quitter, relancer | Toujours sombre. Régression : retour à « système » | P1 |
| 12.13 | Dix thèmes | Paramètres → Apparence → Thèmes : essayer chacun, clair puis sombre | Texte lisible partout, aucune zone restée de l'ancien thème | P1 |
| 12.14 | Taille générale | Tout agrandir à 160 % | Interface et conversations grossissent ; **les emojis gardent leur taille** | P1 |
| 12.15 | Tailles séparées | Conversations à 140 %, interface à 80 % | Messages plus grands que les menus | P1 |
| 12.16 | Couleur personnelle | Changer le fond d'un thème intégré | Une copie « (perso) » apparaît, l'original est intact, le contraste s'affiche | P1 |
| 12.17 | Contraste | Mettre le texte de la couleur du fond | Avertissement rouge sous 4,5:1, l'appli reste utilisable | P1 |
| 12.18 | Police proposée | Choisir chacune des huit, pour les conversations | Messages dans la bonne police, accents français et « Łódź » compris | P1 |
| 12.19 | Import `.ttf` / `.otf` / `.ttc` | Importer une police du système | Nom de famille affiché, police appliquée, conservée au redémarrage | P1 |
| 12.20 | Import `.woff` / `.woff2` | Importer une police web | Idem | P1 |
| 12.21 | Faux fichier de police | Renommer un `.txt` en `.ttf` et l'importer | Refus avec la raison, rien ne change, **pas de plantage** | P0 |
| 12.22 | Police supprimée | Supprimer le fichier dans `fonts/`, relancer | Retour à la police du thème avec un avis | P1 |
| 12.23 | Polices en ligne | Case décochée (défaut) | **Aucune requête** réseau, ouvrir l'onglet y compris | P0 |
| 12.24 | Polices en ligne, activées | Cocher, rechercher « lex », choisir Lexend | Police téléchargée, appliquée, utilisable hors ligne ensuite | P1 |
| 12.25 | Plus petit écran | Fenêtre minimale (560 × 360), texte à 160 % | Paramètres entiers dans la fenêtre, tout défile, rien de coupé | P1 |
| 12.26 | Écran Retina et ordinaire | Déplacer la fenêtre de l'un à l'autre | Texte net, rien ne se déplace | P2 |
| 12.27 | Lecteur d'écran | VoiceOver (macOS) ou NVDA (Windows) sur Apparence | Chaque contrôle est annoncé avec son nom, une carte de thème dit si elle est sélectionnée | P2 |
| 12.28 | Sélecteur natif | Importer une police sur chaque OS | La feuille s'ouvre sans geler la fenêtre ; annuler ne change rien | P0 |
| 12.29 | Emojis inchangés | Fil, emoji seul, saisie, réactions, dans trois thèmes et à 80 / 160 % | Mêmes tailles et même alignement partout | P1 |
| 12.30 | Stockage après calcul | Attendre le tableau rempli, tester en FR/EN à 100 / 160 %, réduire à 560 × 360 puis défiler jusqu'aux boutons | Catégories, tailles et nombres lisibles, aucune colonne hors fenêtre, actions de purge et d'ouverture accessibles | P1 |
| 12.31 | Tableaux des paramètres | Parcourir Général et tous les Crédits, jusqu'aux palettes, à 100 / 160 % | Libellés lisibles, valeurs alignées, défilement sans zone imbriquée | P1 |
| 12.32 | Lecture de la licence | À 100 puis 160 %, lire le début puis défiler jusqu'au dernier paragraphe | Texte agrandi et replié à la largeur disponible, sans indentation excessive ni coupure horizontale | P1 |

---

# 13. Spécificités par OS

Le cœur est identique partout ; ces points diffèrent réellement.

## Linux

| # | Test | Attendu |
|---|---|---|
| L1 | `scripts/abcom-install.sh` | Binaire et **raccourci menu fonctionnel**, aucun nouveau service systemd |
| L2 | **Lancer depuis le menu** | L'app démarre. Régression : `%h` n'était pas développé, le raccourci ne lançait rien |
| L3 | Démarrage unique | Après installation puis reconnexion de session : **une seule** instance tourne. `systemctl --user status abcom` doit répondre que l'unité n'existe pas — le démarrage passe uniquement par `~/.config/autostart/` |
| L6 | Build minimal | `cargo build --release --no-default-features` sur une machine **sans GTK ni ALSA** : doit compiler. L'app démarre sans tray (fermer la fenêtre quitte) et sans bip |
| L7 | Alpine / musl | Installer les paquets de [docs/06](06-installation.md), compiler, lancer. Binaire lié dynamiquement à musl |
| L8 | KSNI et apparition du watcher | Sur un bus D-Bus privé, créer le tray, changer le badge et quitter ; sur un vrai bureau, vérifier Ouvrir/Quitter, repli/restauration et watcher démarrant après l'application |
| L9 | Logo Linux | Après installation, le lanceur utilise le nouveau logo ; tray couleur et badge non lu lisibles sur le panneau cible |
| L10 | AppImage | Télécharger/vérifier le fichier, lancer avec et sans FUSE, vérifier le chemin d’autostart, le tray, le son et les échanges LAN sur le bureau cible |
| L11 | Flatpak | Installer le bundle, lancer, joindre/exporter par portail, vérifier données isolées et absence d’autostart trompeur ; tester tray/notifications/LAN, puis désinstaller sans effacer les données |
| L4 | Wayland et X11 | Tester les deux : tray, notifications, focus (test 3.3) |
| L5 | Tray selon l'environnement | StatusNotifierItem nécessaire ; GNOME peut demander une extension ; vérifier KDE/XFCE |

> L3 vérifie la correction du double autostart : les installateurs ne posent
> plus de service systemd, le démarrage est géré par l'application seule
> (entrée XDG, réglable dans Paramètres → Général).

## macOS

| # | Test | Attendu |
|---|---|---|
| M1 | Repli | Disparaît du Dock, reste dans la barre de menus |
| M2 | Restauration | Revient dans le Dock |
| M3 | Découverte locale | Deux instances sur la même machine se voient (multicast loopback) |
| M4 | Permissions réseau | Le pare-feu demande l'autorisation au premier lancement |
| M5 | Focus (test 3.3) | Cliquer une fenêtre en arrière-plan déclenche l'accusé de lecture |
| M6 | Permissions de la clé | `stat -f "%Sp" identity.key` → `-rw-------` |
| M7 | Logos natifs | Nouveau logo Finder/Dock dans le bundle ; repli/restauration conserve le logo ; barre de menus lisible sur fonds clair/sombre et Retina, avec et sans non-lus Exécuter aussi `cargo run --locked --example check_macos_dock_after_frame` pour contrôler l’icône après le rendu du framework. |
| M8 | Homebrew | Installation, réinstallation et désinstallation via le tap local ; vérifier lancement réel, signature, chemin du démarrage automatique et conservation des données ; distinguer build local et téléchargement en quarantaine |

## Windows

| # | Test | Attendu |
|---|---|---|
| W1 | Repli | Sort de la barre des tâches, **le menu du tray reste actif** |
| W2 | Restauration | Fenêtre replacée à l'écran |
| W3 | ACL de la clé | `icacls identity.key` → accès limité à l'utilisateur courant |
| W4 | Pare-feu | Autoriser au premier lancement, puis vérifier la découverte |
| W5 | Chemins avec espaces | Envoyer un fichier depuis un dossier au nom espacé |
| W6 | Focus (test 3.3) | Idem, avec la fenêtre restaurée depuis le tray |
| W7 | Logos natifs | Explorer, raccourcis et barre des tâches utilisent le nouveau logo aux différentes échelles ; tray couleur avec badge visible |
| W8 | WinGet | Valider le manifeste local, installer et réinstaller l’installateur, vérifier menu Démarrer et lancement, désinstaller ; données conservées, raccourci et démarrage automatique retirés |

---

# 14. Passe minimale avant livraison

Si le temps manque, ces tests seuls :

1. **1.1** découverte — **2.1** message privé — **2.3** salon
2. **3.3** accusé de lecture privé au retour du focus
3. **4.9** renommage de salon *(backend ; parcours GUI en attente)*
4. **6.1**, **6.8**, **6.9** médias et seuil
5. **7.3** rendu d'un GIF
6. **10.3** changement de clé — **10.5** passphrase
7. **11.1** redémarrage — **11.10** migration
8. **12.5 / 12.6** tray, sur chaque OS cible

# 15. Ce que la suite automatique couvre déjà

Inutile de le refaire à la main : hachage des messages, dérivation et migration
des identifiants de salon, filtrage d'URL de médias, schémas de liens markdown,
péremption des pairs, bornes des réactions, durabilité de la file d'envoi,
pagination après débordement, purge en cascade, framing et handshake Noise,
signature des annonces, path traversal, et l'apparence : persistance et redémarrage,
validation et décodage des polices (formats web, bombes de décompression, fichiers
tronqués), catalogue en ligne simulé (aucune requête sans accord, annulation,
erreurs réseau), contraste de chaque thème, et toute l'application rejouée sous
chaque thème, taille de texte, taille de fenêtre et densité. Les paramètres sont
aussi vérifiés avec le tableau de stockage rempli, les textes des grilles en
FR/EN à 100 / 160 %, la licence agrandie sans altération de ses mots et l'accès
aux réglages de purge en petite fenêtre. Il reste à la main :
le rendu sur un vrai écran, le sélecteur natif de chaque OS et le lecteur d'écran
(12.25 à 12.29).

```bash
make check         # suite complète avec toutes les fonctionnalités
cargo test --release --all-features --locked   # mesures avant livraison
```
