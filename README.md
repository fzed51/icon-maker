# Icon Maker

Transforme une photo ou un dessin (JPEG ou PNG) en icône Windows `.ico`, en trois étapes simples. L'interface est en français et pensée pour être utilisable par un jeune public : gros boutons, pas de jargon.

## Fonctionnalités

- **Icône complète** : chaque `.ico` contient les tailles 16, 24, 32, 48, 64, 128 et 256 px.
- **Effacer le fond** : un clic sur le fond de l'image le rend transparent. L'effacement part des bords, donc une zone de même couleur à l'intérieur du sujet est conservée. Deux réglages : la force de l'effacement et l'adoucissement des bords.
- **Image non carrée** : tout garder (marge transparente ajoutée) ou couper en carré en faisant glisser l'image pour choisir la partie à garder.
- **Aperçu en direct** de l'icône à toutes les tailles, sur un damier qui montre la transparence.
- **Glisser-déposer** : une image l'ouvre, plusieurs lancent le traitement par lot.
- **Traitement par lot** : transforme plusieurs images d'un coup avec les mêmes réglages.
- **Export PNG** optionnel : une image PNG par taille, à côté du `.ico`.

## Utilisation

1. **Choisis ton image** : bouton « Choisir une image », ou glisse-la dans la fenêtre.
2. **Arrange-la** : efface le fond, choisis comment la rendre carrée.
3. **Enregistre l'icône** : « Enregistrer mon icône ».

On peut aussi déposer une image directement sur `icon-maker.exe`, ou l'utiliser avec « Ouvrir avec ».

## Compiler

Il faut [Rust](https://rustup.rs/) (édition 2024).

```sh
cargo build --release
```

L'exécutable se trouve dans `target/release/icon-maker.exe`. Il est autonome : un seul fichier, sans installation ni redistribuable Visual C++.

## Développement

```sh
cargo run                    # lance l'application
cargo test                   # tests unitaires et de bout en bout
cargo run --example convert -- entree.png sortie.ico   # conversion sans interface
```

Le code est organisé en trois parties :

- `src/engine/` : le traitement d'image (détourage, mise au carré, redimensionnement, écriture du `.ico`), sans interface ;
- `src/app/` : l'état de l'application, testable sans affichage ;
- `src/ui/` : l'affichage avec [egui](https://github.com/emilk/egui).

## Historique

Les changements de chaque version sont listés dans [CHANGELOG.md](CHANGELOG.md).
