use std::borrow::BorrowMut;
use std::str::FromStr;

use eframe::egui::{Color32, Margin, Ui};
use eframe::run_native;
use eframe::App;
use egui::{Vec2, WidgetText};
use egui_tiles::{Behavior, Linear, Tile, TileId, Tiles, Tree};
use hoodmem::scanner::ScanFilter;
use hoodmem::Process;

struct MemNinja {
    tree: egui_tiles::Tree<Pane>,
    tree_behaviour: TreeBehaviour,
}

struct TreeBehaviour {
    process: Option<Box<dyn Process>>,
    scanner: Option<hoodmem::scanner::Scanner>,
    process_id: String,
    window_name: String,
    attach_type: AttachType,
    attached: bool,
    attached_status: egui::RichText,
    scan_options: ScanOptions,
    scan_results: ScanResults,
}

impl Behavior<Pane> for TreeBehaviour {
    fn pane_ui(
        &mut self,
        ui: &mut Ui,
        _tile_id: TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        egui::Frame::default()
            .inner_margin(Margin::same(5.0))
            .show(ui, |ui| match pane.pane_type {
                PaneType::Attach => self.render_attach_panel(ui),
                PaneType::Results => self.render_results_panel(ui),
                PaneType::Scan => self.render_scanner_panel(ui),
            });

        egui_tiles::UiResponse::None

        // You can make your pane draggable like so:
        // if ui
        //     .add(egui::Button::new("Drag me!").sense(egui::Sense::drag()))
        //     .drag_started()
        // {
        //     egui_tiles::UiResponse::DragStarted
        // } else {
        //     egui_tiles::UiResponse::None
        // }
    }

    fn tab_title_for_pane(&mut self, pane: &Pane) -> WidgetText {
        pane.get_pane_title().into()
    }
}

enum PaneType {
    Attach,
    Results,
    Scan,
}

struct Pane {
    pane_type: PaneType,
}

impl Pane {
    fn from_type(pane_type: PaneType) -> Self {
        Self { pane_type }
    }

    fn get_pane_title(&self) -> &str {
        match self.pane_type {
            PaneType::Attach => "Attach",
            PaneType::Results => "Scan Results",
            PaneType::Scan => "Scanner",
        }
    }
}

impl TreeBehaviour {
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
                                    if let Ok(process) = hoodmem::attach_external(pid) {
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
                                    hoodmem::attach_external_by_name(&self.window_name)
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
                        match self.scan_options.value_type {
                            ValueType::U8 => {
                                do_scan::<u8>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            ValueType::U16 => {
                                do_scan::<u16>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            ValueType::U32 => {
                                do_scan::<u32>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            ValueType::U64 => {
                                do_scan::<u64>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            ValueType::I8 => {
                                do_scan::<i8>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            ValueType::I16 => {
                                do_scan::<i16>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            ValueType::I32 => {
                                do_scan::<i32>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            ValueType::I64 => {
                                do_scan::<i64>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            ValueType::F32 => {
                                do_scan::<f32>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            ValueType::F64 => {
                                do_scan::<f64>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                        }
                    }
                }
                if ui.button("New Scan").clicked() {
                    if let Some(scanner) = self.scanner.borrow_mut() {
                        match self.scan_options.value_type {
                            ValueType::U8 => scanner.new_scan(),
                            ValueType::U16 => scanner.new_scan(),
                            ValueType::U32 => scanner.new_scan(),
                            ValueType::U64 => scanner.new_scan(),
                            ValueType::I8 => scanner.new_scan(),
                            ValueType::I16 => scanner.new_scan(),
                            ValueType::I32 => scanner.new_scan(),
                            ValueType::I64 => scanner.new_scan(),
                            ValueType::F32 => scanner.new_scan(),
                            ValueType::F64 => scanner.new_scan(),
                        };
                    }
                    self.scan_results.visible_results.clear();
                    self.scan_results.num_results = "No results yet".into();
                }
            });
            ui.label("Scan Options");
            ui.vertical_centered(|ui| {
                ui.columns(2, |cols| {
                    cols[0].label("Scan Type");
                    egui::ComboBox::from_id_source("Scan Type")
                        .selected_text(format!("{}", self.scan_options.scan_type))
                        .show_ui(&mut cols[1], |ui| {
                            ui.selectable_value(
                                &mut self.scan_options.scan_type,
                                ScanType::Exact,
                                format!("{}", ScanType::Exact),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.scan_type,
                                ScanType::Unknown,
                                format!("{}", ScanType::Unknown),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.scan_type,
                                ScanType::Increased,
                                format!("{}", ScanType::Increased),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.scan_type,
                                ScanType::Decreased,
                                format!("{}", ScanType::Decreased),
                            );
                        });

                    // Value Type
                    cols[0].label("Value Type");
                    egui::ComboBox::from_id_source("Value Type")
                        .selected_text(format!("{}", self.scan_options.value_type))
                        .show_ui(&mut cols[1], |ui| {
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                ValueType::U8,
                                format!("{}", ValueType::U8),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                ValueType::U16,
                                format!("{}", ValueType::U16),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                ValueType::U32,
                                format!("{}", ValueType::U32),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                ValueType::U64,
                                format!("{}", ValueType::U64),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                ValueType::I8,
                                format!("{}", ValueType::I8),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                ValueType::I16,
                                format!("{}", ValueType::I16),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                ValueType::I32,
                                format!("{}", ValueType::I32),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                ValueType::I64,
                                format!("{}", ValueType::I64),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                ValueType::F32,
                                format!("{}", ValueType::F32),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                ValueType::F64,
                                format!("{}", ValueType::F64),
                            );
                        });
                });
            });
        });
    }

    fn render_results_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            if self.scan_results.scan_status.text().len() > 0 {
                ui.label(self.scan_results.scan_status.clone());
            }
            ui.label(&self.scan_results.num_results);
            ui.columns(3, |cols| {
                cols[0].label("Address");
                cols[1].label("Value");
                for (addr, val) in self.scan_results.visible_results.iter() {
                    cols[0].separator();
                    cols[1].separator();
                    cols[0].label(format!("0x{:016x}", addr));
                    cols[1].label(val);
                }
            });
        });
    }
}

