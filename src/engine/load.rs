use std::io::Cursor;
use std::path::Path;

use anyhow::{Context, Result};
use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageReader, RgbaImage};

const UNREADABLE: &str = "Cette image n'a pas pu être ouverte.";

/// Décode une image JPEG/PNG en RGBA en appliquant l'orientation EXIF.
pub fn decode_bytes(bytes: &[u8]) -> Result<RgbaImage> {
    let mut decoder = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .into_decoder()
        .context(UNREADABLE)?;
    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let mut img = DynamicImage::from_decoder(decoder).context(UNREADABLE)?;
    img.apply_orientation(orientation);
    Ok(img.to_rgba8())
}

/// Charge une image depuis le disque.
pub fn load_image(path: &Path) -> Result<RgbaImage> {
    let bytes = std::fs::read(path).context("Ce fichier n'a pas pu être lu.")?;
    decode_bytes(&bytes)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use image::{ImageFormat, Rgba};
    use std::io::Cursor;

    pub(crate) fn encode(img: &RgbaImage, format: ImageFormat) -> Vec<u8> {
        let mut out = Cursor::new(Vec::new());
        if format == ImageFormat::Jpeg {
            image::DynamicImage::ImageRgba8(img.clone())
                .to_rgb8()
                .write_to(&mut out, format)
                .unwrap();
        } else {
            img.write_to(&mut out, format).unwrap();
        }
        out.into_inner()
    }

    /// Insère un segment EXIF APP1 (orientation = `orientation`) juste après le SOI d'un JPEG.
    fn with_exif_orientation(jpeg: &[u8], orientation: u16) -> Vec<u8> {
        let mut tiff = Vec::new();
        tiff.extend_from_slice(b"II*\0");
        tiff.extend_from_slice(&8u32.to_le_bytes()); // offset du 1er IFD
        tiff.extend_from_slice(&1u16.to_le_bytes()); // 1 entrée
        tiff.extend_from_slice(&0x0112u16.to_le_bytes()); // tag Orientation
        tiff.extend_from_slice(&3u16.to_le_bytes()); // type SHORT
        tiff.extend_from_slice(&1u32.to_le_bytes()); // count
        tiff.extend_from_slice(&orientation.to_le_bytes());
        tiff.extend_from_slice(&[0, 0]); // padding de la valeur
        tiff.extend_from_slice(&0u32.to_le_bytes()); // pas d'IFD suivant

        let mut app1 = b"Exif\0\0".to_vec();
        app1.extend_from_slice(&tiff);

        let mut out = jpeg[..2].to_vec(); // SOI
        out.extend_from_slice(&[0xFF, 0xE1]);
        out.extend_from_slice(&((app1.len() + 2) as u16).to_be_bytes());
        out.extend_from_slice(&app1);
        out.extend_from_slice(&jpeg[2..]);
        out
    }

    fn sample(w: u32, h: u32) -> RgbaImage {
        RgbaImage::from_pixel(w, h, Rgba([200, 30, 30, 255]))
    }

    #[test]
    fn decodes_png_to_rgba_with_same_size() {
        let img = decode_bytes(&encode(&sample(40, 20), ImageFormat::Png)).unwrap();
        assert_eq!(img.dimensions(), (40, 20));
        assert_eq!(img.get_pixel(0, 0), &Rgba([200, 30, 30, 255]));
    }

    #[test]
    fn decodes_jpeg_to_rgba_with_same_size() {
        let img = decode_bytes(&encode(&sample(40, 20), ImageFormat::Jpeg)).unwrap();
        assert_eq!(img.dimensions(), (40, 20));
        assert_eq!(img.get_pixel(0, 0)[3], 255);
    }

    #[test]
    fn applies_exif_orientation_6_by_rotating() {
        let jpeg = encode(&sample(40, 20), ImageFormat::Jpeg);
        let img = decode_bytes(&with_exif_orientation(&jpeg, 6)).unwrap();
        assert_eq!(img.dimensions(), (20, 40));
    }

    #[test]
    fn corrupted_file_returns_error_without_panicking() {
        assert!(decode_bytes(b"ceci n'est pas une image").is_err());
        let mut png = encode(&sample(40, 20), ImageFormat::Png);
        png.truncate(30);
        assert!(decode_bytes(&png).is_err());
    }

    #[test]
    fn load_image_reads_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.png");
        std::fs::write(&path, encode(&sample(8, 8), ImageFormat::Png)).unwrap();
        assert_eq!(load_image(&path).unwrap().dimensions(), (8, 8));
        assert!(load_image(&dir.path().join("absent.png")).is_err());
    }
}
