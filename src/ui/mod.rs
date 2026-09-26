//! Rendu egui. Toute la logique testable vit dans `app::state` et `engine`.

pub mod batch;
pub mod preview;
pub mod settings;
pub mod sizes_strip;
pub mod steps;

use egui::{Color32, ColorImage, Context, Painter, Rect, TextureHandle, TextureOptions, Vec2};
use image::RgbaImage;

/// Couleur des actions principales.
pub const ACCENT: Color32 = Color32::from_rgb(0, 120, 212);

/// Taille des gros boutons.
pub const BIG_BUTTON: Vec2 = Vec2::new(220.0, 44.0);

pub fn to_texture(
    ctx: &Context,
    name: &str,
    img: &RgbaImage,
    options: TextureOptions,
) -> TextureHandle {
    let size = [img.width() as usize, img.height() as usize];
    ctx.load_texture(
        name,
        ColorImage::from_rgba_unmultiplied(size, img.as_raw()),
        options,
    )
}

/// Damier gris clair : représente la transparence.
pub fn checkerboard(painter: &Painter, rect: Rect, cell: f32) {
    painter.rect_filled(rect, 0.0, Color32::from_gray(250));
    let cols = (rect.width() / cell).ceil() as i32;
    let rows = (rect.height() / cell).ceil() as i32;
    for row in 0..rows {
        for col in (row % 2..cols).step_by(2) {
            let min = rect.min + Vec2::new(col as f32 * cell, row as f32 * cell);
            let r = Rect::from_min_size(min, Vec2::splat(cell)).intersect(rect);
            painter.rect_filled(r, 0.0, Color32::from_gray(215));
        }
    }
}

/// Style plus lisible : textes plus grands, boutons plus aérés.
pub fn apply_style(ctx: &Context) {
    ctx.all_styles_mut(|style| {
        use egui::{FontId, TextStyle};
        style.text_styles = [
            (TextStyle::Heading, FontId::proportional(26.0)),
            (TextStyle::Body, FontId::proportional(17.0)),
            (TextStyle::Button, FontId::proportional(17.0)),
            (TextStyle::Small, FontId::proportional(13.0)),
            (TextStyle::Monospace, FontId::monospace(15.0)),
        ]
        .into();
        style.spacing.button_padding = Vec2::new(14.0, 8.0);
        style.spacing.item_spacing = Vec2::new(10.0, 10.0);
        style.spacing.slider_width = 180.0;
    });
}
