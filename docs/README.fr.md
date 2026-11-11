[English](README.md) · **Français** · [Projet](../README.fr.md)

# Documentation Abcom

Commencer par l'installation et le guide utilisateur. Les références techniques décrivent l'implémentation actuelle ; les mesures historiques sont datées dans le journal des audits.

| Guide | Contenu |
|---|---|
| [Présentation](01-presentation.md) | Usage, choix du projet et vocabulaire |
| [Installation](06-installation.md) | Compilation, paquets système et premier lancement |
| [Guide utilisateur](05-fonctionnalites.md) | Conversations, fichiers, notifications et apparence |
| [Dépannage et FAQ](11-depannage.md) | Découverte, réglages, sauvegardes et rapports de bugs |
| [Réseau et sécurité](03-reseau-et-securite.md) | Ports, identité, chiffrement et accès Internet |
| [Stockage local](04-stockage.md) | Fichiers, persistance, conservation et migrations |
| [Développement](07-developpement.md) | Contrôles, contribution et CI |
| [Architecture](02-architecture.md) | Modules, threads, rendu et parcours des messages |
| [Cahier de tests](10-cahier-de-tests.md) | Scénarios manuels et couverture automatique |
| [Limites connues](09-limites-et-pistes.md) | Contraintes actuelles et pistes |

## Références de maintenance

- [Mesurer les performances](12-performances.md) : scénarios reproductibles, outils et interprétation.
- [Journal de maintenance et audits](08-historique-et-audits.md) : résultats datés et avis de dépendances non résolus.

## Maintenir ces guides

L'anglais et le français couvrent les mêmes fonctionnalités et limites. Modifier les deux pages d'une paire quand le comportement change ; conserver les mêmes commandes, limites et identifiants de tests. Chaque guide mène directement à sa traduction. Les noms français existants restent stables pour préserver les références du code.

Lancer `python3 scripts/check_docs.py` depuis la racine pour vérifier les paires de langues, liens locaux, ancres et identifiants de tests manuels. Les comptes rendus historiques gardent leurs dates et leurs résultats. Le changelog, le texte de licence et les fichiers de gouvernance font référence dans leur langue d'origine ; les guides anglais y renvoient explicitement.
