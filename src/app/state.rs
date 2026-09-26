//! État de l'application et règles de l'interface, sans aucun rendu : testable seul.

use std::path::{Path, PathBuf};

use anyhow::Result;
use image::imageops::{self, FilterType};
use image::{Rgb, RgbaImage};

use crate::engine::{Settings, load, pipeline, square};

/// Côté maximal de la copie de travail utilisée pour l'aperçu.
pub const PREVIEW_MAX: u32 = 1024;

/// Les 3 étapes affichées en haut de la fenêtre.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    ChooseImage = 1,
    Arrange = 2,
    Saved = 3,
}

#[derive(Default)]
pub struct AppState {
    pub source_path: Option<PathBuf>,
    /// Image d'origine en pleine résolution (utilisée pour l'enregistrement).
    pub source: Option<RgbaImage>,
    /// Copie réduite pour un aperçu fluide.
    pub preview_source: Option<RgbaImage>,
    pub settings: Settings,
    pub last_saved: Option<PathBuf>,
    /// Message d'erreur en langage courant, affiché à l'utilisateur.
    pub error: Option<String>,
}

impl AppState {
    pub fn step(&self) -> Step {
        match (&self.source, &self.last_saved) {
            (None, _) => Step::ChooseImage,
            (Some(_), None) => Step::Arrange,
            (Some(_), Some(_)) => Step::Saved,
        }
    }

