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
- Bouton Menu en jeu : Save (sauvegarde la partie), codes (elon, stonks, freedom), Menu principal
- Sauvegardes : /data/saves.conf (une entrée par partie)
