use std::{collections::HashMap, path::PathBuf, ptr::null};

use eframe::{
    glow::{LEFT, RIGHT},
    *,
};
use egui::{CursorIcon, Id, Pos2, UiBuilder, epaint::tessellator::Path};
use egui_dnd::{DragDropResponse, dnd};
use log::Log;
use serde;

use super::Gate;
use super::node::*;

mod pan_area;
use pan_area::PanArea;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct MyApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,

    files_loaded: bool,
    prims: Vec<Primitive>,
    saved: Vec<Primitive>,

    current_chip: Option<PathBuf>,
    live_data: Vec<Gate>,

    pan_center: Pos2,
    dragging_kind: Option<GateType>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            files_loaded: false,

            prims: Vec::new(),
            saved: Vec::new(),

            current_chip: None,
            live_data: Vec::<Gate>::new(),

            pan_center: Pos2::new(0.0, 0.0),
            dragging_kind: None,
        }
    }
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            Default::default()
            // eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }

    pub fn load_chips() -> Vec<Primitive> {
        let mut gates = Vec::<Primitive>::new();

        //read saves directory for each file add a gate to the vector
        let dir = std::fs::read_dir("./saves").unwrap();
        for entry in dir {
            print!("Loading gate: ");
            let entry = entry.unwrap();
            if entry.path().is_file() {
                println!("{}", entry.path().display());
                // Check if the file name ends with ".gate"
                let file_name = entry.file_name().into_string().unwrap();
                if file_name.ends_with(".gate") {
                    // Load the gate from the file
                    let gate_template = Primitive::new(file_name);
                    gates.push(gate_template);
                }
            }
        }
        println!("Loaded {} gates", gates.len());
        gates
    }

    pub fn load_prims() -> Vec<Primitive> {
        let mut prims: Vec<Primitive> = Vec::new();

        //read saves directory for each file add a gate to the vector
        let data = std::fs::read_to_string("./saves/primitives").unwrap();
        let lines = data.lines();
        for l in lines {
            print!("Loading gate: ");
            let split = l.split(":");
            let mut parts = Vec::<&str>::new();

            for n in split {
                parts.push(n)
            }
            let n1 = parts[1].parse::<usize>();
            let n2 = parts[2].parse::<usize>();
            if let Ok(n1) = n1 {
                if let Ok(n2) = n2 {
                    let new_gate = Primitive::from_values(parts[0], n1, n2);
                    prims.push(new_gate);
                }
            }
        }
        println!("Loaded {} gates", prims.len());
        prims
    }

    pub fn load_data(path: &str) -> Vec<Gate> {
        //for every line in the file, create a gate
        let data = std::fs::read_to_string(path).unwrap();
        let lines = data.lines();
        let mut gates = Vec::<Gate>::new();
        for l in lines {
            let split = l.split(":");
            let mut parts = Vec::<&str>::new();

            for n in split {
                parts.push(n)
            }
            if parts.len() < 3 {
                continue; // skip invalid lines
            }
            let label = parts[0].to_string();
            let n_ins = parts[1]
                .split("[")
                .nth(1)
                .and_then(|s| s.split("]").next())
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            let n_outs = parts[2]
                .split("[")
                .nth(1)
                .and_then(|s| s.split("]").next())
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            println!(
                "Creating gate: {} with {} inputs and {} outputs",
                label, n_ins, n_outs
            );
            println!("Parts: {:?}", parts);

            let gate = Gate::generate(label, n_ins, n_outs);
            gates.push(gate);
        }
        gates
    }

    pub fn save_chips(&self) {
        // Save each gate to a file in the saves directory, overwriting existing files
        std::fs::create_dir_all("./saves").unwrap();
        for gate in &self.saved {
            let file_path = format!("./saves/{}.gate", gate.label);
            let serialized_gate = serde_json::to_string(gate).unwrap();
            std::fs::write(file_path, serialized_gate).unwrap();
        }
    }
}

impl eframe::App for MyApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        if !self.files_loaded {
            self.saved = Self::load_chips();
            println!("Loaded chips: {}", self.saved.len());

            self.prims = Self::load_prims();
            println!("Loaded prims: {}", self.prims.len());

