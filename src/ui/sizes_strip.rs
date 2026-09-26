use egui::{Color32, Rect, RichText, Sense, TextureHandle, Ui, Vec2, pos2};

use super::checkerboard;
use crate::engine::ICON_SIZES;

/// Au-delà, l'aperçu est réduit pour ne pas envahir la fenêtre.
const MAX_SHOWN: f32 = 96.0;

/// Rangée « Ton icône » : chaque taille affichée à sa taille réelle (jusqu'à 96 px).
pub fn show(ui: &mut Ui, textures: &[TextureHandle]) {
    ui.horizontal_top(|ui| {
        ui.label(RichText::new("Ton icône :").strong());
        for (tex, size) in textures.iter().zip(ICON_SIZES) {
            ui.vertical(|ui| {
                let side = (size as f32).min(MAX_SHOWN);
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(side), Sense::hover());
                checkerboard(ui.painter(), rect, 4.0);
                ui.painter().image(tex.id(), rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                ui.label(RichText::new(format!("{size}")).small().weak());
            });
        }
    });
}
