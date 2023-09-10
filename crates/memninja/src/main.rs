use std::borrow::BorrowMut;

use eframe::egui::{self, Margin};
use eframe::epaint::Color32;
use eframe::{run_native, App};

#[derive(Default)]
struct MemNinja {
    process: Option<hoodmem::Process>,
    scanner: Option<hoodmem::scanner::Scanner>,
    process_id: String,
    window_name: String,
    attach_type: AttachType,
    attached: bool,
    attached_status: egui::RichText,
    scan_options: ScanOptions,
}

#[derive(Default)]
enum ScanType {
    #[default]
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}

#[derive(Default)]
struct ScanOptions {
    scan_type: ScanType,
    is_hex: bool,
    scan_input: String,
}

impl MemNinja {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

#[derive(Debug, Default, PartialEq)]
enum AttachType {
    #[default]
    ByPID,
    ByWindowName,
}

fn do_scan<T>(scanner: &mut hoodmem::scanner::Scanner, scan_options: &ScanOptions) {}

impl MemNinja {
    /// Render the attach to process panel
    fn render_attach_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.label("Attach to process");
            ui.columns(2, |cols| {
                cols[0].radio_value(&mut self.attach_type, AttachType::ByPID, "By PID");
                cols[1].text_edit_singleline(&mut self.process_id);
            });

            ui.columns(2, |cols| {
                cols[0].radio_value(
                    &mut self.attach_type,
                    AttachType::ByWindowName,
                    "By Window Name",
                );
                cols[1].text_edit_singleline(&mut self.window_name);
            });

            ui.horizontal_wrapped(|ui| {
                // Attached status, as well as an attached or detached button
                if !self.attached {
                    // Not currently attached
                    if ui.button("Attach").clicked() {
                        match self.attach_type {
                            AttachType::ByPID => {
                                if let Ok(pid) = self.process_id.parse::<u32>() {
                                    if let Ok(process) = hoodmem::Process::attach(pid) {
                                        self.scanner =
                                            Some(hoodmem::scanner::Scanner::new(process));
                                        self.attached = true;
                                        self.attached_status = egui::RichText::new("Attached")
                                            .color(egui::Color32::LIGHT_GREEN);
                                    } else {
                                        self.attached_status =
                                            egui::RichText::new("Failed to attach to process")
                                                .color(egui::Color32::RED);
                                    }
                                } else {
                                    self.attached_status =
                                        egui::RichText::new("Couldn't parse PID")
                                            .color(egui::Color32::RED);
                                }
                            }
                            AttachType::ByWindowName => {
                                if let Ok(process) =
                                    hoodmem::Process::attach_by_name(&self.window_name)
                                {
                                    self.scanner = Some(hoodmem::scanner::Scanner::new(process));
                                    self.attached = true;
                                    self.attached_status = egui::RichText::new("Attached")
                                        .color(egui::Color32::LIGHT_GREEN);
                                } else {
                                    self.attached_status =
                                        egui::RichText::new("Failed to attach to process")
                                            .color(egui::Color32::RED);
                                }
                            }
                        };
                    }
                } else {
                    // We are currently attached
                    if ui.button("Detach").clicked() {
                        self.scanner = None;
                        self.process = None;
                        self.attached = false;
                        self.attached_status =
                            egui::RichText::new("Detached").color(egui::Color32::RED);
                    }
                }

                // Show the attach status too
                if self.attached_status.text().len() > 0 {
                    ui.label(self.attached_status.clone());
                }
            });
        });
    }

    fn render_scanner_panel(&mut self, ui: &mut egui::Ui) {
        ui.add_enabled_ui(self.attached, |ui| {
            ui.label("Memory Scanning");
            ui.horizontal_top(|ui| {
                ui.checkbox(&mut self.scan_options.is_hex, "Hex");
                ui.text_edit_singleline(&mut self.scan_options.scan_input);
                if ui.button("Scan").clicked() {
                    if let Some(scanner) = self.scanner.borrow_mut() {
                        match self.scan_options.scan_type {
                            ScanType::U8 => do_scan::<u8>(scanner, &self.scan_options),
                            ScanType::U16 => do_scan::<u16>(scanner, &self.scan_options),
                            ScanType::U32 => do_scan::<u32>(scanner, &self.scan_options),
                            ScanType::U64 => do_scan::<u64>(scanner, &self.scan_options),
                            ScanType::I8 => do_scan::<i8>(scanner, &self.scan_options),
                            ScanType::I16 => do_scan::<i16>(scanner, &self.scan_options),
                            ScanType::I32 => do_scan::<i32>(scanner, &self.scan_options),
                            ScanType::I64 => do_scan::<i64>(scanner, &self.scan_options),
                            ScanType::F32 => do_scan::<f32>(scanner, &self.scan_options),
                            ScanType::F64 => do_scan::<f64>(scanner, &self.scan_options),
                        }
                    }
                }
                if ui.button("New Scan").clicked() {
                    if let Some(scanner) = self.scanner.borrow_mut() {
                        match self.scan_options.scan_type {
                            ScanType::U8 => scanner.new_scan(),
                            ScanType::U16 => scanner.new_scan(),
                            ScanType::U32 => scanner.new_scan(),
                            ScanType::U64 => scanner.new_scan(),
                            ScanType::I8 => scanner.new_scan(),
                            ScanType::I16 => scanner.new_scan(),
                            ScanType::I32 => scanner.new_scan(),
                            ScanType::I64 => scanner.new_scan(),
                            ScanType::F32 => scanner.new_scan(),
                            ScanType::F64 => scanner.new_scan(),
                        };
                    }
                }
            });
        });
    }
}

impl App for MemNinja {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        // Main app panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |cols| {
                // Attach to processes panel
                egui::Frame::default()
                    .stroke(eframe::epaint::Stroke {
                        width: 1.0,
                        color: Color32::from_rgb(20, 20, 20),
                    })
                    .inner_margin(Margin::same(5.0))
                    .show(&mut cols[0], |ui| {
                        self.render_attach_panel(ui);
                        ui.spacing();
                    });

                // Scan panel
                egui::Frame::default()
                    .stroke(eframe::epaint::Stroke {
                        width: 1.0,
                        color: Color32::from_rgb(20, 20, 20),
                    })
                    .inner_margin(Margin::same(5.0))
                    .show(&mut cols[1], |ui| {
                        self.render_scanner_panel(ui);
                    });
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        min_window_size: Some(egui::vec2(1000.0, 600.0)),
        ..Default::default()
    };
    run_native(
        "MemNinja",
        native_options,
        Box::new(|cc| Box::new(MemNinja::new(cc))),
    )
}