            self.files_loaded = true
        }
        egui::SidePanel::left("Tools").show(ctx, |ui| {
            ui.set_min_width(400.);
            ui.vertical_centered_justified(|ui| {
                //display title "Saved Gates" 3/4 the width of the panel, bold, and centered
                ui.heading("Saved Chips");
                //display all saved gates in a vertical list
                // Add a button to create a new gate
                if ui.button("New Chip").clicked() {
                    let new_chip = Primitive::new("New Chip".to_string());
                    self.saved.push(new_chip);
                    self.save_chips();
                };

                //two "columns" first 80% th width for chip name, second 20% width for trash icon
                let mut idx = 0;
                let mut queue_rem: Option<usize> = None;

                for g in &self.saved {
                    ui.horizontal(|ui| {
                        ui.add(g.make_prim_widget());

                        if ui.button("Edit").clicked() {
                            // Remove the gate from the saved gates
                            // self.open_gate(idx)
                            println!("TODO! Open gate for editing")
                        }
                        if ui.button("Delete").clicked() {
                            // Remove the gate from the saved gates
                            queue_rem = Some(idx);
                            self.save_chips();
                        }
                        idx += 1;
                    });
                }
                // Remove the gate from the saved gates
                if queue_rem.is_some() {
                    self.saved.remove(idx - 1);
                };
            })
        });

        egui::TopBottomPanel::top("top_panel_bar").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Themes", |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                });
                ui.add_space(16.0);
            });
        });

        egui::TopBottomPanel::top("Primitive Library").show(ctx, |ui| {
            ui.set_min_height(150.);
            ui.horizontal(|ui| {
                ui.set_min_height(15.);
                ui.label("Primitive Gates");
            });

            // println!("Primitive gates: {:?}", self.primitive_gates);
            ui.horizontal_centered(|ui| {
                let response =
                    dnd(ui, "Primitive").show_vec(&mut self.prims, |ui, item, handle, state| {
                        let h = handle.ui(ui, |ui| {
                            let w = ui.add(item.make_prim_widget());
                            if w.is_pointer_button_down_on() {
                                self.dragging_kind = Some(item.kind.clone());
                                println!("Dragging kind: {:?}", self.dragging_kind);
                            } else if w.drag_stopped_by(egui::PointerButton::Primary) {
                                if let Some(kind) = &self.dragging_kind {
                                    let pos = ctx.pointer_hover_pos().unwrap_or(self.pan_center);

                                    let new_gate = Gate::from_template_id(kind.to_string(), pos);

                                    self.live_data.push(new_gate);
                                    println!("Added new gate: {:?}", self.dragging_kind);

                                    self.dragging_kind = None;
                                }
                                self.dragging_kind = None;
                            } else if ui.input(|i| i.pointer.any_released()) {
                                 if let Some(kind) = &self.dragging_kind {
                                    let pos = ctx.pointer_hover_pos().unwrap_or(self.pan_center);

                                    let new_gate = Gate::from_template_id(kind.to_string(), pos);

                                    self.live_data.push(new_gate);
                                    println!("Added new gate: {:?}", self.dragging_kind);

                                    self.dragging_kind = None;
                                }
                                self.dragging_kind = None;
                            }
                        });
                    });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut new_pan_center = self.pan_center; // copy the value (Pos2 is Copy)
            //if drag released in this panel, add the gate to the live data if it was a primitive

            ui.add(PanArea::new(
                &mut new_pan_center,
                |ui: &mut egui::Ui, pan_center: Pos2| {
                    // draw logic here using `pan_center`
                    for mut gate in self.live_data.clone() {
                        // Get gate world position
                        let world_pos = &gate.position;

                        // Convert to screen-local position
                        let screen_pos = world_pos.to_pos2() - pan_center.to_vec2();

                        // Place the widget at the screen position
                        let rect = egui::Rect::from_min_size(screen_pos, egui::vec2(100.0, 60.0)); // customize size
                        let builder = UiBuilder::new()
                            .max_rect(rect)
                            .sense(egui::Sense::click_and_drag());
                        ui.scope_builder(builder, |ui| {
                            ui.add(&mut gate);
                        });
                    }

                    self.pan_center = pan_center; // update AFTER the widget runs
                },
            ));
        });
    }
}