#[derive(Default, PartialEq, Debug)]
enum ScanType {
    #[default]
    Exact,
    Unknown,
    Increased,
    Decreased,
}

impl std::fmt::Display for ScanType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fallback = format!("{:?}", self);
        write!(
            f,
            "{}",
            match self {
                ScanType::Exact => "Exact",
                ScanType::Unknown => "Unknown",
                _ => &fallback,
            }
        )
    }
}

#[derive(Default, PartialEq)]
enum ValueType {
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

fn create_tree() -> Tree<Pane> {
    let mut tiles = Tiles::default();
    let attach_pane = tiles.insert_pane(Pane::from_type(PaneType::Attach));
    let scan_pane = tiles.insert_pane(Pane::from_type(PaneType::Scan));
    let results_pane = tiles.insert_pane(Pane::from_type(PaneType::Results));

    let layout_left = Linear {
        children: vec![attach_pane, results_pane],
        dir: egui_tiles::LinearDir::Vertical,
        ..Default::default()
    };
    let left = tiles.insert_new(egui_tiles::Tile::Container(egui_tiles::Container::Linear(
        layout_left,
    )));

    let layout = Linear {
        children: vec![left, scan_pane],
        dir: egui_tiles::LinearDir::Horizontal,
        ..Default::default()
    };

    let root = tiles.insert_new(Tile::Container(egui_tiles::Container::Linear(layout)));
    Tree::new("root", root, tiles)
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ValueType::U8 => "8-bit Integer (unsigned)",
                ValueType::U16 => "16-bit Integer (unsigned)",
                ValueType::U32 => "32-bit Integer (unsigned)",
                ValueType::U64 => "64-bit Integer (unsigned)",
                ValueType::I8 => "8-bit Integer (signed)",
                ValueType::I16 => "16-bit Integer (signed)",
                ValueType::I32 => "32-bit Integer (signed)",
                ValueType::I64 => "64-bit Integer (signed)",
                ValueType::F32 => "Float (32-bit)",
                ValueType::F64 => "Float (64-bit)",
            }
        )
    }
}

#[derive(Default)]
struct ScanOptions {
    value_type: ValueType,
    scan_type: ScanType,
    is_hex: bool,
    scan_input: String,
}

