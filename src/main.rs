// Pas de console noire derrière la fenêtre en version finale.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use icon_maker::app::IconMakerApp;

/// Icône de la barre de titre et de la barre des tâches (la plus grande taille du .ico).
fn window_icon() -> egui::IconData {
    let img = image::load_from_memory(include_bytes!("../assets/app.ico"))
        .expect("assets/app.ico invalide")
        .to_rgba8();
    egui::IconData {
        width: img.width(),
        height: img.height(),
        rgba: img.into_raw(),
    }
}

fn main() -> eframe::Result {
    let initial = std::env::args_os().nth(1).map(std::path::PathBuf::from);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Icon Maker — crée ton icône")
            .with_inner_size([1040.0, 760.0])
            .with_min_inner_size([860.0, 620.0])
            .with_drag_and_drop(true)
            .with_icon(window_icon()),
        ..Default::default()
    };
    eframe::run_native(
        "Icon Maker",
        options,
        Box::new(|cc| Ok(Box::new(IconMakerApp::new(cc, initial)))),
    )
}
