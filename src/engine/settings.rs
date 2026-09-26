use image::Rgb;

/// Tailles contenues dans chaque icône, de la plus petite à la plus grande.
pub const ICON_SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];

/// Que faire d'une image qui n'est pas carrée.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SquareMode {
    /// Tout garder : l'image est centrée sur un fond transparent.
    Pad,
    /// Couper en carré. `offset` va de 0.0 (début) à 1.0 (fin) le long du grand côté.
    Crop { offset: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackgroundSettings {
    pub enabled: bool,
    /// Couleur du fond ; `None` = moyenne des 4 coins.
    pub key: Option<Rgb<u8>>,
    /// Écart de couleur toléré, de 0 à 100.
    pub tolerance: f32,
    /// Largeur de la transition douce au-delà de la tolérance, de 0 à 100.
    pub feather: f32,
}

impl Default for BackgroundSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            key: None,
            tolerance: 15.0,
            feather: 10.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub background: BackgroundSettings,
    pub square: SquareMode,
    pub export_png: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            background: BackgroundSettings::default(),
            square: SquareMode::Pad,
            export_png: false,
        }
    }
}
