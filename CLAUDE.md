# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Projet

Icon Maker : application de bureau Windows (Rust + egui/eframe) qui transforme une image JPEG/PNG en icône `.ico` (tailles fixes 16/24/32/48/64/128/256). Public visé : jeune utilisateur → UI en 3 étapes, gros boutons, textes en français sans jargon. Détourage du fond **par couleur uniquement** (pas d'IA).

## Commandes

```sh
cargo build --release          # exe unique autonome (CRT statique via .cargo/config.toml, LTO, strip)
cargo run                      # lance l'appli (console visible en debug seulement)
cargo run -- image.png         # ouvre directement une image (comme « Ouvrir avec »)
cargo test                     # tests unitaires (dans chaque module) + tests/end_to_end.rs
cargo test prepare_with_crop   # un seul test, par nom (filtre sur sous-chaîne)
cargo test --test end_to_end   # uniquement les tests d'intégration
cargo clippy --all-targets -- -D warnings   # comme la CI : aucun avertissement toléré
cargo fmt --check              # vérifié par la CI (cargo fmt pour corriger)
cargo run --example convert -- entree.png sortie.ico   # conversion sans UI (sert à régénérer assets/app.ico)
```

Méthode de travail : TDD — écrire le test d'abord et le voir échouer avant d'implémenter.

## Architecture

Séparation stricte en trois couches (`src/lib.rs` expose les trois ; `src/main.rs` ne fait que configurer la fenêtre) :

- **`engine/`** — logique pure sur `image::RgbaImage`, sans egui. Point d'entrée : `pipeline.rs`
  - `prepare()` = détourage optionnel (`background.rs`, diffusion depuis les bords à partir d'une couleur clé ; clé `None` = moyenne des 4 coins) puis mise au carré (`square.rs`, `Pad` ou `Crop { offset }`).
  - `process()` = `prepare()` + `resize::resize_all()` → une image par taille de `ICON_SIZES`.
  - `export()` / `convert_file()` écrivent le `.ico` (`ico.rs`, frames encodées en PNG) et éventuellement `nom_<taille>.png`.
  - `batch.rs` : traitement de lot dans un thread, progression via `mpsc::Sender<BatchEvent>`. `batch_settings()` force clé auto par image et recadrage centré.
  - `settings.rs` : `Settings` est `Copy + PartialEq`, ce qui sert à détecter les changements.
- **`app/`** — `state.rs` (`AppState`) contient l'état et les règles d'interface **sans rendu**, donc testable ; `Step` est déduit de l'état (`source` / `last_saved`). `mod.rs` (`IconMakerApp`) relie l'état à egui : dialogues `rfd`, glisser-déposer (1 fichier → ouverture, plusieurs → fenêtre de lot), et recalcul de l'aperçu avec anti-rebond de 150 ms en comparant `rendered: Option<Settings>` aux réglages courants.
- **`ui/`** — rendu egui uniquement (pas de logique testable ici). Constantes de style partagées dans `ui/mod.rs` (`ACCENT`, `BIG_BUTTON`, `apply_style`).

Points à connaître :
- L'aperçu travaille sur `preview_source` (copie réduite à `PREVIEW_MAX` = 1024 px) ; l'enregistrement repart de `source` en pleine résolution.
- La pipette (`AppState::pick_background`) reçoit des coordonnées normalisées du carré affiché et relit le pixel via `square::make_square` sur l'image avant détourage : tout changement de la mise au carré doit rester cohérent entre l'aperçu et la pipette. Un clic dans la marge transparente est ignoré ; un clic valide active le détourage.
- Tout réglage modifié après un enregistrement remet `last_saved` à `None` (retour à l'étape 2).
- Les messages d'erreur (`anyhow` `.context(...)`) sont affichés tels quels à l'utilisateur : les rédiger en français courant.
- `build.rs` intègre `assets/app.ico` dans l'exe (Windows) ; `main.rs` l'utilise aussi comme icône de fenêtre via `include_bytes!`.
- `profile.dev.package."*"` en `opt-level = 2` : les dépendances image restent rapides en debug.
