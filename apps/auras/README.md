# Auras (lapp)

Jeu de tirage d'auras pour lapp-sdk : 54 auras de 14 types, de Poussière (1 sur 2) à Infini
(1 sur 1 000 000 000). Chaque appui sur « Invoquer » donne une aura ; tous les 10 lancers la
chance est doublée. L'aura équipée donne un bonus sur les auras de son type (ex. Sakura : ×2 sur
les auras Plante ; Glitch, Créateur et Infini boostent toutes les auras). Chaque nouvelle aura a
sa cinématique, et l'Index permet d'équiper une aura ou de revoir son animation.

Progression enregistrée dans /data/auras-v2.conf.

## Installation

Décompressez ce dossier à côté du SDK, pour obtenir `~/lapp/auras` et `~/lapp/sdk`
(sinon, corrigez le chemin `lapp-sdk = { path = "../sdk" }` dans Cargo.toml).

    cd ~/lapp/auras
    cargo run -- --screenshot out.png     # aperçu hors écran
    ~/lapp/sdk/tools/lapp-build           # produit com.alexi.auras-1.4.0.lapp
    sudo ~/lapp/target/release/lapp install com.alexi.auras-1.4.0.lapp

Captures de test : `--equip galaxie`, `--page index [--detail 17]`,
`--cs infini --t 6`.
