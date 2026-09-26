use image::{RgbaImage, imageops};

use super::SquareMode;

/// Centre l'image sur un carré transparent de côté `max(w, h)`.
pub fn pad_to_square(img: &RgbaImage) -> RgbaImage {
    let (w, h) = img.dimensions();
    let side = w.max(h);
    let mut out = RgbaImage::new(side, side);
    imageops::replace(
        &mut out,
        img,
        ((side - w) / 2) as i64,
        ((side - h) / 2) as i64,
    );
    out
}

/// Découpe un carré de côté `min(w, h)`. `offset` (0.0 → 1.0, limité) choisit la position
/// le long du grand côté.
pub fn crop_square(img: &RgbaImage, offset: f32) -> RgbaImage {
    let (w, h) = img.dimensions();
    let side = w.min(h);
    let start = ((w.max(h) - side) as f32 * offset.clamp(0.0, 1.0)).round() as u32;
    let (x, y) = if w > h { (start, 0) } else { (0, start) };
    imageops::crop_imm(img, x, y, side, side).to_image()
}

pub fn make_square(img: &RgbaImage, mode: SquareMode) -> RgbaImage {
    match mode {
        SquareMode::Pad => pad_to_square(img),
        SquareMode::Crop { offset } => crop_square(img, offset),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    const RED: Rgba<u8> = Rgba([255, 0, 0, 255]);
    const BLUE: Rgba<u8> = Rgba([0, 0, 255, 255]);

    /// 200×100 : moitié gauche rouge, moitié droite bleue.
    fn wide() -> RgbaImage {
        RgbaImage::from_fn(200, 100, |x, _| if x < 100 { RED } else { BLUE })
    }

    #[test]
    fn pad_wide_image_gives_centered_square_with_transparent_bands() {
        let out = pad_to_square(&wide());
        assert_eq!(out.dimensions(), (200, 200));
        assert_eq!(out.get_pixel(0, 0)[3], 0);
        assert_eq!(out.get_pixel(199, 49)[3], 0);
        assert_eq!(out.get_pixel(0, 50), &RED);
        assert_eq!(out.get_pixel(199, 149), &BLUE);
        assert_eq!(out.get_pixel(100, 150)[3], 0);
    }

    #[test]
    fn pad_tall_image_adds_side_bands() {
        let tall = RgbaImage::from_pixel(50, 100, RED);
        let out = pad_to_square(&tall);
        assert_eq!(out.dimensions(), (100, 100));
        assert_eq!(out.get_pixel(0, 0)[3], 0);
        assert_eq!(out.get_pixel(50, 0), &RED);
    }

    #[test]
    fn crop_with_offset_zero_keeps_start() {
        let out = crop_square(&wide(), 0.0);
        assert_eq!(out.dimensions(), (100, 100));
        assert!(out.pixels().all(|p| *p == RED));
    }

    #[test]
    fn crop_offset_out_of_range_is_clamped() {
        let out = crop_square(&wide(), 5.0);
        assert!(out.pixels().all(|p| *p == BLUE));
        let out = crop_square(&wide(), -3.0);
        assert!(out.pixels().all(|p| *p == RED));
    }

    #[test]
    fn crop_centered_takes_the_middle() {
        let out = crop_square(&wide(), 0.5);
        assert_eq!(out.get_pixel(0, 0), &RED);
        assert_eq!(out.get_pixel(99, 0), &BLUE);
    }

    #[test]
    fn square_image_is_unchanged() {
        let sq = RgbaImage::from_pixel(64, 64, RED);
        assert_eq!(make_square(&sq, SquareMode::Pad), sq);
        assert_eq!(make_square(&sq, SquareMode::Crop { offset: 0.3 }), sq);
    }
}
