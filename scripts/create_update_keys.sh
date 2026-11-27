#!/usr/bin/env bash
# Crée les clés de signature des mises à jour. À lancer une seule fois, par un dirigeant (R18) :
# - clé de release : sa partie secrète part directement dans le secret GitHub
#   ABCOM_UPDATE_SIGNING_KEY (environnement release), sans jamais s'afficher ni rester sur le disque ;
# - clé de secours : protégée par un mot de passe demandé ici, à ranger hors ligne
#   (gestionnaire de mots de passe), pour remplacer la clé de release si elle était perdue.
# Les deux clés publiques sont écrites dans assets/update/, à committer.
set -euo pipefail

repo=abnd-corp/Abcom
root=$(git rev-parse --show-toplevel)
backup_dir="$HOME/Desktop/abcom-cle-de-secours"

if ! command -v minisign > /dev/null; then
    if command -v brew > /dev/null; then
        brew install minisign
    else
        echo "Installer minisign (https://jedisct1.github.io/minisign/), puis relancer." >&2
        exit 1
    fi
fi
gh auth status > /dev/null

if grep -q '^RW' "$root/assets/update/release.pub"; then
    echo "Une clé de release existe déjà dans assets/update/release.pub : rien n'est modifié." >&2
    exit 1
fi
if [ -e "$backup_dir" ]; then
    echo "$backup_dir existe déjà : le déplacer ou le supprimer d'abord." >&2
    exit 1
fi

umask 077
work=$(mktemp -d)
trap 'rm -f "$work/release.key" "$work/release.pub"; rmdir "$work"' EXIT

echo "1/2 Clé de release : création et envoi dans GitHub…"
minisign -G -W -p "$work/release.pub" -s "$work/release.key" > /dev/null
gh secret set ABCOM_UPDATE_SIGNING_KEY --env release --repo "$repo" < "$work/release.key"
cp "$work/release.pub" "$root/assets/update/release.pub"

echo "2/2 Clé de secours : choisissez un mot de passe solide, et gardez-le avec le fichier."
mkdir -p "$backup_dir"
minisign -G -p "$backup_dir/backup.pub" -s "$backup_dir/backup.key"
cp "$backup_dir/backup.pub" "$root/assets/update/backup.pub"

cat <<DONE

Terminé.
- Secret ABCOM_UPDATE_SIGNING_KEY déposé dans l'environnement release de $repo.
- Clés publiques écrites dans assets/update/ (à committer).
- Rangez $backup_dir/backup.key et son mot de passe dans votre gestionnaire de mots de passe,
  puis supprimez le dossier $backup_dir.
DONE
