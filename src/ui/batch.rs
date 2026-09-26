use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use egui::{Context, RichText};

use super::ACCENT;
use crate::engine::Settings;
use crate::engine::batch::{BatchEvent, spawn_batch};

/// Fenêtre « Transformer plusieurs images ».
#[derive(Default)]
pub struct BatchWindow {
    pub open: bool,
    pub files: Vec<PathBuf>,
    pub out_dir: Option<PathBuf>,
    rx: Option<Receiver<BatchEvent>>,
    done: usize,
    total: usize,
    errors: Vec<(PathBuf, String)>,
    finished: Option<(usize, usize)>,
}

impl BatchWindow {
    pub fn open_with(&mut self, files: Vec<PathBuf>) {
        self.open = true;
        if self.rx.is_none() {
            self.files.extend(files);
            self.files.dedup();
        }
    }

    fn running(&self) -> bool {
        self.rx.is_some()
    }

    fn poll(&mut self, ctx: &Context) {
        let Some(rx) = &self.rx else { return };
        while let Ok(event) = rx.try_recv() {
            match event {
                BatchEvent::Progress { done, total } => (self.done, self.total) = (done, total),
                BatchEvent::Failed { file, message } => self.errors.push((file, message)),
                BatchEvent::Finished { ok, failed } => self.finished = Some((ok, failed)),
            }
        }
        if self.finished.is_some() {
            self.rx = None;
        } else {
            ctx.request_repaint();
        }
    }

    pub fn show(&mut self, ctx: &Context, settings: &Settings) {
        self.poll(ctx);
        let mut open = self.open;
        egui::Window::new("Transformer plusieurs images")
            .open(&mut open)
            .collapsible(false)
            .default_width(460.0)
            .show(ctx, |ui| self.contents(ui, settings));
        self.open = open;
    }

    fn contents(&mut self, ui: &mut egui::Ui, settings: &Settings) {
        ui.label("Toutes les images seront transformées avec les réglages actuels.");
        ui.add_enabled_ui(!self.running(), |ui| {
            ui.horizontal(|ui| {
                if ui.button("Ajouter des images…").clicked()
                    && let Some(files) = rfd::FileDialog::new()
                        .add_filter("Images", &["jpg", "jpeg", "png"])
                        .pick_files()
                {
                    self.files.extend(files);
                    self.files.dedup();
                    self.finished = None;
                }
                if !self.files.is_empty() && ui.button("Vider la liste").clicked() {
                    self.files.clear();
                    self.finished = None;
                }
            });
            ui.label(format!("{} image(s) choisie(s)", self.files.len()));
            egui::ScrollArea::vertical()
                .max_height(120.0)
                .show(ui, |ui| {
                    for f in &self.files {
                        ui.label(
                            RichText::new(f.file_name().unwrap_or_default().to_string_lossy())
                                .small(),
                        );
                    }
                });
            ui.horizontal(|ui| {
                ui.label("Enregistrer dans :");
                let name = self
                    .out_dir
                    .as_ref()
                    .map(|d| d.display().to_string())
                    .unwrap_or_else(|| "(pas encore choisi)".into());
                ui.label(RichText::new(name).italics());
                if ui.button("Choisir…").clicked()
                    && let Some(dir) = rfd::FileDialog::new().pick_folder()
                {
                    self.out_dir = Some(dir);
                }
            });
        });

        let ready = !self.files.is_empty() && self.out_dir.is_some() && !self.running();
        let start = egui::Button::new(
            RichText::new("Lancer")
                .size(18.0)
                .color(egui::Color32::WHITE),
        )
        .fill(ACCENT);
        if ui.add_enabled(ready, start).clicked()
            && let Some(out) = &self.out_dir
        {
            self.errors.clear();
            self.finished = None;
            (self.done, self.total) = (0, self.files.len());
            self.rx = Some(spawn_batch(self.files.clone(), out.clone(), *settings));
        }

        if self.running() || self.finished.is_some() {
            let frac = if self.total == 0 {
                0.0
            } else {
                self.done as f32 / self.total as f32
            };
            ui.add(egui::ProgressBar::new(frac).text(format!("{} / {}", self.done, self.total)));
        }
        if let Some((ok, failed)) = self.finished {
            let msg = if failed == 0 {
                format!("✔ C'est fini ! {ok} icône(s) créée(s).")
            } else {
                format!("C'est fini : {ok} icône(s) créée(s), {failed} image(s) n'ont pas marché.")
            };
            ui.label(RichText::new(msg).strong());
            if let Some(out) = &self.out_dir
                && ui.button("Ouvrir le dossier").clicked()
            {
                let _ = std::process::Command::new("explorer").arg(out).spawn();
            }
        }
        for (file, message) in &self.errors {
            ui.colored_label(
                egui::Color32::from_rgb(200, 60, 40),
                format!(
                    "✖ {} : {message}",
                    file.file_name().unwrap_or_default().to_string_lossy()
                ),
            );
        }
    }
}
