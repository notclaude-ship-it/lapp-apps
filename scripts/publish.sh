#!/usr/bin/env bash
# publish.sh : compile des apps (aarch64) et les publie sur le store.
#
# Tourne sur les serveurs GitHub (voir .github/workflows/build.yml) et aussi en local.
# Le paquet .lapp n'est PAS signé ici : on envoie l'arbre compilé (tar.gz) au store, qui le
# signe avec la clé qui reste sur le PC (route PUT /stage/). La requête est authentifiée par une
# signature HMAC : le jeton ne circule jamais sur le réseau.
#
# Usage : scripts/publish.sh apps/<dossier> [apps/<dossier> ...]
#
# Variables :
#   STORE_URL         adresse du store, ex. http://natalos.duckdns.org:10078   (obligatoire)
#   LAPP_STORE_TOKEN  jeton du store                                           (obligatoire sauf PUBLISH=0)
#   PUBLISH=0         compile seulement, sans rien envoyer (pull request)
#
# Une version déjà présente sur le store est ignorée : incrémente `version` dans manifest.toml.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PUBLISH="${PUBLISH:-1}"
STORE_URL="${STORE_URL:-}"; STORE_URL="${STORE_URL%/}"
TARGET=aarch64-unknown-linux-gnu
export PATH="$HOME/.cargo/bin:$PATH"

field() { sed -n "s/^$1[[:space:]]*=[[:space:]]*\"\(.*\)\".*/\1/p" "$2" | head -n 1; }

if [ "$#" -eq 0 ]; then echo "publish.sh : aucune app à traiter"; exit 0; fi
if [ "$PUBLISH" != 0 ]; then
  if [ -z "$STORE_URL" ]; then echo "::error::STORE_URL absent (variable du dépôt : Settings > Variables)"; exit 1; fi
  if [ -z "${LAPP_STORE_TOKEN:-}" ]; then echo "::error::LAPP_STORE_TOKEN absent (secret du dépôt)"; exit 1; fi
  if [ "$(curl -s -m 20 "$STORE_URL/health" || true)" != ok ]; then
    echo "::error::le store ne répond pas sur $STORE_URL (PC éteint, port non redirigé ou mauvaise adresse ?)"; exit 1
  fi
fi

fail=0
for dir in "$@"; do
  dir="${dir%/}"
  proj="$ROOT/$dir"; m="$proj/manifest.toml"
  if [ ! -f "$m" ]; then echo "::warning::$dir : pas de manifest.toml, ignoré"; continue; fi
  id="$(field id "$m")"; ver="$(field version "$m")"; exe="$(field exec "$m")"; icon="$(field icon "$m")"
  if [ -z "$id" ] || [ -z "$ver" ] || [ -z "$exe" ]; then echo "::error::$dir : manifest.toml incomplet (id, version, exec)"; fail=1; continue; fi
  file="$id-$ver.lapp"
  echo "=== $dir -> $file"

  if [ "$PUBLISH" != 0 ]; then
    code="$(curl -s -m 30 -o /dev/null -I -w '%{http_code}' "$STORE_URL/files/$file" || echo 000)"
    if [ "$code" = 200 ]; then
      echo "::notice::$file est déjà sur le store : incrémente la version dans $dir/manifest.toml pour publier"
      continue
    elif [ "$code" != 404 ]; then
      echo "::error::store injoignable pour $file (HTTP $code)"; fail=1; continue
    fi
  fi

  # --- compilation + arbre à empaqueter (même assemblage que lapp-build) ---
  if ! ( cd "$proj" && cargo build --release --target "$TARGET" ); then
    echo "::error::compilation de $dir échouée"; fail=1; continue
  fi
  stage="$proj/stage"; rm -rf "$stage"
  mkdir -p "$stage/$(dirname "$exe")"
  cp "$proj/target/$TARGET/release/$(basename "$exe")" "$stage/$exe"
  cp "$m" "$stage/"
  if [ -n "$icon" ]; then
    if [ -f "$proj/$icon" ]; then cp "$proj/$icon" "$stage/$icon"; else echo "::warning::$dir : icône $icon absente"; fi
  fi
  [ -d "$proj/assets" ] && cp -r "$proj/assets" "$stage/assets"

  if [ "$PUBLISH" = 0 ]; then
    echo "::notice::$file compilé (publication désactivée)"
    continue
  fi

  # --- envoi signé (HMAC) au store, qui signe le paquet avec sa clé ---
  tgz="$(mktemp "${TMPDIR:-/tmp}/stage-XXXXXX.tar.gz")"
  tar -czf "$tgz" -C "$stage" .
  ts="$(date +%s)"
  sha="$(sha256sum "$tgz" | cut -d' ' -f1)"
  sig="$(printf '%s.%s.%s' "$ts" "$file" "$sha" | openssl dgst -sha256 -hmac "$LAPP_STORE_TOKEN" -hex | awk '{print $NF}')"
  resp="$(curl -sS --connect-timeout 20 --max-time 900 -X PUT \
          -H "X-Lapp-Timestamp: $ts" -H "X-Lapp-Signature: $sig" \
          --data-binary "@$tgz" -w '\nHTTP %{http_code}' "$STORE_URL/stage/$file")"
  rm -f "$tgz"
  echo "$resp"
  case "$resp" in
    *"HTTP 201") echo "::notice::$file publié et signé sur le store" ;;
    *"HTTP 409") echo "::notice::$file existait déjà sur le store" ;;
    *) echo "::error::publication de $file refusée"; fail=1 ;;
  esac
done
exit "$fail"