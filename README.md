# lapp-apps

Dépôt des applications `.lapp`. **Un push sur `main` compile les apps modifiées sur les serveurs
GitHub, puis les publie automatiquement sur le store** : plus de compilation manuelle.

```
apps/<nom>/     une app par dossier (manifest.toml, Cargo.toml, src/, assets/, icon.png)
lapp-sdk/       le SDK, partagé par toutes les apps
scripts/        publish.sh (compile + envoie), changed-apps.sh (quelles apps ont changé)
.github/workflows/build.yml   la chaîne automatique
```

## Publier une app

1. Créer le projet **dans `apps/`** :
   ```sh
   lapp-sdk/tools/lapp-new com.moi.notes --name Notes --dir apps/notes
   ```
2. Dans `apps/notes/Cargo.toml`, remplacer le chemin absolu du SDK par le chemin relatif du dépôt :
   ```toml
   lapp-sdk = { path = "../../lapp-sdk" }
   ```
3. Tester sur PC (`cargo run -- --screenshot out.png`), puis `git push` sur `main` (ou via une pull request).

L'app apparaît ensuite dans l'app Store de l'appareil. L'avancement se suit dans l'onglet *Actions*.

## Comment ça marche

```
push → GitHub (ubuntu-latest) : cargo build --release --target aarch64
     → arbre de l'app (binaire + manifest + icône + assets) en tar.gz
     → PUT /stage/<id>-<version>.lapp sur le store, avec une signature HMAC
     → le PC du store vérifie, signe avec SA clé (lapp pack) et publie
```

La **clé de signature ne quitte jamais le PC** : GitHub ne l'a pas. Le jeton du store (secret
`LAPP_STORE_TOKEN`) ne circule pas non plus sur le réseau : seule une signature HMAC de la requête
(horodatée, valable 5 minutes, liée au fichier envoyé) est transmise.

## Règles

- **Chaque publication = une nouvelle version.** Le store n'écrase jamais un paquet : si
  `<id>-<version>.lapp` existe déjà, l'app est ignorée avec un message. Incrémentez `version` dans
  `manifest.toml` (et `Cargo.toml`) pour publier.
- **Seules les apps modifiées sont reconstruites.** Si `lapp-sdk/` ou `scripts/` change, toutes les apps le sont.
- **Pull request** : l'app est compilée pour vérifier qu'elle passe, sans rien publier. Les PR venant de forks
  ne sont pas construites.
- **Relancer à la main** : onglet *Actions* → *Compiler et publier les apps* → *Run workflow*
  (dossier d'une app, ou vide pour toutes).
- **Permissions du manifeste** : ne demandez que le nécessaire. `files`, `system`, `desktop` et `install`
  retirent l'isolation par espace de noms utilisateur du bac à sable.
- **Le PC du store doit être allumé et joignable** au moment de la publication.

## Mise en place (une fois)

- Secret du dépôt `LAPP_STORE_TOKEN` : jeton du store (celui de `/etc/lapp-store.env`).
- Variable du dépôt `STORE_URL` : adresse publique du store, ex. `http://natalos.duckdns.org:10078`
  (le port doit être redirigé vers le PC).
- Sur le PC : `lapp-store.service` (serveur `server.py`, route `/stage/`), le binaire `lapp` et la clé `~/.lapp/dev.key`.
- Garder le dépôt **privé** et protéger `main` : qui peut pousser sur `main` peut publier sur le store.

## Note sur la sécurité

Le store est en HTTP simple. Le jeton n'y circule pas (HMAC), et le contenu envoyé n'a rien de secret
(binaire compilé), mais pour une exposition durable sur Internet il vaut mieux placer un HTTPS devant
(par exemple Caddy), puis changer `STORE_URL`.