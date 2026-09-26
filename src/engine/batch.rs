use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};

use super::{Settings, SquareMode, pipeline};

#[derive(Debug, Clone, PartialEq)]
pub enum BatchEvent {
    /// `done` fichiers traités sur `total` (réussis ou non).
    Progress { done: usize, total: usize },
    /// Un fichier n'a pas pu être transformé ; `message` est lisible par l'utilisateur.
    Failed { file: PathBuf, message: String },
    Finished { ok: usize, failed: usize },
}

/// Réglages adaptés au lot : couleur du fond auto par image, recadrage centré.
pub fn batch_settings(settings: &Settings) -> Settings {
    let mut s = *settings;
    s.background.key = None;
    if let SquareMode::Crop { .. } = s.square {
        s.square = SquareMode::Crop { offset: 0.5 };
    }
    s
}

/// Transforme chaque fichier en `out_dir/<nom>.ico` et envoie la progression sur `tx`.
/// Les erreurs d'envoi sont ignorées : si l'interface a fermé le canal, on termine simplement.
pub fn run_batch(files: &[PathBuf], out_dir: &Path, settings: &Settings, tx: &Sender<BatchEvent>) {
    let settings = batch_settings(settings);
    let total = files.len();
    let mut failed = 0;
    for (i, file) in files.iter().enumerate() {
        let stem = file.file_stem().unwrap_or_default().to_string_lossy();
        let output = out_dir.join(format!("{stem}.ico"));
        if let Err(e) = pipeline::convert_file(file, &output, &settings) {
            failed += 1;
            let _ = tx.send(BatchEvent::Failed {
                file: file.clone(),
                message: e.to_string(),
            });
        }
        let _ = tx.send(BatchEvent::Progress { done: i + 1, total });
    }
    let _ = tx.send(BatchEvent::Finished {
        ok: total - failed,
        failed,
    });
}

/// Lance [`run_batch`] dans un thread séparé.
pub fn spawn_batch(files: Vec<PathBuf>, out_dir: PathBuf, settings: Settings) -> Receiver<BatchEvent> {
    let (tx, rx) = channel();
    std::thread::spawn(move || run_batch(&files, &out_dir, &settings, &tx));
    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::SquareMode;
    use image::{Rgb, Rgba, RgbaImage};
    use std::sync::mpsc::channel;

    #[test]
    fn batch_settings_force_auto_key_and_centered_crop() {
        let mut s = Settings::default();
        s.background.key = Some(Rgb([1, 2, 3]));
        s.square = SquareMode::Crop { offset: 0.1 };
        let b = batch_settings(&s);
        assert_eq!(b.background.key, None);
        assert_eq!(b.square, SquareMode::Crop { offset: 0.5 });

        s.square = SquareMode::Pad;
        assert_eq!(batch_settings(&s).square, SquareMode::Pad);
    }

    #[test]
    fn three_files_one_corrupted() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out");
        std::fs::create_dir(&out).unwrap();
        let img = RgbaImage::from_pixel(30, 30, Rgba([0, 0, 255, 255]));
        let a = dir.path().join("a.png");
        let b = dir.path().join("b.png");
        let bad = dir.path().join("casse.jpg");
        img.save(&a).unwrap();
        img.save(&b).unwrap();
        std::fs::write(&bad, b"pas une image").unwrap();

        let (tx, rx) = channel();
        run_batch(&[a, bad.clone(), b], &out, &Settings::default(), &tx);
        drop(tx);
        let events: Vec<_> = rx.iter().collect();

        assert!(out.join("a.ico").exists());
        assert!(out.join("b.ico").exists());
        assert!(!out.join("casse.ico").exists());

        let progress = events.iter().filter(|e| matches!(e, BatchEvent::Progress { .. })).count();
        assert_eq!(progress, 3);
        let failed: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                BatchEvent::Failed { file, message } => Some((file.clone(), message.clone())),
                _ => None,
            })
            .collect();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].0, bad);
        assert!(!failed[0].1.is_empty());
        assert_eq!(events.last(), Some(&BatchEvent::Finished { ok: 2, failed: 1 }));
    }

    #[test]
    fn spawn_batch_finishes_in_background() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.png");
        RgbaImage::from_pixel(10, 10, Rgba([0, 0, 0, 255])).save(&a).unwrap();
        let rx = spawn_batch(vec![a], dir.path().to_path_buf(), Settings::default());
        let last = rx.iter().last();
        assert_eq!(last, Some(BatchEvent::Finished { ok: 1, failed: 0 }));
    }
}
