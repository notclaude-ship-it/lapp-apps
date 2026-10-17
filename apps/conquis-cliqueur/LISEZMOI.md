# Conquis Cliqueur (lapp)

Jeu cliqueur infini.

- Échap (ou le bouton ≡) : menu Reprendre / Entrer un code / Quitter.
- Code **BUG** : vidéo en plein écran, impossible de la fermer avant la fin.
- Code **TUNE** : ouvre l'interface secrète, qui demande un 2e code :
  - **INFINI** : château en or, clics et production ×4 pour toujours ;
  - **DIAMEN** : château en diamant, clics et production ×1000 pour toujours.
- Onglet **Magie** : sorts (Frénésie, Pluie d'or, Bénédiction) et améliorations à débloquer.
  Le mana se remplit tout seul ; pour en avoir plus : Réservoir de mana I (+50 max),
  Réservoir de mana II (+100 max), Source arcanique (mana 2× plus vite).

## Compiler et installer (sur ta machine, avec le SDK lapp déjà en place)

    cd conquis-cliqueur
    ../lapp-sdk/tools/lapp-build                 # produit com.moi.conquis-cliqueur-1.2.0.lapp
    sudo ~/lapp/target/release/lapp install com.moi.conquis-cliqueur-1.2.0.lapp

Prérequis : `rustup target add aarch64-unknown-linux-gnu`, `gcc-aarch64-linux-gnu`,
le binaire `lapp` et la clé `~/.lapp/dev.key` (voir le README du SDK).

## Tester sur PC

    LAPP_DATA=/tmp/d cargo run
    cargo run -- --screenshot out.png --page menu|code|tune|video|magic [--vt 8] [--skin 1|2] [--demo]

## Notes
- La vidéo est rejouée image par image (assets/frames, 6 images/s, 272 px de large) : le SDK n'a pas de lecteur vidéo ni de son.
- Le dossier `lapp-sdk/` est une copie du SDK ; le jeu l'utilise via `../lapp-sdk`.
- Le son de la vidéo (code BUG) vient de `assets/video.pcm` (PCM 48 kHz stéréo, extrait de la vidéo d'origine), joué par `src/audio.rs` via PulseAudio/PipeWire (permission `audio`) ; l'image attend que le son soit prêt pour partir avec lui.
- La permission `files` est nécessaire au son : sans elle, `lapp` isole l'app dans un espace de noms utilisateur et `bwrap` ne peut pas monter `/run/user/<uid>/pipewire-0` quand `lapp` tourne en root (comme pour VLC).
