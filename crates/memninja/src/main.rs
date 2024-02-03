use std::borrow::BorrowMut;
use std::fmt::Pointer;
use std::str::FromStr;
use std::{default, mem};

use eframe::egui::{Color32, Margin, Ui};
use eframe::run_native;
use eframe::App;
use egui::{Vec2, WidgetText};
use egui_extras::Column;
use egui_tiles::{Behavior, Linear, Tile, TileId, Tiles, Tree};
use hoodmem::scanner::ScanFilter;
use hoodmem::Process;

struct MemNinja {
    tree: egui_tiles::Tree<Pane>,
    tree_behaviour: TreeBehaviour,
}

enum MemValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Null,
}

impl std::fmt::Display for MemValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemValue::U8(x) => write!(f, "{}", x),
            MemValue::U16(x) => write!(f, "{}", x),
            MemValue::U32(x) => write!(f, "{}", x),
            MemValue::U64(x) => write!(f, "{}", x),
            MemValue::I8(x) => write!(f, "{}", x),
            MemValue::I16(x) => write!(f, "{}", x),
            MemValue::I32(x) => write!(f, "{}", x),
            MemValue::I64(x) => write!(f, "{}", x),
            MemValue::F32(x) => write!(f, "{}", x),
            MemValue::F64(x) => write!(f, "{}", x),
            MemValue::Null => write!(f, "null"),
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
enum MemType {
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
    Unknown,
}

impl std::fmt::Display for MemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MemType::U8 => "8-bit Integer (unsigned)",
                MemType::U16 => "16-bit Integer (unsigned)",
                MemType::U32 => "32-bit Integer (unsigned)",
                MemType::U64 => "64-bit Integer (unsigned)",
                MemType::I8 => "8-bit Integer (signed)",
                MemType::I16 => "16-bit Integer (signed)",
                MemType::I32 => "32-bit Integer (signed)",
                MemType::I64 => "64-bit Integer (signed)",
                MemType::F32 => "Float (32-bit)",
                MemType::F64 => "Float (64-bit)",
                MemType::Unknown => "Unknown",
            }
        )
    }
}

impl From<MemValue> for MemType {
    fn from(value: MemValue) -> Self {
        match value {
            MemValue::U8(_) => Self::U8,
            MemValue::U16(_) => Self::U16,
            MemValue::U32(_) => Self::U32,
            MemValue::U64(_) => Self::U64,
            MemValue::I8(_) => Self::I8,
            MemValue::I16(_) => Self::I16,
            MemValue::I32(_) => Self::I32,
            MemValue::I64(_) => Self::I64,
            MemValue::F32(_) => Self::F32,
            MemValue::F64(_) => Self::F64,
            MemValue::Null => Self::Unknown,
        }
    }
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
    scan_results: MemValues,
    min_results_index: usize,
    max_results_index: usize,
    cheats: Vec<Cheat>,
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
                PaneType::Cheats => self.render_cheats_panel(ui),
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
    Cheats,
}

enum CheatType {
    Simple { addr: u64, mem_type: MemType },
}

trait CheatSummary {
    fn get_summary(&self) -> String;
}

impl CheatSummary for CheatType {
    fn get_summary(&self) -> String {
        match self {
            CheatType::Simple { addr, mem_type } => format!("[{}] 0x{:016x}", mem_type, addr),
        }
    }
}

impl std::fmt::Display for CheatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheatType::Simple { addr, mem_type } => write!(f, "Simple ({})", mem_type),
        }
    }
}

struct Cheat {
    enabled: bool,
    name: String,
    cheat_type: CheatType,
}

impl CheatSummary for Cheat {
    fn get_summary(&self) -> String {
        self.cheat_type.get_summary()
    }
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
            PaneType::Cheats => "Cheats",
        }
    }
}

