pub mod state;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui::{RichText, TextureHandle, TextureOptions, Ui};

use crate::engine::{Settings, pipeline, resize};
use crate::ui::{self, ACCENT, BIG_BUTTON, batch::BatchWindow, preview, settings, sizes_strip};
use state::{AppState, Step};

/// Délai d'attente après un réglage avant de recalculer l'aperçu.
const DEBOUNCE: Duration = Duration::from_millis(150);

pub struct IconMakerApp {
    state: AppState,
    preview_tex: Option<TextureHandle>,
    sizes_tex: Vec<TextureHandle>,
    /// Réglages utilisés pour l'aperçu affiché ; `None` = à recalculer.
    rendered: Option<Settings>,
    changed_at: Option<Instant>,
    show_more: bool,
    batch: BatchWindow,
}

impl IconMakerApp {
    /// `initial` : image déposée sur l'exe ou passée par « Ouvrir avec ».
    pub fn new(cc: &eframe::CreationContext<'_>, initial: Option<PathBuf>) -> Self {
        ui::apply_style(&cc.egui_ctx);
        let mut app = Self {
            state: AppState::default(),
            preview_tex: None,
            sizes_tex: Vec::new(),
            rendered: None,
            changed_at: None,
            show_more: false,
            batch: BatchWindow::default(),
        };
        if let Some(path) = initial {
            app.open(&path);
        }
        app
    }

    fn open(&mut self, path: &std::path::Path) {
        self.state.open(path);
        if self.state.error.is_none() {
            self.rendered = None;
            self.changed_at = None;
        }
    }

    fn open_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["jpg", "jpeg", "png"])
            .pick_file()
        {
            self.open(&path);
        }
    }

    fn save_dialog(&mut self) {
        let suggested = self.state.suggested_save_path();
        let mut dialog = rfd::FileDialog::new().add_filter("Icône Windows", &["ico"]);
        if let Some(path) = &suggested {
            if let Some(dir) = path.parent() {
                dialog = dialog.set_directory(dir);
            }
            if let Some(name) = path.file_name() {
                dialog = dialog.set_file_name(name.to_string_lossy());
            }
        }
        if let Some(mut path) = dialog.save_file() {
            if path.extension().is_none_or(|e| !e.eq_ignore_ascii_case("ico")) {
                path.set_extension("ico");
            }
            self.state.error = self.state.save(&path).err().map(|e| e.to_string());
        }
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).collect());
        match dropped.len() {
            0 => {}
            1 => self.open(&dropped[0]),
            _ => self.batch.open_with(dropped),
        }
    }

    /// Recalcule l'aperçu quand les réglages ont changé, après un court délai.
    fn refresh_preview(&mut self, ctx: &egui::Context) {
        let Some(src) = &self.state.preview_source else { return };
        let settings = self.state.settings;
        if self.rendered == Some(settings) {
            return;
        }
        if self.rendered.is_some() {
            // Un réglage a changé : l'icône enregistrée ne correspond plus.
            self.state.last_saved = None;
            let since = *self.changed_at.get_or_insert_with(Instant::now);
            if since.elapsed() < DEBOUNCE {
                ctx.request_repaint_after(DEBOUNCE);
                return;
            }
        }
        let prepared = pipeline::prepare(src, &settings);
        self.preview_tex = Some(ui::to_texture(ctx, "preview", &prepared, TextureOptions::LINEAR));
        self.sizes_tex = resize::resize_all(&prepared)
            .iter()
            .enumerate()
            .map(|(i, img)| ui::to_texture(ctx, &format!("size{i}"), img, TextureOptions::NEAREST))
            .collect();
        self.rendered = Some(settings);
        self.changed_at = None;
    }

    fn top_bar(&mut self, ui: &mut Ui) {
        ui.add_space(6.0);
        ui::steps::show(ui, self.state.step());
        if let Some(err) = self.state.error.clone() {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(200, 60, 40), RichText::new(format!("✖ {err}")).strong());
                if ui.small_button("OK").clicked() {
                    self.state.error = None;
                }
            });
        }
        if let (Step::Saved, Some(path)) = (self.state.step(), self.state.last_saved.clone()) {
            ui.horizontal(|ui| {
                ui.label(RichText::new("✔ Icône enregistrée !").strong().color(egui::Color32::from_rgb(20, 140, 60)));
                ui.label(RichText::new(path.file_name().unwrap_or_default().to_string_lossy()).weak());
                if ui.button("Ouvrir le dossier").clicked() {
                    let _ = std::process::Command::new("explorer")
                        .arg(format!("/select,{}", path.display()))
                        .spawn();
                }
            });
        }
        ui.add_space(4.0);
    }

    fn bottom_bar(&mut self, ui: &mut Ui) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            sizes_strip::show(ui, &self.sizes_tex);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let btn = egui::Button::new(RichText::new("💾  Enregistrer mon icône").size(20.0).color(egui::Color32::WHITE))
                    .fill(ACCENT)
                    .min_size(BIG_BUTTON + egui::vec2(60.0, 10.0));
                if ui.add(btn).on_hover_text("Choisis où ranger ton icône").clicked() {
                    self.save_dialog();
                }
            });
        });
        ui.add_space(6.0);
    }
}

impl eframe::App for IconMakerApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_dropped_files(&ctx);
        self.refresh_preview(&ctx);
        let has_image = self.state.source.is_some();
        let files_hovered = ctx.input(|i| !i.raw.hovered_files.is_empty());

        egui::Panel::top("steps").show(ui, |ui| self.top_bar(ui));
        if has_image {
            egui::Panel::bottom("sizes").show(ui, |ui| self.bottom_bar(ui));
            let action = egui::Panel::right("settings")
                .exact_size(290.0)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .show(ui, |ui| settings::show(ui, &mut self.state, &mut self.show_more))
                        .inner
                })
                .inner;
            match action {
                settings::SettingsAction::OpenDialog => self.open_dialog(),
                settings::SettingsAction::OpenBatch => self.batch.open_with(Vec::new()),
                settings::SettingsAction::None => {}
            }
        }
        let action = egui::CentralPanel::default()
            .show(ui, |ui| preview::show(ui, &mut self.state, self.preview_tex.as_ref(), files_hovered))
            .inner;
        if let preview::PreviewAction::OpenDialog = action {
            self.open_dialog();
        }

        self.batch.show(&ctx, &self.state.settings);
    }
}
