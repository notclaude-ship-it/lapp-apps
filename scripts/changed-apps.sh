#!/usr/bin/env bash
# changed-apps.sh : liste les dossiers d'apps à reconstruire (un par ligne, ex. apps/notes).
#
# Usage : scripts/changed-apps.sh <commit-de-base> [dossier-d-app]
#   - dossier-d-app donné      : seulement celui-là
#   - base vide ou 0000…       : toutes les apps
#   - lapp-sdk/ ou scripts/ modifiés : toutes les apps (le SDK est partagé)
#   - sinon                     : les apps dont un fichier a changé
set -euo pipefail
cd "$(dirname "$0")/.."

all() { for d in apps/*/; do [ -f "${d}manifest.toml" ] && echo "${d%/}"; done; true; }

base="${1:-}"; only="${2:-}"
if [ -n "$only" ]; then
  only="${only%/}"; case "$only" in apps/*) ;; *) only="apps/$only" ;; esac
  [ -f "$only/manifest.toml" ] && echo "$only"
  exit 0
fi
if [ -z "$base" ] || [ "${base//0/}" = "" ] || ! git cat-file -e "$base^{commit}" 2>/dev/null; then
  all; exit 0
fi
changed="$(git diff --name-only "$base" HEAD)"
if printf '%s\n' "$changed" | grep -qE '^(lapp-sdk|scripts)/'; then
  all; exit 0
fi
printf '%s\n' "$changed" | awk -F/ '$1=="apps" && NF>2 {print "apps/"$2}' | sort -u | while read -r d; do
  [ -f "$d/manifest.toml" ] && echo "$d"
done
true