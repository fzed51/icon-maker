use icon_maker::engine::{Settings, pipeline};
use image::{ImageFormat, Rgba, RgbaImage};

/// Logo : fond blanc, carré rouge au centre. Enregistré en JPEG.
fn write_logo_jpeg(path: &std::path::Path) {
    let img = RgbaImage::from_fn(120, 80, |x, y| {
        if (40..80).contains(&x) && (20..60).contains(&y) {
            Rgba([220, 20, 20, 255])
        } else {
            Rgba([255, 255, 255, 255])
        }
    });
    image::DynamicImage::ImageRgba8(img)
        .to_rgb8()
        .save_with_format(path, ImageFormat::Jpeg)
        .unwrap();
}

fn settings(export_png: bool) -> Settings {
    let mut s = Settings::default();
    s.background.enabled = true;
    s.export_png = export_png;
    s
}

#[test]
fn jpeg_on_white_gives_icon_with_transparent_corner() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("logo.jpg");
    let output = dir.path().join("logo.ico");
    write_logo_jpeg(&input);

    pipeline::convert_file(&input, &output, &settings(false)).unwrap();

    let icon = image::open(&output).unwrap().to_rgba8();
    assert_eq!(icon.dimensions(), (256, 256));
    assert_eq!(
        icon.get_pixel(0, 0)[3],
        0,
        "le coin devrait être transparent"
    );
    let center = icon.get_pixel(128, 128);
    assert_eq!(center[3], 255);
    assert!(center[0] > 180, "le sujet rouge doit être conservé");
}

#[test]
fn png_export_creates_seven_files_only_when_asked() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("logo.jpg");
    write_logo_jpeg(&input);

    let without = dir.path().join("sans.ico");
    pipeline::convert_file(&input, &without, &settings(false)).unwrap();
    assert!(pipeline::png_paths(&without).iter().all(|p| !p.exists()));

    let with = dir.path().join("avec.ico");
    pipeline::convert_file(&input, &with, &settings(true)).unwrap();
    for p in pipeline::png_paths(&with) {
        assert!(p.exists(), "{} manquant", p.display());
    }
    let small = image::open(dir.path().join("avec_16.png")).unwrap();
    assert_eq!((small.width(), small.height()), (16, 16));
}
