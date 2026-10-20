# Conquis Cliqueur v16 (lapp)

Jeu cliqueur infini.

- Échap (ou le bouton ≡) : menu Reprendre / Entrer un code / Quitter.
- Code **BUG** : vidéo en plein écran, impossible de la fermer avant la fin.
- Code **TUNE** : ouvre l'interface secrète, qui demande un 2e code :
  - **INFINI** : château en or, clics et production ×4 pour toujours ;
  - **DIAMEN** : château en diamant, ×1000 pour toujours ;
  - **NETHERITE** : château noir, ×10000 pour toujours ;
  - **MAX** : demande un 3e code. **GALAXI** : château violet avec une galaxie animée, ×100000 pour toujours ; au 3e code, **MAXGALAXI** (après GALAXI) passe à ×1000000 (galaxie MAX, anneau doré), et **TROU NOIR** donne le château trou noir, ×10000000 (disque d'accrétion, anneau de photons, anneau de lumière, clics aspirés). **ALIEN** donne le château alien, ×100000000 (soucoupe avec son petit alien, faisceau tracteur, radar, anneaux en pointillés, clics en carrés verts qui montent avec un rayon). Un code secret au 3e code ouvre un **4e code**, lui aussi secret : il donne le **château Éclipse** (×100 milliards, deux orbes d'énergie rouge et bleue en orbite, éclairs, entrée en scène animée, cliqueurs rouges et bleus qui frappent ×8). Ces deux codes ne sont écrits nulle part : le programme ne contient que des empreintes PBKDF2-SHA256 à 1 million de tours (`K3`, `K4`), avec un sel propre à chacun, et l'empreinte du 4e code est liée au 3e (il faut avoir tapé le bon 3e code pour que le 4e fonctionne). Chaque essai demande ≈ 2 s de calcul (dans un thread, l'écran ne gèle pas), ce qui rend la recherche par force brute très lente. Le clavier a les touches Espace et « ? ».
  Un château moins puissant ne remplace jamais un plus puissant.
