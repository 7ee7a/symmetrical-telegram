use eframe::egui;
use std::path::PathBuf;
use crate::extractor;

pub struct ExtractorApp {
    pub dropped_files: Vec<PathBuf>,
    pub excel_path: Option<PathBuf>,
    pub log_messages: Vec<String>,
    pub is_processing: bool,
    pub extract_success_time: Option<std::time::Instant>,
}

impl Default for ExtractorApp {
    fn default() -> Self {
        Self {
            dropped_files: Vec::new(),
            excel_path: None,
            log_messages: vec!["Application started. Please select an Excel file and drop PDF files.".to_string()],
            is_processing: false,
            extract_success_time: None,
        }
    }
}

impl ExtractorApp {
    fn log(&mut self, msg: &str) {
        self.log_messages.push(msg.to_string());
    }
}

impl eframe::App for ExtractorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());

        // Footer pinned to the bottom
        egui::TopBottomPanel::bottom("footer_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("V1.1 Created by W1164 for queries contact aditya.gottapu@waisldigital.com")
                    .color(egui::Color32::GRAY)
                    .size(12.0));
            });
            ui.add_space(8.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new("✈️ 🅿️ Invoice PDF to Excel").size(28.0).strong());
            });
            ui.add_space(15.0);

            // Excel Picker Group
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.label(egui::RichText::new("1. Select Target Excel File").size(16.0).strong());
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    if ui.button("📁 Browse...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Excel files", &["xlsx"])
                            .pick_file() {
                            self.excel_path = Some(path.clone());
                            self.extract_success_time = None;
                            self.log(&format!("Selected Excel: {}", path.display()));
                        }
                    }
                    if let Some(path) = &self.excel_path {
                        ui.label(path.display().to_string());
                    } else {
                        ui.label(egui::RichText::new("No Excel file selected.").color(egui::Color32::GRAY));
                    }
                });
            });

            ui.add_space(15.0);

            // Dropzone Group
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.label(egui::RichText::new("2. Add PDF Files").size(16.0).strong());
                ui.add_space(5.0);
                ui.label("Drag and drop your PDF files directly into this window.");
                ui.add_space(10.0);
                
                if !self.dropped_files.is_empty() {
                    ui.label(egui::RichText::new(format!("{} files ready.", self.dropped_files.len())).color(egui::Color32::LIGHT_GREEN));
                    ui.add_space(5.0);
                    
                    egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                        let mut files_to_remove = Vec::new();
                        for (i, file) in self.dropped_files.iter().enumerate() {
                            ui.horizontal(|ui| {
                                if ui.button("❌").clicked() {
                                    files_to_remove.push(i);
                                }
                                ui.label(file.file_name().unwrap_or_default().to_string_lossy());
                            });
                        }
                        for i in files_to_remove.into_iter().rev() {
                            self.dropped_files.remove(i);
                        }
                    });
                    
                    ui.add_space(5.0);
                    if ui.button("Clear All").clicked() {
                        self.dropped_files.clear();
                    }
                } else {
                    ui.label(egui::RichText::new("No PDFs added yet.").color(egui::Color32::GRAY));
                }
            });

            ui.add_space(25.0);

            // Action Button
            ui.vertical_centered(|ui| {
                if self.excel_path.is_some() && !self.dropped_files.is_empty() {
                    if self.is_processing {
                        ui.add(egui::Spinner::new());
                        ui.label("Processing...");
                    } else {
                        let mut btn_text = "🚀 Extract & Append";
                        let mut btn_color = egui::Color32::from_rgb(0, 122, 204);
                        
                        if let Some(success_time) = self.extract_success_time {
                            if success_time.elapsed().as_secs() < 3 {
                                btn_text = "✅ Completed!";
                                btn_color = egui::Color32::from_rgb(40, 167, 69);
                                ctx.request_repaint_after(std::time::Duration::from_millis(500));
                            } else {
                                self.extract_success_time = None;
                            }
                        }

                        let btn = egui::Button::new(egui::RichText::new(btn_text).size(20.0))
                            .fill(btn_color);
                        
                        if ui.add_sized([250.0, 50.0], btn).clicked() {
                            self.is_processing = true;
                            self.log("Starting extraction...");
                            
                            let excel_path = self.excel_path.as_ref().unwrap().clone();
                            let pdf_paths = self.dropped_files.clone();
                            
                            match extractor::process_files(&pdf_paths, &excel_path) {
                                Ok((count, warnings)) => {
                                    self.extract_success_time = Some(std::time::Instant::now());
                                    self.log(&format!("Success! Appended {} rows.", count));
                                    if !warnings.is_empty() {
                                        self.log(&format!("Warning: {} rows were ignored because their format didn't match.", warnings.len()));
                                        for w in warnings.iter() { 
                                            self.log(&format!("Ignored: {}", w));
                                        }
                                    }
                                }
                                Err(e) => self.log(&format!("Error: {}", e)),
                            }
                            
                            self.is_processing = false;
                        }
                    }
                } else {
                    ui.label(egui::RichText::new("Please complete Step 1 and Step 2 to enable extraction.").color(egui::Color32::from_rgb(200, 100, 100)));
                }
            });

            ui.add_space(20.0);
            ui.separator();
            
            // Console log area (Collapsible)
            egui::CollapsingHeader::new(egui::RichText::new("📝 Console Log").size(16.0))
                .default_open(false)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    egui::ScrollArea::vertical().stick_to_bottom(true).max_height(150.0).show(ui, |ui| {
                        for msg in &self.log_messages {
                            if msg.starts_with("Error") || msg.starts_with("Ignored") {
                                ui.label(egui::RichText::new(msg).color(egui::Color32::from_rgb(255, 100, 100)));
                            } else if msg.starts_with("Success") {
                                ui.label(egui::RichText::new(msg).color(egui::Color32::LIGHT_GREEN));
                            } else {
                                ui.label(msg);
                            }
                        }
                    });
                });
        });

        // Handle drag and drop
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            let files = ctx.input(|i| i.raw.dropped_files.clone());
            for file in files {
                if let Some(path) = file.path {
                    if path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("pdf")) {
                        self.dropped_files.push(path.clone());
                        self.extract_success_time = None;
                        self.log(&format!("Added PDF: {}", path.display()));
                    } else {
                        self.log(&format!("Ignored non-PDF file: {}", path.display()));
                    }
                }
            }
        }
    }
}