#[derive(Default)]
struct ScanResults {
    scan_status: egui::RichText,
    num_results: String,
    visible_results: Vec<(u64, String)>,
}

impl Default for MemNinja {
    fn default() -> Self {
        Self {
            tree: create_tree(),
            tree_behaviour: TreeBehaviour {
                process: Default::default(),
                scanner: Default::default(),
                process_id: Default::default(),
                window_name: Default::default(),
                attach_type: Default::default(),
                attached: Default::default(),
                attached_status: Default::default(),
                scan_options: Default::default(),
                scan_results: Default::default(),
            },
        }
    }
}

#[derive(Debug, Default, PartialEq)]
enum AttachType {
    #[default]
    ByPID,
    ByWindowName,
}

fn do_scan<T>(
    scanner: &mut hoodmem::scanner::Scanner,
    scan_options: &ScanOptions,
    scan_results: &mut ScanResults,
) where
    T: Copy
        + std::fmt::Debug
        + Send
        + Sync
        + PartialOrd
        + PartialEq
        + std::ops::Sub<Output = T>
        + std::ops::Add<Output = T>
        + FromStr,
{
    match scan_options.scan_type {
        ScanType::Exact => {
            if let Ok(scan_value) = scan_options.scan_input.parse::<T>() {
                if let Err(scan_err) = scanner.scan(ScanFilter::Exact::<T>(scan_value)) {
                    eprintln!("Scan failed: {}", scan_err);
                    scan_results.scan_status =
                        egui::RichText::new("Scan failed (see console for details)")
                            .color(Color32::RED);
                } else {
                    scan_results.scan_status =
                        egui::RichText::new("Scan succeeded").color(Color32::LIGHT_GREEN);
                }
            } else {
                scan_results.scan_status =
                    egui::RichText::new("Invalid scan value").color(Color32::RED);
                scan_results.visible_results.clear();
            }
        }
        ScanType::Unknown => {
            if let Err(scan_err) = scanner.scan(ScanFilter::Unknown::<T>) {
                eprintln!("Scan failed: {}", scan_err);
                scan_results.scan_status =
                    egui::RichText::new("Scan failed (see console for details)")
                        .color(Color32::RED);
            } else {
                scan_results.scan_status =
                    egui::RichText::new("Scan succeeded").color(Color32::LIGHT_GREEN);
            }
        }
        ScanType::Increased => {
            if let Err(scan_err) = scanner.scan(ScanFilter::Increased::<T>) {
                eprintln!("Scan failed: {}", scan_err);
                scan_results.scan_status =
                    egui::RichText::new("Scan failed (see console for details)")
                        .color(Color32::RED);
            } else {
                scan_results.scan_status =
                    egui::RichText::new("Scan succeeded").color(Color32::LIGHT_GREEN);
            }
        }
        ScanType::Decreased => {
            if let Err(scan_err) = scanner.scan(ScanFilter::Decreased::<T>) {
                eprintln!("Scan failed: {}", scan_err);
                scan_results.scan_status =
                    egui::RichText::new("Scan failed (see console for details)")
                        .color(Color32::RED);
            } else {
                scan_results.scan_status =
                    egui::RichText::new("Scan succeeded").color(Color32::LIGHT_GREEN);
            }
        }
    }

    if let Some(num_results) = scanner.count_results() {
        scan_results.num_results = format!("{} results", num_results);
        if num_results <= 50 {
            scan_results.visible_results = scanner
                .get_first_results::<T>(50)
                .iter()
                .map(|(addr, val)| (*addr, format!("{:?}", *val)))
                .collect();
        } else {
            scan_results.visible_results.clear();
        }
    } else {
        scan_results.num_results = "No results yet".into();
        scan_results.visible_results.clear();
    }
}

impl App for MemNinja {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Main app panel
        egui::CentralPanel::default().show(ctx, |ui| {
            // Tree UI
            self.tree.ui(&mut self.tree_behaviour, ui);
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        viewport: egui::ViewportBuilder {
            min_inner_size: Some(Vec2 { x: 1280.0, y: 600.0 }),
            ..Default::default()
        },
        ..Default::default()
    };
    run_native(
        "MemNinja",
        native_options,
        Box::new(|_cc| Box::new(MemNinja::default())),
    )
}
