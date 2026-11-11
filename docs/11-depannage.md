# 11 — Dépannage et FAQ

[English](en/11-troubleshooting.md) · **Français** · [Documentation](README.fr.md)

## Les ordinateurs ne se voient pas

1. Vérifier que les deux utilisent le même réseau local. Un Wi-Fi invité, l'isolation des clients, un VPN ou des sous-réseaux différents peuvent empêcher la découverte.
2. Autoriser Abcom dans le pare-feu du système sur le réseau de confiance. Par défaut : UDP 9001 pour la découverte, TCP 9000 et 9001 pour les échanges.
3. Choisir des pseudos différents et vérifier que les versions du protocole sont compatibles.
4. Si `ABCOM_PASSPHRASE` est définie, utiliser la même valeur des deux côtés sans la publier dans un rapport de bug.
5. Essayer `make run2` sur un même poste pour distinguer un problème applicatif d'une restriction réseau.

Les [ports et annonces](03-reseau-et-securite.md) sont décrits dans le guide réseau. Les annonces sont périodiques : laisser quelques secondes après le lancement.

## L'application a disparu après un clic sur la croix

Avec le tray actif, la croix masque la fenêtre. Utiliser l'icône de la barre de menus macOS ou de la zone de notification Windows/Linux pour ouvrir Abcom ou quitter réellement. Sans tray disponible, la croix quitte.

## Mes réglages ne reviennent pas

- Quitter par le menu du tray, puis relancer la même instance.
- Vérifier `ABCOM_INSTANCE` : chaque numéro a son propre dossier de données.
- Vérifier les droits d'écriture et les avis de sauvegarde. Une apparence peut être appliquée pour la session malgré un échec d'enregistrement.
- Les thèmes, tailles, polices, mode clair/sombre et règles de purge sont persistés. **La langue reste un choix de session** et revient actuellement au français.
- Une police supprimée ou illisible est remplacée par celle du thème, avec un avis.

Voir [stockage et préférences](04-stockage.md).

## Un message privé attend

Un pair hors ligne peut recevoir le message à son retour : la file privée est persistée et les messages non acquittés sont réémis. Vérifier aussi les alertes de changement de clé et la disponibilité du pair. Les salons et « Tous » n'ont pas de rattrapage des messages pour les absents.

## La clé d'un contact a changé

Cela peut venir d'une réinstallation, d'un autre dossier d'instance ou d'une usurpation. Comparer la nouvelle empreinte avec le contact par un canal de confiance avant d'accepter la nouvelle clé. Effacer aveuglément les données de confiance contournerait précisément cette protection.

## Les GIF ou polices en ligne ne fonctionnent pas

Les GIF nécessitent Internet et une clé Klipy. Chaque destinataire charge le GIF depuis le CDN autorisé ; une URL hors liste est affichée comme fichier. Le catalogue de polices nécessite un accord dans Paramètres → Apparence → Polices. Les polices intégrées et importées localement restent utilisables hors ligne. Voir les [accès réseau optionnels](03-reseau-et-securite.md#accès-réseau-optionnels-hors-lan).

## Le stockage grossit

Il n'y a pas de purge automatique des pièces jointes. Dans Paramètres → Stockage, attendre le calcul, choisir une durée et lire l'aperçu avant de purger. Zéro jour signifie nettoyer les transferts inachevés et orphelins, pas tout effacer. Les images du fil sont épargnées si la case n'est pas cochée. Le message reste dans l'historique après suppression de sa pièce jointe.

Après suppression de conversations, compacter la base dans Paramètres → Général → Données pour récupérer de l'espace. [Détail de la conservation](04-stockage.md#le-dossier-media).

## Sauvegarder ou changer d'ordinateur

Quitter réellement Abcom et copier le **dossier de données complet**, pas seulement `abcom.db`. Il contient aussi la clé d'identité, les médias, thèmes et polices. Conserver cette sauvegarde dans un emplacement privé. Une copie à chaud doit tenir compte du WAL SQLite ; la procédure simple consiste à arrêter l'application avant la copie.

Restaurer avec Abcom arrêté. Un export texte de conversation ne remplace pas cette sauvegarde. Les [chemins par plateforme](04-stockage.md#répertoire-de-données) sont documentés.

## Pourquoi compiler échoue sous Linux ?

Installer les paquets de développement listés dans le [guide d'installation](06-installation.md#dépendances-système-sous-linux). `--no-default-features` retire tray et son, mais l'application reste graphique. Il faut toujours un environnement de fenêtrage et un pilote compatible. Un binaire musl entièrement statique n'est pas pris en charge.

## Abcom est-il utilisable sans Internet ?

Oui pour la découverte locale, les conversations, les fichiers, les polices embarquées et les imports locaux. Les GIF, le catalogue de polices et l'ouverture de liens externes ont leurs propres accès Internet. Aucun serveur central n'est nécessaire à la messagerie.

## Les données sont-elles chiffrées ?

Les sessions de chat et de transfert utilisent Noise. Les annonces de présence sont visibles sur le LAN. L'historique, les médias et les sauvegardes ne sont pas chiffrés au repos. La première identité rencontrée doit être vérifiée pour un échange sensible. Lire le [modèle de menace](03-reseau-et-securite.md#modèle-de-menace).

## Signaler un problème

Dans une [issue](https://github.com/abnd-corp/Abcom/issues), indiquer : version ou commit, OS et architecture, mode de compilation, étapes minimales, résultat attendu et observé. Pour un défaut visuel, préciser taille de fenêtre, densité, thème et taille du texte.

Les journaux sont dans `logs/` du dossier de données ; une panique peut produire `last-panic.txt`. Relire et expurger tout extrait avant de le partager. Ne jamais joindre `.env`, `identity.key`, la base de conversations ou une sauvegarde complète ; masquer les noms et adresses privés dans les captures.
