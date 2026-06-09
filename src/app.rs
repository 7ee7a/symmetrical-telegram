use eframe::egui;
use std::path::PathBuf;
use crate::extractor;

pub struct ExtractorApp {
    pub dropped_files: Vec<PathBuf>,
    pub excel_path: Option<PathBuf>,
    pub log_messages: Vec<String>,
    pub is_processing: bool,
}

impl Default for ExtractorApp {
    fn default() -> Self {
        Self {
            dropped_files: Vec::new(),
            excel_path: None,
            log_messages: vec!["Application started. Please select an Excel file and drop PDF files.".to_string()],
            is_processing: false,
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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PDF to Excel Extractor");
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Select Excel File").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Excel files", &["xlsx"])
                        .pick_file() {
                        self.excel_path = Some(path.clone());
                        self.log(&format!("Selected Excel: {}", path.display()));
                    }
                }
                if let Some(path) = &self.excel_path {
                    ui.label(path.display().to_string());
                } else {
                    ui.label("No Excel file selected.");
                }
            });

            ui.separator();
            ui.label("Drag and drop PDF files here:");
            
            // Show dropped files
            if !self.dropped_files.is_empty() {
                ui.label(format!("{} files ready.", self.dropped_files.len()));
                let mut files_to_remove = Vec::new();
                for (i, file) in self.dropped_files.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(file.file_name().unwrap_or_default().to_string_lossy());
                        if ui.button("❌").clicked() {
                            files_to_remove.push(i);
                        }
                    });
                }
                for i in files_to_remove.into_iter().rev() {
                    self.dropped_files.remove(i);
                }
                
                if ui.button("Clear All").clicked() {
                    self.dropped_files.clear();
                }
            } else {
                ui.label("No PDFs added yet.");
            }

            ui.separator();

            // Extract Button
            if self.excel_path.is_some() && !self.dropped_files.is_empty() {
                if self.is_processing {
                    ui.label("Processing...");
                } else {
                    if ui.button("Extract & Append").clicked() {
                        self.is_processing = true;
                        self.log("Starting extraction...");
                        
                        let excel_path = self.excel_path.as_ref().unwrap().clone();
                        let pdf_paths = self.dropped_files.clone();
                        
                        // Execute extraction
                        match extractor::process_files(&pdf_paths, &excel_path) {
                            Ok(count) => self.log(&format!("Success! Appended {} rows.", count)),
                            Err(e) => self.log(&format!("Error: {}", e)),
                        }
                        
                        self.is_processing = false;
                    }
                }
            } else {
                ui.label("Please select an Excel file and add PDFs to extract.");
            }

            ui.separator();
            ui.heading("Log");
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                for msg in &self.log_messages {
                    ui.label(msg);
                }
            });
        });

        // Handle drag and drop
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            let files = ctx.input(|i| i.raw.dropped_files.clone());
            for file in files {
                if let Some(path) = file.path {
                    if path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("pdf")) {
                        self.dropped_files.push(path.clone());
                        self.log(&format!("Added PDF: {}", path.display()));
                    } else {
                        self.log(&format!("Ignored non-PDF file: {}", path.display()));
                    }
                }
            }
        }
    }
}
