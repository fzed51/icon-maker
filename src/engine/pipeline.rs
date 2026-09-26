use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use image::RgbaImage;

use super::{ICON_SIZES, Settings, background, ico, load, resize, square};

/// Détourage éventuel puis mise au carré : l'image « maîtresse » avant redimensionnement.
pub fn prepare(img: &RgbaImage, settings: &Settings) -> RgbaImage {
    let bg = &settings.background;
    if bg.enabled {
        let mut img = img.clone();
        let key = bg.key.unwrap_or_else(|| background::corner_color(&img));
        background::remove_background(&mut img, key, bg.tolerance, bg.feather);
        square::make_square(&img, settings.square)
    } else {
        square::make_square(img, settings.square)
    }
}

/// Chaîne complète : une image par taille d'icône.
pub fn process(img: &RgbaImage, settings: &Settings) -> Vec<RgbaImage> {
    resize::resize_all(&prepare(img, settings))
}

/// Chemins des PNG exportés à côté de `ico_path` : `nom_16.png` … `nom_256.png`.
pub fn png_paths(ico_path: &Path) -> Vec<PathBuf> {
    let stem = ico_path.file_stem().unwrap_or_default().to_string_lossy();
    ICON_SIZES
        .iter()
        .map(|s| ico_path.with_file_name(format!("{stem}_{s}.png")))
        .collect()
}

/// Écrit le .ico et, si demandé, les PNG de chaque taille.
pub fn export(images: &[RgbaImage], ico_path: &Path, export_png: bool) -> Result<()> {
    ico::write_ico(ico_path, images)?;
    if export_png {
        for (img, path) in images.iter().zip(png_paths(ico_path)) {
            img.save(&path)
                .context("Les images PNG n'ont pas pu être enregistrées.")?;
        }
    }
    Ok(())
}

/// Charge `input`, le traite et l'enregistre sous `ico_path`.
pub fn convert_file(input: &Path, ico_path: &Path, settings: &Settings) -> Result<()> {
    let img = load::load_image(input)?;
    export(&process(&img, settings), ico_path, settings.export_png)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{ICON_SIZES, SquareMode};
    use image::Rgba;

    #[test]
    fn prepare_without_background_only_squares() {
        let img = RgbaImage::from_pixel(40, 20, Rgba([255, 255, 255, 255]));
        let out = prepare(&img, &Settings::default());
        assert_eq!(out.dimensions(), (40, 40));
        assert_eq!(out.get_pixel(20, 20), &Rgba([255, 255, 255, 255]));
    }

    #[test]
    fn prepare_with_crop_uses_offset() {
        let img = RgbaImage::from_fn(40, 20, |x, _| {
            if x < 20 {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([0, 0, 255, 255])
            }
        });
        let s = Settings {
            square: SquareMode::Crop { offset: 1.0 },
            ..Default::default()
        };
        let out = prepare(&img, &s);
        assert_eq!(out.dimensions(), (20, 20));
        assert_eq!(out.get_pixel(0, 0), &Rgba([0, 0, 255, 255]));
    }

    #[test]
    fn process_returns_all_sizes() {
        let img = RgbaImage::from_pixel(50, 30, Rgba([1, 2, 3, 255]));
        assert_eq!(process(&img, &Settings::default()).len(), ICON_SIZES.len());
    }

    #[test]
    fn png_paths_are_named_after_icon() {
        let paths = png_paths(Path::new("C:/out/logo.ico"));
        assert_eq!(paths.len(), 7);
        assert_eq!(paths[0], Path::new("C:/out/logo_16.png"));
        assert_eq!(paths[6], Path::new("C:/out/logo_256.png"));
    }
}
