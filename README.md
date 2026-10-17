# lapp-apps

Dépôt des applications `.lapp`. **Un push sur `main` compile les apps modifiées et les publie
automatiquement sur le store** : plus de compilation manuelle.

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

Le workflow compile pour `aarch64`, signe le paquet et l'envoie au store. L'app apparaît dans l'app Store
de l'appareil.

## Règles

- **Chaque publication = une nouvelle version.** Le store n'écrase jamais un paquet : si
  `<id>-<version>.lapp` existe déjà, l'app est ignorée avec un message. Incrémentez `version` dans
  `manifest.toml` (et `Cargo.toml`) pour publier.
- **Seules les apps modifiées sont reconstruites.** Si `lapp-sdk/` ou `scripts/` change, toutes les apps le sont.
- **Pull request** : l'app est compilée pour vérifier qu'elle passe, sans rien publier. Les PR venant de forks
  ne sont jamais construites (le runner est un vrai PC).
- **Relancer à la main** : onglet *Actions* → *Compiler et publier les apps* → *Run workflow*
  (dossier d'une app, ou vide pour toutes).
- **Permissions du manifeste** : ne demandez que le nécessaire. `files`, `system`, `desktop` et `install`
  retirent l'isolation par espace de noms utilisateur du bac à sable.

## Mise en place (une fois)

- Secret du dépôt `LAPP_STORE_TOKEN` : jeton de publication du store.
- Variable facultative `STORE_URL` (défaut `http://127.0.0.1:10079`).
- Un runner GitHub auto-hébergé étiqueté `lapp` sur le PC du store (`~/actions-runner`, service systemd),
  avec Rust + cible `aarch64-unknown-linux-gnu`, `gcc-aarch64-linux-gnu`, le binaire `~/lapp/target/release/lapp`
  et la clé de signature `~/.lapp/dev.key`. La clé reste sur le PC : ce n'est pas un secret GitHub.
- Le dépôt doit rester **privé** et `main` protégé : le code poussé s'exécute sur ce PC.