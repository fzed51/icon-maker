use egui::{Color32, RichText, Ui};

use super::BIG_BUTTON;
use crate::app::state::AppState;
use crate::engine::SquareMode;

pub enum SettingsAction {
    None,
    OpenDialog,
    OpenBatch,
}

/// Panneau latéral : peu de choix, avec des mots simples.
pub fn show(ui: &mut Ui, state: &mut AppState, show_more: &mut bool) -> SettingsAction {
    let mut action = SettingsAction::None;
    let has_image = state.source.is_some();

    ui.add_space(8.0);
    if ui
        .add(egui::Button::new("Changer d'image").min_size(BIG_BUTTON))
        .on_hover_text("Choisir une autre photo ou un autre dessin")
        .clicked()
    {
        action = SettingsAction::OpenDialog;
    }
    ui.separator();

    ui.add_enabled_ui(has_image, |ui| {
        // --- Fond ---
        let bg = &mut state.settings.background;
        let label = if bg.enabled { "✔  Fond effacé" } else { "Effacer le fond" };
        if ui
            .add(egui::Button::new(RichText::new(label).size(18.0)).selected(bg.enabled).min_size(BIG_BUTTON))
            .on_hover_text("Rend le fond transparent, comme sur le damier")
            .clicked()
        {
            bg.enabled = !bg.enabled;
        }
        if bg.enabled {
            ui.label("👆 Clique sur le fond de l'image pour choisir la couleur à effacer.");
            ui.horizontal(|ui| {
                ui.label("Couleur du fond :");
                match bg.key {
                    Some(c) => {
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 20.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 4.0, Color32::from_rgb(c[0], c[1], c[2]));
                        ui.painter().rect_stroke(rect, 4.0, (1.0, Color32::GRAY), egui::StrokeKind::Inside);
                    }
                    None => {
                        ui.label(RichText::new("automatique").italics());
                    }
                }
            });
            ui.label("Force de l'effacement :");
            ui.horizontal(|ui| {
                ui.label("moins");
                ui.add(egui::Slider::new(&mut bg.tolerance, 0.0..=60.0).show_value(false))
                    .on_hover_text("Plus fort : efface aussi les couleurs proches du fond");
                ui.label("plus");
            });
        }
        ui.separator();

        // --- Forme ---
        let is_square = state
            .preview_source
            .as_ref()
            .is_some_and(|img| img.width() == img.height());
        if has_image && is_square {
            ui.label("Ton image est déjà carrée 👍");
        } else {
            ui.label(RichText::new("Ton image n'est pas carrée :").strong());
            let pad = state.settings.square == SquareMode::Pad;
            if ui
                .add(egui::Button::new("▣  Tout garder").selected(pad).min_size(BIG_BUTTON))
                .on_hover_text("Garde toute l'image et ajoute du vide transparent autour")
                .clicked()
            {
                state.settings.square = SquareMode::Pad;
            }
            if ui
                .add(egui::Button::new("✂  Couper en carré").selected(!pad).min_size(BIG_BUTTON))
                .on_hover_text("Coupe les bords ; fais glisser l'image pour choisir ce que tu gardes")
                .clicked()
                && pad
            {
                state.settings.square = SquareMode::Crop { offset: 0.5 };
            }
        }
        ui.separator();
    });

    // --- Plus d'options ---
    let header = egui::CollapsingHeader::new("Plus d'options").open(Some(*show_more)).show(ui, |ui| {
        ui.add_enabled_ui(state.settings.background.enabled, |ui| {
            ui.label("Bords adoucis :");
            ui.add(egui::Slider::new(&mut state.settings.background.feather, 0.0..=40.0).show_value(false))
                .on_hover_text("Rend le contour plus doux après l'effacement du fond");
        });
        ui.checkbox(&mut state.settings.export_png, "Enregistrer aussi les images PNG")
            .on_hover_text("Crée en plus une image PNG pour chaque taille");
        if ui.button("Transformer plusieurs images…").clicked() {
            action = SettingsAction::OpenBatch;
        }
    });
    if header.header_response.clicked() {
        *show_more = !*show_more;
    }
    action
}
