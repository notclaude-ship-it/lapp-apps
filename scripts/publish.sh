#!/usr/bin/env bash
# publish.sh : compile des apps .lapp (lapp-build) et les publie sur le store.
#
# Usage : scripts/publish.sh apps/<dossier> [apps/<dossier> ...]
#
# Variables :
#   STORE_URL         adresse du store            (défaut http://127.0.0.1:10079)
#   LAPP_STORE_TOKEN  jeton de publication        (obligatoire sauf si PUBLISH=0)
#   PUBLISH=0         compile seulement, sans envoyer au store (ex. pull request)
#   LAPP              binaire lapp                (défaut ~/lapp/target/release/lapp)
#   LAPP_KEY          clé de signature            (défaut ~/.lapp/dev.key, voir lapp-build)
#
# Une version déjà présente sur le store n'est ni recompilée ni écrasée : il faut incrémenter
# `version` dans manifest.toml pour publier une nouvelle version.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
STORE_URL="${STORE_URL:-http://127.0.0.1:10079}"
PUBLISH="${PUBLISH:-1}"
export LAPP="${LAPP:-$HOME/lapp/target/release/lapp}"

field() { sed -n "s/^$1[[:space:]]*=[[:space:]]*\"\(.*\)\".*/\1/p" "$2" | head -n 1; }

if [ "$#" -eq 0 ]; then echo "publish.sh : aucune app à traiter"; exit 0; fi
if [ "$PUBLISH" != 0 ] && [ -z "${LAPP_STORE_TOKEN:-}" ]; then
  echo "::error::LAPP_STORE_TOKEN absent (secret du dépôt)"; exit 1
fi

fail=0
for dir in "$@"; do
  dir="${dir%/}"
  m="$ROOT/$dir/manifest.toml"
  if [ ! -f "$m" ]; then echo "::warning::$dir : pas de manifest.toml, ignoré"; continue; fi
  id="$(field id "$m")"; ver="$(field version "$m")"
  if [ -z "$id" ] || [ -z "$ver" ]; then echo "::error::$dir : manifest.toml incomplet (id, version)"; fail=1; continue; fi
  file="$id-$ver.lapp"
  echo "=== $dir -> $file"

  if [ "$PUBLISH" != 0 ]; then
    code="$(curl -s -o /dev/null -I -w '%{http_code}' "$STORE_URL/files/$file" || echo 000)"
    if [ "$code" = 200 ]; then
      echo "::notice::$file est déjà sur le store : incrémente la version dans $dir/manifest.toml pour publier"
      continue
    elif [ "$code" != 404 ]; then
      echo "::error::store injoignable ($STORE_URL, HTTP $code)"; fail=1; continue
    fi
  fi

  rm -f "$ROOT/$dir"/*.lapp
  if ! "$ROOT/lapp-sdk/tools/lapp-build" "$ROOT/$dir"; then
    echo "::error::compilation de $dir échouée"; fail=1; continue
  fi
  if [ ! -f "$ROOT/$dir/$file" ]; then
    echo "::error::$file non produit (id/version du manifeste ?)"; fail=1; continue
  fi

  if [ "$PUBLISH" = 0 ]; then
    echo "::notice::$file compilé (publication désactivée)"
    continue
  fi
  resp="$(curl -sS -X PUT -H "Authorization: Bearer $LAPP_STORE_TOKEN" --data-binary "@$ROOT/$dir/$file" \
          -w '\nHTTP %{http_code}' "$STORE_URL/upload/$file")"
  echo "$resp"
  case "$resp" in
    *"HTTP 201") echo "::notice::$file publié sur le store" ;;
    *) echo "::error::publication de $file refusée"; fail=1 ;;
  esac
done
exit "$fail"