impl TreeBehaviour {
    fn get_results_range(
        &self,
        start_index: usize,
        end_index: usize,
    ) -> Vec<Option<(u64, MemValue)>> {
        let mut final_results: Vec<Option<(u64, MemValue)>> = vec![];
        if let Some(ref scanner) = self.scanner {
            match self.scan_options.value_type {
                MemType::U8 => {
                    let results = scanner.get_results_range::<u8>(start_index, end_index);
                    final_results.extend(
                        results
                            .iter()
                            .map(|(addr, val)| Some((*addr, MemValue::U8(*val)))),
                    );
                }
                MemType::U16 => {
                    let results = scanner.get_results_range::<u16>(start_index, end_index);
                    final_results.extend(
                        results
                            .iter()
                            .map(|(addr, val)| Some((*addr, MemValue::U16(*val)))),
                    );
                }
                MemType::U32 => {
                    let results = scanner.get_results_range::<u32>(start_index, end_index);
                    final_results.extend(
                        results
                            .iter()
                            .map(|(addr, val)| Some((*addr, MemValue::U32(*val)))),
                    );
                }
                MemType::U64 => {
                    let results = scanner.get_results_range::<u64>(start_index, end_index);
                    final_results.extend(
                        results
                            .iter()
                            .map(|(addr, val)| Some((*addr, MemValue::U64(*val)))),
                    );
                }
                MemType::I8 => {
                    let results = scanner.get_results_range::<i8>(start_index, end_index);
                    final_results.extend(
                        results
                            .iter()
                            .map(|(addr, val)| Some((*addr, MemValue::I8(*val)))),
                    );
                }
                MemType::I16 => {
                    let results = scanner.get_results_range::<i16>(start_index, end_index);
                    final_results.extend(
                        results
                            .iter()
                            .map(|(addr, val)| Some((*addr, MemValue::I16(*val)))),
                    );
                }
                MemType::I32 => {
                    let results = scanner.get_results_range::<i32>(start_index, end_index);
                    final_results.extend(
                        results
                            .iter()
                            .map(|(addr, val)| Some((*addr, MemValue::I32(*val)))),
                    );
                }
                MemType::I64 => {
                    let results = scanner.get_results_range::<i64>(start_index, end_index);
                    final_results.extend(
                        results
                            .iter()
                            .map(|(addr, val)| Some((*addr, MemValue::I64(*val)))),
                    );
                }
                MemType::F32 => {
                    let results = scanner.get_results_range::<f32>(start_index, end_index);
                    final_results.extend(
                        results
                            .iter()
                            .map(|(addr, val)| Some((*addr, MemValue::F32(*val)))),
                    );
                }
                MemType::F64 => {
                    let results = scanner.get_results_range::<f64>(start_index, end_index);
                    final_results.extend(
                        results
                            .iter()
                            .map(|(addr, val)| Some((*addr, MemValue::F64(*val)))),
                    );
                }
                MemType::Unknown => panic!("Cannot read values of type Unknown"),
            };
        };

        final_results
    }

