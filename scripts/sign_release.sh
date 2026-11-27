#!/usr/bin/env bash
# Signe chaque fichier de release avec minisign, puis vérifie la signature avec la clé publique
# du dépôt. Le commentaire de confiance « abcom <version> <fichier> » lie le fichier à sa version :
# l'application refuse un ancien artefact signé resservi comme nouveau.
# Usage : SIGNING_KEY=<clé secrète minisign> sign_release.sh <dossier> <version> <clé publique>
set -euo pipefail

dir=$1
version=$2
public_key=$3

if [ -z "${SIGNING_KEY:-}" ]; then
    echo "::error::Secret ABCOM_UPDATE_SIGNING_KEY absent de l'environnement release." >&2
    exit 1
fi
if ! grep -q '^RW' "$public_key"; then
    echo "::error::$public_key ne contient pas de clé publique minisign." >&2
    exit 1
fi

key_file=$(mktemp)
trap 'rm -f "$key_file"' EXIT
chmod 600 "$key_file"
printf '%s\n' "$SIGNING_KEY" > "$key_file"

for file in "$dir"/*; do
    case "$file" in
        *.minisig) continue ;;
    esac
    name=$(basename "$file")
    minisign -S -s "$key_file" -m "$file" -t "abcom $version $name" < /dev/null
    minisign -V -q -p "$public_key" -m "$file"
    echo "signé : $name"
done
