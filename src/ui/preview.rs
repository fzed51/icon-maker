use egui::{Align2, Color32, CursorIcon, FontId, Rect, Sense, TextureHandle, Ui, Vec2, pos2};

use super::{BIG_BUTTON, checkerboard};
use crate::app::state::{AppState, screen_to_uv};
use crate::engine::SquareMode;

pub enum PreviewAction {
    None,
    OpenDialog,
}

/// Zone centrale : invitation à déposer une image, ou aperçu interactif du résultat.
pub fn show(
    ui: &mut Ui,
    state: &mut AppState,
    texture: Option<&TextureHandle>,
    files_hovered: bool,
) -> PreviewAction {
    let mut action = PreviewAction::None;
    let avail = ui.available_rect_before_wrap();

    match texture {
        None => {
            ui.scope_builder(egui::UiBuilder::new().max_rect(avail), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(avail.height() * 0.3);
                    ui.label(
                        egui::RichText::new("Glisse ton image ici")
                            .size(30.0)
                            .strong(),
                    );
                    ui.label("(une photo ou un dessin en JPEG ou PNG)");
                    ui.add_space(12.0);
                    let btn =
                        egui::Button::new(egui::RichText::new("Choisir une image").size(20.0))
                            .min_size(BIG_BUTTON);
                    if ui.add(btn).clicked() {
                        action = PreviewAction::OpenDialog;
                    }
                });
            });
        }
        Some(tex) => {
            let hint_h = 28.0;
            let side = (avail.width().min(avail.height() - hint_h) - 20.0).max(64.0);
            let rect = Rect::from_center_size(
                pos2(avail.center().x, avail.min.y + 10.0 + side / 2.0),
                Vec2::splat(side),
            );
            let response = ui.allocate_rect(rect, Sense::click_and_drag());
            let painter = ui.painter_at(rect);
            checkerboard(&painter, rect, 16.0);
            painter.image(
                tex.id(),
                rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
            ui.painter().rect_stroke(
                rect,
                0.0,
                (1.0, Color32::from_gray(180)),
                egui::StrokeKind::Outside,
            );

            let bg_on = state.settings.background.enabled;
            let crop = matches!(state.settings.square, SquareMode::Crop { .. });

            // Clic : choisir la couleur du fond.
            if bg_on
                && response.clicked()
                && let Some(pos) = response.interact_pointer_pos()
                && let Some((u, v)) = screen_to_uv(rect.min.into(), rect.size().into(), pos.into())
            {
                state.pick_background(u, v);
            }
            // Glisser : déplacer la partie gardée.
            if crop && response.dragged() {
                drag_crop(state, response.drag_delta(), side);
            }
            if response.hovered() {
                let icon = match (crop, bg_on) {
                    (true, _) if response.dragged() => CursorIcon::Grabbing,
                    (true, false) => CursorIcon::Grab,
                    (_, true) => CursorIcon::Crosshair,
                    _ => CursorIcon::Default,
                };
                ui.ctx().set_cursor_icon(icon);
            }

            let hint = match (bg_on, crop) {
                (true, true) => {
                    "Clique sur le fond pour l'effacer · fais glisser pour choisir la partie à garder"
                }
                (true, false) => "Clique sur le fond de l'image pour l'effacer",
                (false, true) => "Fais glisser l'image pour choisir la partie à garder",
                (false, false) => "Le damier montre les parties transparentes",
            };
            ui.painter().text(
                pos2(rect.center().x, rect.max.y + 6.0),
                Align2::CENTER_TOP,
                hint,
                FontId::proportional(15.0),
                ui.visuals().weak_text_color(),
            );
        }
    }

    if files_hovered {
        let painter = ui.painter();
        painter.rect_filled(
            avail,
            12.0,
            Color32::from_rgba_unmultiplied(0, 120, 212, 60),
        );
        painter.text(
            avail.center(),
            Align2::CENTER_CENTER,
            "Lâche ton image ici",
            FontId::proportional(32.0),
            Color32::WHITE,
        );
    }
    action
}

/// Le contenu suit la souris : glisser vers la droite montre la partie gauche de l'image.
fn drag_crop(state: &mut AppState, delta: Vec2, shown_side: f32) {
    let Some(img) = &state.preview_source else {
        return;
    };
    let (w, h) = img.dimensions();
    let (long, short) = (w.max(h) as f32, w.min(h) as f32);
    if long <= short {
        return;
    }
    let along = if w > h { delta.x } else { delta.y };
    let moved_px = along / shown_side * short;
    if let SquareMode::Crop { offset } = &mut state.settings.square {
        *offset = (*offset - moved_px / (long - short)).clamp(0.0, 1.0);
    }
}
