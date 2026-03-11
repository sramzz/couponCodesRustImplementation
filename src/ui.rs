use eframe::egui;
use std::path::PathBuf;

use coupon_generator::exporter::export_to_excel;
use coupon_generator::generator::generate_coupons;

/// The main application state.
pub struct CouponApp {
    prefix: String,
    base_coupon_code: String,
    count_input: String,
    max_per_file_input: String,
    output_dir: PathBuf,
    status_message: String,
}

impl Default for CouponApp {
    fn default() -> Self {
        Self {
            prefix: String::new(),
            base_coupon_code: "COUPON".to_string(),
            count_input: "100".to_string(),
            max_per_file_input: "10000".to_string(),
            output_dir: dirs::desktop_dir().unwrap_or_else(|| PathBuf::from(".")),
            status_message: String::new(),
        }
    }
}

impl eframe::App for CouponApp {
    /// Called every frame. Draws the UI and handles interaction.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // ── Title ───────────────────────────────────────
            ui.heading("Coupon Generator");
            ui.add_space(12.0);

            // ── Base Coupon Code input ──────────────────────
            ui.horizontal(|ui| {
                ui.label("Base Coupon Code:");
                if ui.text_edit_singleline(&mut self.base_coupon_code).changed() {
                    if self.base_coupon_code.len() > 10 {
                        self.base_coupon_code.truncate(10);
                    }
                }
            });

            // ── Prefix input ────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Prefix:");
                if ui.text_edit_singleline(&mut self.prefix).changed() {
                    if self.prefix.len() > 10 {
                        self.prefix.truncate(10);
                    }
                }
            });

            // ── Count input ─────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Number of coupons:");
                ui.text_edit_singleline(&mut self.count_input);
            });

            // ── Max Per File input ──────────────────────────
            ui.horizontal(|ui| {
                ui.label("Max coupons codes per file:");
                ui.text_edit_singleline(&mut self.max_per_file_input);
            });

            // ── Expected Files Display ──────────────────────
            let count: usize = self.count_input.trim().parse().unwrap_or(0);
            let max_per_file: usize = self.max_per_file_input.trim().parse().unwrap_or(10_000);
            let expected_files = if count == 0 || max_per_file == 0 {
                1
            } else {
                (count + max_per_file - 1) / max_per_file
            };
            ui.label(format!("➔ Expected generated files: {}", expected_files));

            ui.add_space(6.0);

            // ── Output Directory ────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Save To:");
                ui.label(self.output_dir.display().to_string());
                if ui.button("Browse...").clicked() {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        self.output_dir = folder;
                    }
                }
            });

            ui.add_space(12.0);

            // ── Generate button ─────────────────────────────
            if ui.button("Generate & Export to Excel").clicked() {
                self.handle_generate();
            }

            ui.add_space(12.0);

            // ── Status message ──────────────────────────────
            if !self.status_message.is_empty() {
                ui.label(&self.status_message);
            }
        });
    }
}

impl CouponApp {
    /// Validates input, generates coupons, exports to Excel.
    fn handle_generate(&mut self) {
        // Parse the count
        let count: usize = match self.count_input.trim().parse() {
            Ok(n) if n > 0 => n,
            _ => {
                self.status_message =
                    "Please enter a valid positive number for total coupons.".to_string();
                return;
            }
        };

        // Parse max per file
        let max_per_file: usize = match self.max_per_file_input.trim().parse() {
            Ok(n) if n > 0 => n,
            _ => {
                self.status_message =
                    "Please enter a valid positive number for max per file.".to_string();
                return;
            }
        };

        // Generate
        self.status_message = "Generating coupons...".to_string();

        match generate_coupons(&self.prefix, count) {
            Ok(coupons) => {
                let output_dir = &self.output_dir;
                let base_name = if self.base_coupon_code.trim().is_empty() {
                    "COUPON"
                } else {
                    self.base_coupon_code.trim()
                };

                // Export
                match export_to_excel(&coupons, output_dir, base_name, max_per_file) {
                    Ok(files) => {
                        self.status_message = format!(
                            "Done! {} coupons saved to {} file(s) in {}.",
                            coupons.len(),
                            files.len(),
                            output_dir.display()
                        );
                    }
                    Err(e) => {
                        self.status_message =
                            format!("Export failed: {:?}", e);
                    }
                }
            }
            Err(e) => {
                self.status_message =
                    format!("Generation failed: {:?}", e);
            }
        }
    }
}