    fn render_cheats_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.heading("Cheats");
            ui.push_id("CheatsUI", |ui| {
                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .columns(Column::remainder().at_least(200.0), 4)
                    .sense(egui::Sense {
                        click: true,
                        drag: false,
                        focusable: true,
                    })
                    .auto_shrink(false)
                    .min_scrolled_height(20.0)
                    .header(20.0, |mut header_row| {
                        header_row.col(|ui| {
                            ui.heading("Enabled");
                        });
                        header_row.col(|ui| {
                            ui.heading("Cheat");
                        });
                        header_row.col(|ui| {
                            ui.heading("Type");
                        });
                        header_row.col(|ui| {
                            ui.heading("Info");
                        });
                    })
                    .body(|tbody| {
                        tbody.rows(20.0, self.cheats.len(), |mut row| {
                            let row_index = row.index();
                            let cheat = self.cheats[row_index].borrow_mut();
                            row.col(|ui| {
                                ui.checkbox(&mut cheat.enabled, "");
                            });
                            row.col(|ui| {
                                ui.label(&cheat.name);
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", cheat.cheat_type));
                            });
                            row.col(|ui| {
                                ui.label(cheat.get_summary());
                            });

                            if row.response().double_clicked() {}
                        });
                    });
            });
        });
    }

    fn render_attach_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.heading("Attach to process");
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
            ui.heading("Memory Scanning");
            ui.horizontal_top(|ui| {
                ui.checkbox(&mut self.scan_options.is_hex, "Hex");
                ui.text_edit_singleline(&mut self.scan_options.scan_input);
                if ui.button("Scan").clicked() {
                    if let Some(scanner) = self.scanner.borrow_mut() {
                        match self.scan_options.value_type {
                            MemType::U8 => {
                                do_scan::<u8>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            MemType::U16 => {
                                do_scan::<u16>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            MemType::U32 => {
                                do_scan::<u32>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            MemType::U64 => {
                                do_scan::<u64>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            MemType::I8 => {
                                do_scan::<i8>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            MemType::I16 => {
                                do_scan::<i16>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            MemType::I32 => {
                                do_scan::<i32>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            MemType::I64 => {
                                do_scan::<i64>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            MemType::F32 => {
                                do_scan::<f32>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            MemType::F64 => {
                                do_scan::<f64>(scanner, &self.scan_options, &mut self.scan_results)
                            }
                            MemType::Unknown => panic!("Cannot scan for values of unknown type"),
                        }
                    }
                }
                if ui.button("New Scan").clicked() {
                    if let Some(scanner) = self.scanner.borrow_mut() {
                        match self.scan_options.value_type {
                            MemType::U8 => scanner.new_scan(),
                            MemType::U16 => scanner.new_scan(),
                            MemType::U32 => scanner.new_scan(),
                            MemType::U64 => scanner.new_scan(),
                            MemType::I8 => scanner.new_scan(),
                            MemType::I16 => scanner.new_scan(),
                            MemType::I32 => scanner.new_scan(),
                            MemType::I64 => scanner.new_scan(),
                            MemType::F32 => scanner.new_scan(),
                            MemType::F64 => scanner.new_scan(),
                            MemType::Unknown => panic!("Cannot scan for values of unknown type"),
                        };
                    }
                    self.scan_results.visible_results.clear();
                    self.scan_results.num_results = "No results yet".into();
                }
            });
            ui.heading("Scan Options");
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
                                MemType::U8,
                                format!("{}", MemType::U8),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                MemType::U16,
                                format!("{}", MemType::U16),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                MemType::U32,
                                format!("{}", MemType::U32),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                MemType::U64,
                                format!("{}", MemType::U64),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                MemType::I8,
                                format!("{}", MemType::I8),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                MemType::I16,
                                format!("{}", MemType::I16),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                MemType::I32,
                                format!("{}", MemType::I32),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                MemType::I64,
                                format!("{}", MemType::I64),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                MemType::F32,
                                format!("{}", MemType::F32),
                            );
                            ui.selectable_value(
                                &mut self.scan_options.value_type,
                                MemType::F64,
                                format!("{}", MemType::F64),
                            );
                        });
                });
            });
        });
    }

    fn render_results_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.heading("Scan Results");
            if self.scan_results.scan_status.text().len() > 0 {
                ui.label(self.scan_results.scan_status.clone());
            }
            ui.label(&self.scan_results.num_results);
        });

        ui.push_id("ResultsUI", |ui| {
            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .columns(Column::remainder().at_least(200.0), 2)
                .sense(egui::Sense {
                    click: true,
                    drag: false,
                    focusable: true,
                })
                .auto_shrink(false)
                .min_scrolled_height(20.0)
                .header(20.0, |mut header_row| {
                    header_row.col(|ui| {
                        ui.heading("Address");
                    });
                    header_row.col(|ui| {
                        ui.heading("Value");
                    });
                })
                .body(|tbody| {
                    tbody.rows(20.0, self.scan_results.visible_results.len(), |mut row| {
                        let row_index = row.index();
                        if let Some((addr, val)) = self.scan_results.visible_results.get(row_index)
                        {
                            row.col(|ui| {
                                ui.label(format!("0x{:016x}", addr));
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", val));
                            });
                            if row.response().double_clicked() {
                                self.cheats.push(Cheat {
                                    enabled: false,
                                    name: "New Cheat".into(),
                                    cheat_type: CheatType::Simple {
                                        addr: *addr,
                                        mem_type: self.scan_options.value_type,
                                    },
                                })
                            }
                        } else {
                            row.col(|ui| {
                                ui.label("null");
                            });
                            row.col(|ui| {
                                ui.label("null");
                            });
                        }
                    });
                });
        });

        ui.add_space(20.0);
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

fn create_tree() -> Tree<Pane> {
    let mut tiles = Tiles::default();
    let attach_pane = tiles.insert_pane(Pane::from_type(PaneType::Attach));
    let scan_pane = tiles.insert_pane(Pane::from_type(PaneType::Scan));
    let results_pane = tiles.insert_pane(Pane::from_type(PaneType::Results));
    let cheats_pane = tiles.insert_pane(Pane::from_type(PaneType::Cheats));

    let layout_left = Linear {
        children: vec![attach_pane, results_pane],
        dir: egui_tiles::LinearDir::Vertical,
        ..Default::default()
    };
    let left = tiles.insert_new(egui_tiles::Tile::Container(egui_tiles::Container::Linear(
        layout_left,
    )));

    let layout_top = Linear {
        children: vec![left, scan_pane],
        dir: egui_tiles::LinearDir::Horizontal,
        ..Default::default()
    };

    let top = tiles.insert_new(egui_tiles::Tile::Container(egui_tiles::Container::Linear(
        layout_top,
    )));

    let layout = Linear {
        children: vec![top, cheats_pane],
        dir: egui_tiles::LinearDir::Vertical,
        ..Default::default()
    };

    let root = tiles.insert_new(Tile::Container(egui_tiles::Container::Linear(layout)));
    Tree::new("root", root, tiles)
}

#[derive(Default)]
struct ScanOptions {
    value_type: MemType,
    scan_type: ScanType,
    is_hex: bool,
    scan_input: String,
}

#[derive(Default)]
struct MemValues {
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
                min_results_index: 0,
                max_results_index: 0,
                cheats: vec![],
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
    scan_results: &mut MemValues,
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
            min_inner_size: Some(Vec2 {
                x: 1280.0,
                y: 600.0,
            }),
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
