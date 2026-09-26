use std::collections::VecDeque;

use image::{Rgb, Rgba, RgbaImage};

/// Couleur moyenne des 4 coins : bonne estimation de la couleur du fond.
pub fn corner_color(img: &RgbaImage) -> Rgb<u8> {
    let (w, h) = img.dimensions();
    let corners = [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)];
    let mut sum = [0u32; 3];
    for (x, y) in corners {
        let p = img.get_pixel(x, y);
        for c in 0..3 {
            sum[c] += p[c] as u32;
        }
    }
    Rgb(sum.map(|s| ((s + 2) / 4) as u8))
}

/// Écart entre deux couleurs, de 0 (identiques) à 100 (noir ↔ blanc).
fn distance(p: &Rgba<u8>, key: Rgb<u8>) -> f32 {
    let sq: f32 = (0..3)
        .map(|c| (p[c] as f32 - key[c] as f32).powi(2))
        .sum();
    sq.sqrt() / (255.0 * 3f32.sqrt()) * 100.0
}

/// Rend transparent le fond de couleur `key` relié aux bords de l'image.
///
/// `tolerance` et `feather` vont de 0 à 100. Les pixels à une distance ≤ `tolerance`
/// deviennent transparents ; entre `tolerance` et `tolerance + feather`, ils deviennent
/// partiellement transparents. Le blanc à l'intérieur du sujet n'est pas touché.
pub fn remove_background(img: &mut RgbaImage, key: Rgb<u8>, tolerance: f32, feather: f32) {
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return;
    }
    let limit = tolerance + feather;
    // Un pixel déjà transparent laisse passer la diffusion, quelle que soit sa couleur.
    let reachable = |p: &Rgba<u8>| p[3] == 0 || distance(p, key) <= limit;

    let mut visited = vec![false; (w * h) as usize];
    let mut queue = VecDeque::new();
    let border = (0..w)
        .flat_map(|x| [(x, 0), (x, h - 1)])
        .chain((0..h).flat_map(|y| [(0, y), (w - 1, y)]));
    for (x, y) in border {
        let i = (y * w + x) as usize;
        if !visited[i] && reachable(img.get_pixel(x, y)) {
            visited[i] = true;
            queue.push_back((x, y));
        }
    }

    while let Some((x, y)) = queue.pop_front() {
        let p = img.get_pixel_mut(x, y);
        let d = distance(p, key);
        let factor = if d <= tolerance {
            0.0
        } else {
            ((d - tolerance) / feather).min(1.0)
        };
        p[3] = (p[3] as f32 * factor).round() as u8;

        let neighbors = [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ];
        for (nx, ny) in neighbors {
            if nx >= w || ny >= h {
                continue;
            }
            let i = (ny * w + nx) as usize;
            if !visited[i] && reachable(img.get_pixel(nx, ny)) {
                visited[i] = true;
                queue.push_back((nx, ny));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
    const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);

    /// Fond blanc 20×20, anneau noir de 10×10 au centre, intérieur blanc.
    fn ring_on_white() -> RgbaImage {
        let mut img = RgbaImage::from_pixel(20, 20, WHITE);
        for y in 5..15 {
            for x in 5..15 {
                let edge = x == 5 || x == 14 || y == 5 || y == 14;
                if edge {
                    img.put_pixel(x, y, BLACK);
                }
            }
        }
        img
    }

    #[test]
    fn uniform_white_background_becomes_transparent() {
        let mut img = ring_on_white();
        remove_background(&mut img, Rgb([255, 255, 255]), 10.0, 0.0);
        assert_eq!(img.get_pixel(0, 0)[3], 0);
        assert_eq!(img.get_pixel(19, 19)[3], 0);
        assert_eq!(img.get_pixel(3, 10)[3], 0);
    }

    #[test]
    fn white_inside_subject_is_not_erased() {
        let mut img = ring_on_white();
        remove_background(&mut img, Rgb([255, 255, 255]), 10.0, 10.0);
        assert_eq!(img.get_pixel(10, 10), &WHITE, "l'intérieur du sujet est troué");
        assert_eq!(img.get_pixel(5, 5), &BLACK);
    }

    #[test]
    fn zero_tolerance_only_erases_exact_color() {
        let mut img = RgbaImage::from_pixel(10, 10, WHITE);
        img.put_pixel(0, 5, Rgba([254, 254, 254, 255]));
        remove_background(&mut img, Rgb([255, 255, 255]), 0.0, 0.0);
        assert_eq!(img.get_pixel(0, 0)[3], 0);
        assert_eq!(img.get_pixel(0, 5)[3], 255);
    }

    #[test]
    fn feather_gives_partial_alpha_in_transition() {
        // Gris 230 : distance ≈ 9,8 du blanc → entre tolérance (5) et tolérance + adoucissement (15).
        let mut img = RgbaImage::from_pixel(10, 10, WHITE);
        img.put_pixel(0, 5, Rgba([230, 230, 230, 255]));
        remove_background(&mut img, Rgb([255, 255, 255]), 5.0, 10.0);
        let a = img.get_pixel(0, 5)[3];
        assert!(a > 0 && a < 255, "alpha attendu intermédiaire, obtenu {a}");
        assert_eq!(img.get_pixel(0, 0)[3], 0);
    }

    #[test]
    fn already_transparent_pixels_stay_transparent() {
        let mut img = RgbaImage::from_pixel(10, 10, Rgba([10, 200, 10, 255]));
        img.put_pixel(4, 4, Rgba([10, 200, 10, 0]));
        img.put_pixel(0, 0, Rgba([255, 255, 255, 0]));
        remove_background(&mut img, Rgb([255, 255, 255]), 10.0, 0.0);
        assert_eq!(img.get_pixel(4, 4)[3], 0);
        assert_eq!(img.get_pixel(0, 0)[3], 0);
        assert_eq!(img.get_pixel(5, 5)[3], 255);
    }

    #[test]
    fn corner_color_is_average_of_corners() {
        let mut img = RgbaImage::from_pixel(10, 10, BLACK);
        img.put_pixel(0, 0, Rgba([100, 0, 0, 255]));
        img.put_pixel(9, 0, Rgba([200, 0, 0, 255]));
        img.put_pixel(0, 9, Rgba([100, 40, 0, 255]));
        img.put_pixel(9, 9, Rgba([200, 40, 8, 255]));
        assert_eq!(corner_color(&img), Rgb([150, 20, 2]));
    }
}