    /// Ouvre une image. En cas d'échec, l'image courante est conservée et `error` est rempli.
    pub fn open(&mut self, path: &Path) {
        match load::load_image(path) {
            Ok(img) => {
                self.preview_source = Some(preview_copy(&img));
                self.source = Some(img);
                self.source_path = Some(path.to_path_buf());
                self.settings.background.key = None;
                self.last_saved = None;
                self.error = None;
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    /// Clic dans l'aperçu, en coordonnées normalisées (0..1) du carré affiché.
    /// Prend la couleur de l'image d'origine à cet endroit comme couleur du fond.
    /// Renvoie `false` si le clic tombe hors de l'image (marge transparente).
    pub fn pick_background(&mut self, u: f32, v: f32) -> bool {
        let Some(preview) = &self.preview_source else {
            return false;
        };
        let shown = square::make_square(preview, self.settings.square);
        let side = shown.width();
        let to_px = |t: f32| ((t * side as f32) as u32).min(side - 1);
        let p = shown.get_pixel(to_px(u), to_px(v));
        if p[3] == 0 {
            return false;
        }
        self.settings.background.key = Some(Rgb([p[0], p[1], p[2]]));
        self.settings.background.enabled = true;
        true
    }

    /// Nom proposé à l'enregistrement : même dossier et même nom que l'image, en `.ico`.
    pub fn suggested_save_path(&self) -> Option<PathBuf> {
        let src = self.source_path.as_ref()?;
        let stem = src.file_stem()?.to_string_lossy();
        Some(src.with_file_name(format!("{stem}.ico")))
    }

    pub fn save(&mut self, ico_path: &Path) -> Result<()> {
        let Some(source) = &self.source else {
            anyhow::bail!("Choisis d'abord une image.");
        };
        let images = pipeline::process(source, &self.settings);
        pipeline::export(&images, ico_path, self.settings.export_png)?;
        self.last_saved = Some(ico_path.to_path_buf());
        Ok(())
    }
}

/// Convertit une position écran en coordonnées normalisées dans le rectangle de l'aperçu.
/// Renvoie `None` hors du rectangle.
pub fn screen_to_uv(
    rect_min: (f32, f32),
    rect_size: (f32, f32),
    pos: (f32, f32),
) -> Option<(f32, f32)> {
    let u = (pos.0 - rect_min.0) / rect_size.0;
    let v = (pos.1 - rect_min.1) / rect_size.1;
    ((0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v)).then_some((u, v))
}

/// Réduit l'image pour l'aperçu si elle dépasse [`PREVIEW_MAX`].
pub fn preview_copy(img: &RgbaImage) -> RgbaImage {
    let (w, h) = img.dimensions();
    let longest = w.max(h);
    if longest <= PREVIEW_MAX {
        return img.clone();
    }
    let scale = |d: u32| ((d as u64 * PREVIEW_MAX as u64) / longest as u64).max(1) as u32;
    imageops::resize(img, scale(w), scale(h), FilterType::Triangle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::SquareMode;
    use image::Rgba;

    /// 100×50 : moitié gauche verte, moitié droite jaune. Enregistrée dans un dossier temporaire.
    fn saved_image(dir: &Path) -> PathBuf {
        let img = RgbaImage::from_fn(100, 50, |x, _| {
            if x < 50 {
                Rgba([0, 200, 0, 255])
            } else {
                Rgba([250, 250, 0, 255])
            }
        });
        let path = dir.join("photo.png");
        img.save(&path).unwrap();
        path
    }

    #[test]
    fn step_goes_from_choose_to_arrange_when_image_loaded() {
        let dir = tempfile::tempdir().unwrap();
        let mut st = AppState::default();
        assert_eq!(st.step(), Step::ChooseImage);
        st.open(&saved_image(dir.path()));
        assert_eq!(st.step(), Step::Arrange);
        assert!(st.error.is_none());
    }

    #[test]
    fn step_is_saved_after_saving_then_back_to_arrange_on_change() {
        let dir = tempfile::tempdir().unwrap();
        let mut st = AppState::default();
        st.open(&saved_image(dir.path()));
        st.save(&dir.path().join("out.ico")).unwrap();
        assert_eq!(st.step(), Step::Saved);
        assert!(dir.path().join("out.ico").exists());
        st.open(&saved_image(dir.path()));
        assert_eq!(st.step(), Step::Arrange);
    }

    #[test]
    fn opening_a_bad_file_keeps_current_image_and_shows_friendly_error() {
        let dir = tempfile::tempdir().unwrap();
        let bad = dir.path().join("casse.jpg");
        std::fs::write(&bad, b"rien").unwrap();
        let mut st = AppState::default();
        st.open(&saved_image(dir.path()));
        st.open(&bad);
        assert!(st.source.is_some());
        let msg = st.error.clone().expect("un message d'erreur");
        assert!(msg.contains("image"), "message peu clair : {msg}");
        assert_eq!(st.step(), Step::Arrange);
    }

    #[test]
    fn opening_a_new_image_resets_picked_color() {
        let dir = tempfile::tempdir().unwrap();
        let mut st = AppState::default();
        st.settings.background.key = Some(Rgb([1, 2, 3]));
        st.open(&saved_image(dir.path()));
        assert_eq!(st.settings.background.key, None);
    }

    #[test]
    fn click_on_preview_picks_color_and_enables_background_removal() {
        let dir = tempfile::tempdir().unwrap();
        let mut st = AppState::default();
        st.open(&saved_image(dir.path()));
        // Marge : l'image 100×50 occupe la bande verticale 0.25..0.75 du carré affiché.
        assert!(st.pick_background(0.9, 0.5));
        assert_eq!(st.settings.background.key, Some(Rgb([250, 250, 0])));
        assert!(st.settings.background.enabled);
        assert!(st.pick_background(0.1, 0.5));
        assert_eq!(st.settings.background.key, Some(Rgb([0, 200, 0])));
    }

    #[test]
    fn click_in_transparent_margin_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let mut st = AppState::default();
        st.open(&saved_image(dir.path()));
        assert!(!st.pick_background(0.5, 0.05));
        assert_eq!(st.settings.background.key, None);
    }

    #[test]
    fn click_follows_crop_position() {
        let dir = tempfile::tempdir().unwrap();
        let mut st = AppState::default();
        st.open(&saved_image(dir.path()));
        st.settings.square = SquareMode::Crop { offset: 1.0 }; // on voit la moitié jaune
        assert!(st.pick_background(0.1, 0.5));
        assert_eq!(st.settings.background.key, Some(Rgb([250, 250, 0])));
    }

    #[test]
    fn suggested_name_is_image_name_with_ico() {
        let dir = tempfile::tempdir().unwrap();
        let mut st = AppState::default();
        assert_eq!(st.suggested_save_path(), None);
        let src = saved_image(dir.path());
        st.open(&src);
        assert_eq!(st.suggested_save_path(), Some(dir.path().join("photo.ico")));
    }

    #[test]
    fn screen_to_uv_converts_and_rejects_outside() {
        assert_eq!(
            screen_to_uv((100.0, 50.0), (200.0, 200.0), (200.0, 100.0)),
            Some((0.5, 0.25))
        );
        assert_eq!(
            screen_to_uv((100.0, 50.0), (200.0, 200.0), (99.0, 100.0)),
            None
        );
        assert_eq!(
            screen_to_uv((100.0, 50.0), (200.0, 200.0), (200.0, 251.0)),
            None
        );
    }

    #[test]
    fn preview_copy_limits_size_and_keeps_ratio() {
        let big = RgbaImage::new(3000, 1500);
        assert_eq!(preview_copy(&big).dimensions(), (1024, 512));
        let small = RgbaImage::new(300, 200);
        assert_eq!(preview_copy(&small).dimensions(), (300, 200));
    }
}
