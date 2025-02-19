mod memninja_core;

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

use memninja_core::utils::GenericScanFilter;
use memninja_core::{types::*, CoreCommand, CoreController};

struct MemNinja {
    tree: egui_tiles::Tree<Pane>,
    tree_behaviour: TreeBehaviour,
}

struct TreeBehaviour {
    core: Option<CoreController>,
    process_id: String,
    window_name: String,
    attach_type: AttachType,
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
                if !self.core.as_ref().is_some_and(|core| core.check_attached()) {
                    // Not currently attached
                    if ui.button("Attach").clicked() {
                        match self.attach_type {
                            AttachType::ByPID => {
                                if let Ok(pid) = self.process_id.parse::<u32>() {
                                    if let Some(core) = self.core.as_ref() {
                                        core.send_command(CoreCommand::Attach(
                                            AttachTarget::Process(pid),
                                        ));
                                    }
                                } else {
                                    self.attached_status =
                                        egui::RichText::new("Couldn't parse PID")
                                            .color(egui::Color32::RED);
                                }
                            }
                            AttachType::ByWindowName => {
                                if let Some(core) = self.core.as_ref() {
                                    core.send_command(CoreCommand::Attach(AttachTarget::Window(
                                        self.window_name.clone(),
                                    )));
                                }
                            }
                        };
                    }
                } else {
                    // We are currently attached
                    if ui.button("Detach").clicked() {
                        if let Some(core) = self.core.as_mut() {
                            core.send_command(CoreCommand::Detach);
                        }
                    }
                }

                if let Some(core) = self.core.as_ref() {
                    self.attached_status = match core.get_attach_status() {
                        AttachStatus::Detached => {
                            egui::RichText::new("Detached").color(egui::Color32::RED)
                        }
                        AttachStatus::Attached(_) => {
                            egui::RichText::new("Attached").color(egui::Color32::LIGHT_GREEN)
                        }
                        AttachStatus::Unknown => egui::RichText::new("Unknown Attach Status")
                            .color(egui::Color32::LIGHT_RED),
                    };
                }

                // Show the attach status too
                if self.attached_status.text().len() > 0 {
                    ui.label(self.attached_status.clone());
                }
            });
        });
    }

    fn render_scanner_panel(&mut self, ui: &mut egui::Ui) {
        ui.add_enabled_ui(
            self.core.as_ref().is_some_and(|core| core.check_attached()),
            |ui| {
                ui.heading("Memory Scanning");
                ui.horizontal_top(|ui| {
                    ui.checkbox(&mut self.scan_options.is_hex, "Hex");
                    // TODO: Allow for setting is_hex to true later. For now we don't support hex
                    self.scan_options.is_hex = false;
                    ui.text_edit_singleline(&mut self.scan_options.scan_input);
                    if ui.button("Scan").clicked() {
                        if let Some(core) = self.core.as_ref() {
                            let mem_type = self.scan_options.value_type;
                            let mem_value =
                                mem_type.parse_value(&self.scan_options.scan_input).ok();
                            let scan_filter = GenericScanFilter::new(
                                self.scan_options.scan_type,
                                mem_type,
                                mem_value,
                            );
                            if let Ok(scan_filter) = scan_filter {
                                let _ = core.send_command(CoreCommand::Scan(scan_filter));
                            };
                        }
                    }
                    if ui.button("New Scan").clicked() {
                        if let Some(core) = self.core.as_ref() {
                            let _ = core.send_command(CoreCommand::NewScan);
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
            },
        );
    }

    fn render_results_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.heading("Scan Results");
            if let Some(core) = self.core.as_ref() {
                let scan_status = core.get_scan_status();
                self.scan_results.scan_status = egui::RichText::new(format!("{}", scan_status))
                    .color(match scan_status {
                        ScanStatus::Scanning => Color32::LIGHT_BLUE,
                        ScanStatus::Done(_) => Color32::LIGHT_GREEN,
                        ScanStatus::Failed(_) => Color32::RED,
                        _ => Color32::WHITE,
                    });
            }
            if self.scan_results.scan_status.text().len() > 0 {
                ui.label(self.scan_results.scan_status.clone());
            }
            // ui.label(&self.scan_results.num_results);
        });

        if let Some(core) = self.core.as_ref() {
            let scan_status = core.get_scan_status();
            if let ScanStatus::Done(num_results) = scan_status {
                self.scan_results.num_results = format!("{} Results", num_results);
                self.scan_results.visible_results =
                    core.get_first_results(self.scan_options.value_type, 500);
            }
        }
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
    visible_results: Vec<(usize, String)>,
}

impl Default for MemNinja {
    fn default() -> Self {
        let mut core = CoreController::default();
        core.start().expect("Failure starting MemNinja Core");
        Self {
            tree: create_tree(),
            tree_behaviour: TreeBehaviour {
                core: Some(core),
                process_id: Default::default(),
                window_name: Default::default(),
                attach_type: Default::default(),
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
