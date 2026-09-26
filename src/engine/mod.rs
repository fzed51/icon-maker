//! Logique de traitement pure, indépendante de l'interface.

pub mod background;
pub mod batch;
pub mod ico;
pub mod load;
pub mod pipeline;
pub mod resize;
pub mod settings;
pub mod square;

pub use settings::{BackgroundSettings, ICON_SIZES, Settings, SquareMode};