- Onglet **Châteaux** : achète le **château en or** (×4, 1M) et le **château en diamant** (×1000, 100B), puis 8 châteaux avec tes territoires ; chacun donne un multiplicateur en plus de tous les autres (grade, château des codes, magie).
- Onglet **Cliqueurs** : des souris d'ordinateur tournent autour du château et cliquent dessus. Chaque clic compte comme un vrai clic (tous les multiplicateurs). Achète des souris en plus ou augmente leur vitesse de clic.
- **Animation de clic** de plus en plus stylée selon le château : pierre, or, diamant, netherite, galaxie, galaxie MAX.
- **Château Solaire** (3e niveau de code) : ×10 milliards ; soleil animé (couronne de rayons, granulation, taches, protubérances), fond de tempête solaire, étincelles de clic, boss exclusif (Seigneur solaire) avec sa propre apparition, animation de déblocage (allumage du soleil). Capacité **Éclat du soleil** : multiplicateur du château ×1 million pendant 3 min (recharge 10 min), avec son animation.
- **Onglet Défense** : achète des gardes (8 max) qui défendent le château contre des vagues de monstres (toutes les 75 s après le premier garde). Améliorations : force (dégâts ×2), cadence de tir, rempart (les monstres volent moins). Un monstre qui atteint le château vole des territoires ; chaque monstre vaincu en rapporte. Tu peux aussi toucher les monstres. Chaque château a son garde (chevalier de pierre, paladin doré, … prêtre solaire) ; pour l'Éclipse ultime, le **Maître de l'Infini** (design original) lance des orbes violettes qui explosent sur les monstres proches.
- **Château Éclipse ultime** (4e code secret, bis) : ×10 000 milliards ; tout ce qu'a le château Éclipse (orbes, cliqueurs ×8, musique, Extension du territoire, Orbe violette, thème) **plus l'Œil absolu** : butin des boss ×5, sorts gratuits (aucun mana), recharges des capacités ÷10, Extension du territoire ×10 milliards (au lieu de ×1 milliard). Ouverture de l'œil animée au déblocage ; l'œil cligne sur le château.
- **Code caché** (1er niveau, vérifié par empreinte) : en mode Éclipse (Éclipse ou Éclipse ultime), l'Extension du territoire reste active à l'infini (∞) ; le même code la désactive.
- **Apparition des boss** : chaque boss arrive avec sa propre animation plein écran (le sol qui se fissure pour le golem, pluie de lumière et de pièces pour le gardien doré, éclats de cristal, faille de lave, traversée des étoiles pour les titans, trou noir qui aspire l'écran, faisceau tracteur pour l'envahisseur, entailles rouges et quatre yeux dans le noir pour le roi des malédictions, cadran qui remonte le temps) ; le boss surgit à 1,7 s.
- **Orbe violette** (animation de 4,4 s) : bandes de cinéma, énergie aspirée vers l'orbe, anneaux d'étincelles, rayons qui s'élargissent, lancement avec traînée en hélice, impact (ondes de choc, fissures, débris, flash), « ANÉANTI ».
- **Code TESTEBOSS** (menu › Entrer un code, 1er niveau) : fait apparaître un boss de test (avec son animation) qui ne donne **aucun butin**, ne compte ni pour les succès ni pour les quêtes, et ne décale pas le chrono du prochain vrai boss.
- **Boss** : toutes les 10 min un boss apparaît à côté du château (bannière « BOSS », vignette rouge qui pulse, apparition animée, explosion à la mort) ; un **chrono** « Prochain boss dans m:ss » s'affiche en bas à gauche de la zone du château. **Il faut le toucher lui** : les cliqueurs n'ont aucun effet. **Chaque château a son boss** (golem de roche, gardien doré, colosse de cristal, brute de netherite, titan stellaire, titan cosmique, dévoreur d'horizon, envahisseur, roi des malédictions pour Éclipse, gardien du temps) ; il prend l'aspect du château équipé quand il apparaît. À moins de 35 % de vie il s'enrage (aura rouge). Touche-le pour le tuer (20 PV, +2 par boss tué, max 60). Butin : 10 min de production. Avec le château **Éclipse**, les coups ne le blessent pas : utilise la capacité **Orbe violette** (une orbe violette se forme, part vers le boss et le détruit ; recharge 1 min). Bouton Orbe sous le boss, ou dans l'onglet Magie.
- **Menu › Progrès** : **Succès** (22, +2 % à tout chacun), **Quêtes du jour** (3 par jour, +2 min de production chacune, coffre = 20 min + 1 point de talent ; le jour change à minuit UTC) et **Talents** (6 talents ; +3 points à chaque Big Bang, +1 par coffre ; ils survivent au Big Bang).
- **Combos** de capacités (cachés, annoncés quand ils s'activent) : Singularité (Vide + Frénésie ×5), Horizon d'événements (Vide + Bénédiction ×3), Territoire parfait (Extension + Frénésie ×10), Domaine béni (Extension + Bénédiction ×5), Surcharge arcanique (Clone + Ascension ×4).
- Le **château en netherite** s'achète aussi (onglet Châteaux, 1e16).
- **Musique** du château Éclipse (celle de ta vidéo, `assets/music.m4a`) : le SDK n'a pas de son, le jeu essaie donc de lancer `mpv`, `ffplay` ou `cvlc` s'ils sont installés. Réglage dans Paramètres. La version HTML joue la musique directement.
- **Menu › Conquis** : liste des 10 châteaux ; touche un château débloqué pour l'équiper (plus ou moins fort). Un château se débloque par son code ou par achat ; s'il est moins fort que celui que tu portes, il est débloqué sans être équipé. Les anciennes sauvegardes gardent tous les châteaux jusqu'à celui qu'elles avaient.
- Code **TEMPS** (3e code) : château du Temps ×1 milliard (cadran, engrenages, aiguilles, sable, halo de repères, fond et boutons à lui). Capacité **Avancement temporel** : gagne d'un coup 5 min de production (clics des cliqueurs compris), recharge 10 min.
- Les châteaux **alien** et **temps** ont maintenant un fond et des boutons personnalisés, comme galaxie, galaxie MAX, trou noir et Éclipse.
- **Capacités** (onglet Magie, dernière ligne, selon le château actif) :
  - **Le Vide** (château trou noir) : multiplicateur du château ×100 pendant 3 min, recharge 8 min ;
  - **Extension du territoire** (château Éclipse) : tous les multiplicateurs ×1 milliard pendant 5 min, recharge 5 min.
  Chacune a une animation d'activation plein écran, et le temps restant s'affiche sous le score. Les temps sont sauvegardés (la capacité continue pendant que tu es hors-ligne, et le gain hors-ligne en tient compte).
- **Fond et boutons personnalisés** pour les châteaux galaxie, galaxie MAX, trou noir et Éclipse (fond animé, boutons, lignes, barre d'onglets).
- **16 châteaux** à acheter dans l'onglet Châteaux (8 nouveaux, de la Tour de cristal au Château du multivers).
- **Grades** : plus tu as conquis de territoires au total, plus ton grade est haut (Paysan -> Éternel, 12 grades)
  et plus tu as un multiplicateur en plus de celui du château.
- **Paramètres** (menu) : compteur de FPS activable / désactivable (FPS réels, non limités par l'écran, mesurés 0,3 s toutes les 2 s pour ne pas faire laguer), et **Test de puissance** (processeur puis graphismes, avec un score).
- **Le Big Bang** : tout en bas de la boutique, après la Dimension 52. Il remet tout à zéro (territoires, unités, magie, châteaux achetés, grade) mais double tous tes multiplicateurs pour toujours ; le château des codes est conservé. Il se confirme en appuyant deux fois, puis une animation de 4 s se joue (implosion, flash, explosion).
- Onglet **Magie** : sorts (Frénésie, Pluie d'or, Bénédiction, Clone magique, Distorsion temporelle, Ascension arcanique) et améliorations à débloquer (Réservoirs, Source arcanique, Cristal de mana, Pacte d'or, Sablier éternel, Main de mage, Grimoire ancien, Archimage).
  Le mana se remplit tout seul ; pour en avoir plus : Réservoir de mana I (+50 max),
  Réservoir de mana II (+100 max), Cristal de mana (+150 max), Source arcanique (mana 2× plus vite).

## Compiler et installer (sur ta machine, avec le SDK lapp déjà en place)

    cd conquis-cliqueur
    ../lapp-sdk/tools/lapp-build                 # produit com.moi.conquis-cliqueur-16.0.0.lapp
    sudo ~/lapp/target/release/lapp install com.moi.conquis-cliqueur-16.0.0.lapp

Prérequis : `rustup target add aarch64-unknown-linux-gnu`, `gcc-aarch64-linux-gnu`,
le binaire `lapp` et la clé `~/.lapp/dev.key` (voir le README du SDK).

## Tester sur PC

    LAPP_DATA=/tmp/d cargo run
    cargo run -- --screenshot out.png --page menu|code|tune|tune3|video|magic|castles|settings|bench|cursors|bigbang|intro|tune4|domainfx|voidfx|skins|timefx|progress [--tab 0|1|2]|orbfx|bossintro [--bskin N --bbt T]|eyefx|sunfx|solarfx|defense [--def N] [--sun] [--bbt 2] [--vt 8] [--skin 1|2|3|4|5|6|7|8|9] [--fps] [--burst] [--ability] [--boss] [--bskin N] [--progress] [--demo]

## Notes
- La vidéo est rejouée image par image (assets/frames, 6 images/s, 272 px de large) : le SDK n'a pas de lecteur vidéo ni de son.
- Le dossier `lapp-sdk/` est une copie du SDK ; le jeu l'utilise via `../lapp-sdk`.

## Modification du SDK (copie `lapp-sdk/` de ce zip)
Le SDK force la synchro verticale (EGL `SwapInterval::Wait(1)`), ce qui plafonne les FPS à la fréquence de l'écran.
J'ai ajouté `Ctx::unlimited_fps(bool)` (dans `lapp-sdk/src/runtime.rs`) : le jeu l'active seulement pendant les courtes mesures du compteur de FPS.
Ça ne change rien avec le rendu CPU (repli automatique), qui n'est pas synchronisé par le SDK.
