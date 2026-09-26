use image::imageops::{self, FilterType};
use image::{ImageBuffer, Rgba, Rgba32FImage, RgbaImage};

use super::ICON_SIZES;

/// Redimensionne une image carrée en `size`×`size`, en alpha prémultiplié (pas de halo sombre).
pub fn resize_square(img: &RgbaImage, size: u32) -> RgbaImage {
    // Prémultiplier : un pixel transparent ne transmet plus sa couleur à ses voisins.
    let premul: Rgba32FImage = ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        let p = img.get_pixel(x, y);
        let a = p[3] as f32 / 255.0;
        Rgba([
            p[0] as f32 / 255.0 * a,
            p[1] as f32 / 255.0 * a,
            p[2] as f32 / 255.0 * a,
            a,
        ])
    });
    let resized = imageops::resize(&premul, size, size, FilterType::Lanczos3);
    RgbaImage::from_fn(size, size, |x, y| {
        let p = resized.get_pixel(x, y);
        // Lanczos peut déborder légèrement de [0, 1].
        let a = p[3].clamp(0.0, 1.0);
        if a < 1.0 / 255.0 {
            return Rgba([0, 0, 0, 0]);
        }
        let to_u8 = |v: f32| (v * 255.0).round().clamp(0.0, 255.0) as u8;
        Rgba([
            to_u8(p[0] / a),
            to_u8(p[1] / a),
            to_u8(p[2] / a),
            to_u8(a),
        ])
    })
}

/// Produit une image par taille de [`ICON_SIZES`].
pub fn resize_all(img: &RgbaImage) -> Vec<RgbaImage> {
    ICON_SIZES.iter().map(|&s| resize_square(img, s)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ICON_SIZES;
    use image::Rgba;

    #[test]
    fn produces_exactly_the_seven_sizes() {
        let img = RgbaImage::from_pixel(300, 300, Rgba([0, 128, 255, 255]));
        let sizes: Vec<_> = resize_all(&img).iter().map(|i| i.dimensions()).collect();
        let expected: Vec<_> = ICON_SIZES.iter().map(|&s| (s, s)).collect();
        assert_eq!(sizes, expected);
    }

    #[test]
    fn no_dark_halo_on_transparent_edges() {
        // Moitié gauche : noir totalement transparent ; moitié droite : rouge opaque.
        let img = RgbaImage::from_fn(64, 64, |x, _| {
            if x < 32 { Rgba([0, 0, 0, 0]) } else { Rgba([255, 0, 0, 255]) }
        });
        let out = resize_square(&img, 16);
        for p in out.pixels().filter(|p| p[3] > 8) {
            assert!(p[0] > 230, "halo sombre : {p:?}");
            assert!(p[1] < 20 && p[2] < 20, "couleur parasite : {p:?}");
        }
    }

    #[test]
    fn fully_transparent_area_stays_transparent() {
        let img = RgbaImage::from_fn(64, 64, |x, _| {
            if x < 32 { Rgba([0, 0, 0, 0]) } else { Rgba([255, 0, 0, 255]) }
        });
        let out = resize_square(&img, 16);
        assert_eq!(out.get_pixel(0, 0)[3], 0);
        assert_eq!(out.get_pixel(3, 8)[3], 0);
    }
}
