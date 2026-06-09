#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod extractor;

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([650.0, 500.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    
    eframe::run_native(
        "PDF to Excel Extractor",
        options,
        Box::new(|_cc| Box::new(app::ExtractorApp::default())),
    )
}
