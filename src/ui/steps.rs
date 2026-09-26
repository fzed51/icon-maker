use egui::{Color32, RichText, Ui};

use super::ACCENT;
use crate::app::state::Step;

const LABELS: [(Step, &str); 3] = [
    (Step::ChooseImage, "1. Choisis ton image"),
    (Step::Arrange, "2. Arrange-la"),
    (Step::Saved, "3. Enregistre l'icône"),
];

/// Barre des 3 étapes ; l'étape en cours est surlignée.
pub fn show(ui: &mut Ui, current: Step) {
    ui.horizontal(|ui| {
        for (i, (step, label)) in LABELS.iter().enumerate() {
            if i > 0 {
                ui.label(RichText::new("›").size(22.0).color(Color32::GRAY));
            }
            let text = RichText::new(*label).size(19.0);
            if *step == current {
                egui::Frame::new()
                    .fill(ACCENT)
                    .corner_radius(8.0)
                    .inner_margin(egui::Margin::symmetric(12, 6))
                    .show(ui, |ui| ui.label(text.strong().color(Color32::WHITE)));
            } else if (*step as u8) < (current as u8) {
                ui.label(text.color(ACCENT));
            } else {
                ui.label(text.color(Color32::GRAY));
            }
        }
    });
}
