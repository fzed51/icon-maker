use std::path::Path;

use anyhow::{Context, Result};
use image::codecs::ico::{IcoEncoder, IcoFrame};
use image::{ExtendedColorType, RgbaImage};

/// Assemble les images (carrées, ≤ 256 px) dans un conteneur ICO, chacune encodée en PNG.
pub fn encode_ico(images: &[RgbaImage]) -> Result<Vec<u8>> {
    let frames = images
        .iter()
        .map(|img| IcoFrame::as_png(img.as_raw(), img.width(), img.height(), ExtendedColorType::Rgba8))
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::new();
    IcoEncoder::new(&mut out).encode_images(&frames)?;
    Ok(out)
}

pub fn write_ico(path: &Path, images: &[RgbaImage]) -> Result<()> {
    let bytes = encode_ico(images)?;
    std::fs::write(path, bytes).context("L'icône n'a pas pu être enregistrée à cet endroit.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ICON_SIZES;
    use image::Rgba;

    fn frames() -> Vec<RgbaImage> {
        ICON_SIZES
            .iter()
            .map(|&s| RgbaImage::from_pixel(s, s, Rgba([10, 20, 30, 255])))
            .collect()
    }

    fn u16_at(b: &[u8], i: usize) -> u16 {
        u16::from_le_bytes([b[i], b[i + 1]])
    }

    #[test]
    fn header_is_valid_icon_with_seven_entries() {
        let bytes = encode_ico(&frames()).unwrap();
        assert_eq!(u16_at(&bytes, 0), 0, "réservé");
        assert_eq!(u16_at(&bytes, 2), 1, "type icône");
        assert_eq!(u16_at(&bytes, 4), 7, "nombre d'entrées");
    }

    #[test]
    fn entries_cover_sizes_16_to_256() {
        let bytes = encode_ico(&frames()).unwrap();
        let mut widths: Vec<u32> = (0..7)
            .map(|i| match bytes[6 + i * 16] {
                0 => 256,
                w => w as u32,
            })
            .collect();
        widths.sort();
        assert_eq!(widths, ICON_SIZES.to_vec());
    }

    #[test]
    fn file_is_readable_by_ico_decoder() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.ico");
        write_ico(&path, &frames()).unwrap();
        let img = image::open(&path).unwrap();
        assert_eq!((img.width(), img.height()), (256, 256));
    }
}
