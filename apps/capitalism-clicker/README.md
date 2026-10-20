# Capitalism Supremacy Clicker
Game made by Ziyad.K for the phone with Alexis.R

## Installation (recommandé)
    ~/lapp/sdk/tools/lapp-new com.ziyad.capitalismclicker --name "Capitalism Supremacy Clicker"
    # remplacer src/main.rs et assets/ par ceux de ce zip (garder le manifest.toml généré)
    cargo run -- --screenshot out.png
    ~/lapp/sdk/tools/lapp-build

## Notes
- Code écrit d'après le README du SDK, NON compilé : les signatures incertaines
  (Rect::new, g.rect/rrect/circle, Scroller::new, Store...) sont isolées dans les
  "Adaptateurs SDK" en haut de src/main.rs.
- Bouton Menu en jeu : Code, Save (sauvegarde la partie), Menu principal
- Code : clavier AZERTY a l'ecran (on appuie sur les touches), puis Valider
- Codes : elon, stonks, freedom, random (image au hasard 5 s), stalin / lenin (drapeau URSS 5 s, -50% d'argent),
  cheat ("uncle sam is desapointed", -100% d'argent)
- Ameliorations ajoutees : Scam, Trading (les anciennes sauvegardes sont converties automatiquement)
- "tax time" : toutes les 10 secondes, -5% de l'argent
- Images : assets/rand0..6.png (code random) et assets/ussr.png (drapeau, aussi dans random)
- Sauvegardes : /data/saves.conf (une entree par partie)
