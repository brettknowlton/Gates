mod ui_util;
pub use ui_util::ClickItem;

mod pan_area;
use pan_area::PanArea;

use super::node::*;

use std::collections::HashMap;
use std::path::PathBuf;

use eframe::{self, *};
use egui::{Pos2, Ui, UiBuilder};
use egui_dnd::dnd;
use serde;

const TITLE_BAR_HEIGHT: f32 = 30.0;

const SIDE_PANEL_WIDTH: f32 = 400.0;

static mut NEXT_ID: usize = 0; // static variable to generate unique ids for gates and wires

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

    #[serde(skip)]
    live_data: HashMap<usize, Box<dyn Logical>>, // (GateType, position, id)

    pan_center: Pos2,
    pan_area_rect: Option<egui::Rect>,
    dragging_kind: Option<GateType>,
    pub dragging_gate: Option<usize>,
    pub clicked_gate: Option<usize>, // the gate that is currently being clicked on

    #[serde(skip)]
    pub click_item: Option<ClickItem>,
    pub holding_wire: Option<Box<Wire>>,
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
            live_data: HashMap::<usize, Box<dyn Logical>>::new(),

            pan_center: Pos2::new(0.0, 0.0),
            pan_area_rect: None,
            dragging_kind: None,
            
            dragging_gate: None,
            clicked_gate: None,

            click_item: None,
            holding_wire: None,
        }
    }
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let new: Self;
        if let Some(_) = cc.storage {
            new = Default::default();
            // eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            new = Default::default();
        }

        new.gates_setup()
    }

    fn gates_setup(mut self) -> Self {
        self.saved = Self::load_chips();
        println!("Loaded chips: {}", self.saved.len());

        self.prims = Self::load_prims();
        println!("Loaded prims: {}", self.prims.len());

        self.files_loaded = true;
        self
    }
    pub fn load_chips() -> Vec<Primitive> {
        let gates = Vec::<Primitive>::new();

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
                    // let gate_template = Primitive::new(file_name);
                    // gates.push(gate_template);
                }
            }
        }
        println!("Loaded {} gates", gates.len());
        gates
    }

    pub fn next_id() -> usize {
        //generate new id incrementally from a static variable
        let id = unsafe { NEXT_ID };
        unsafe { NEXT_ID += 1 };
        id
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

    fn update_wires(&mut self, ui: &mut Ui) {
        //loop live data and collect all inputs and outputs into one HashMap and Wires into another
        let gates: HashMap<usize, Pos2> = self
            .live_data
            .iter()
            .filter_map(|(id, item)| {
                if let Some(input) = item.as_any().downcast_ref::<Input>() {
                    // println!(
                    //     "Input found: {:?} at position {:?} with parent: {:?}",
                    //     input,
                    //     input.get_position(&self.live_data),
                    //     input.parent_id
                    // );
                    Some((*id, input.get_position(&self.live_data).unwrap()))
                } else if let Some(output) = item.as_any().downcast_ref::<Output>() {
                    Some((*id, output.get_position(&self.live_data).unwrap()))
                } else {
                    None
                }
            })
            .collect();
        println!("Found {} gates", gates.len());

        // Now iterate mutably to update wires, using only the data collected above
        for (_id, wire) in self.live_data.iter_mut() {
            if let Some(w) = wire.as_any_mut().downcast_mut::<Wire>() {
                if w.connected {
                    if let Some(source_pos) = gates.get(&w.source_id) {
                        w.set_positions(
                            *source_pos,
                            w.dest
                                .and_then(|dest_id| gates.get(&dest_id).cloned())
                                .unwrap_or(*source_pos),
                        );
                    }
                } else {
                    //if not connected, p2 will be on the mouse cursor
                    if let Some(cursor_pos) = ui.input(|i| i.pointer.hover_pos()) {
                        w.set_p2(cursor_pos);
                    }
                    // If not connected, set p1 to the source position
                    if let Some(source_pos) = gates.get(&w.source_id) {
                        w.set_p1(*source_pos);
                    }
                }
            }
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

        //check if output_click exists, if so, call the function
        if let Some(clicked_io) = &self.click_item {
            // an output was clicked, so we want to create a wire if we are not currently holding a wire
            //lookup the type of the clicked IO by its id in the live_data map
            match clicked_io.item_type {
                Logicals::IO(IOKind::Input) => {
                    println!("Clicked on Input: {:?}", clicked_io);

                    if let Some(mut wire) = self.holding_wire.take() {
                        //connect the wire to the input
                        println!("Connecting wire to input: {:?}", clicked_io);
                        self.holding_wire = None;

                        //set wire's p2 to the clicked position
                        // and set the wires destination to the input's id
                        wire.set_p2(clicked_io.screen_position);
                        wire.dest = Some(clicked_io.item_id);
                        wire.connected = true; // mark the wire as connected
                        self.live_data.insert(wire.id, wire);
                    }else {
                        println!("No wire to connect to input, holding_wire is None");
                    }
                }
                Logicals::IO(IOKind::Output) => {
                    println!("Clicked on Output: {:?}", clicked_io);
                    if self.holding_wire.is_none() {
                        println!("Creating wire from clicked IO: {:?}", clicked_io);
                        self.holding_wire = Some(Wire::from_io(
                            clicked_io.item_id,
                            clicked_io.screen_position,
                        ));
                    }else{
                        //reconnect the wire using this output as the new source
                        if let Some(mut wire) = self.holding_wire.take() {
                            println!("Reconnecting wire to output: {:?}", clicked_io);

                            wire.source_id = clicked_io.item_id;
                            wire.set_p1(clicked_io.screen_position);

                        } else {
                            println!("No wire to reconnect, holding_wire is None");
                        }
                    }
                }
                _ => {
                    println!("Clicked on unknown IO type: {:?}", clicked_io);
                }
            }

            // Clear the click item after processing
            self.click_item = None;
        }

        egui::SidePanel::left("Tools").show(ctx, |ui| {
            ui.set_min_width(SIDE_PANEL_WIDTH);
            ui.vertical_centered_justified(|ui| {
                //display title "Saved Gates" 3/4 the width of the panel, bold, and centered
                ui.heading("Saved Chips");
                //display all saved gates in a vertical list
                // Add a button to create a new gate
                if ui.button("New Chip").clicked() {
                    let new_chip = Primitive::from_values("New Chip", 0, 0);
                    self.saved.push(new_chip);
                    self.save_chips();
                };

                //two "columns" first 80% th width for chip name, second 20% width for trash icon
                let mut idx = 0;
                let mut queue_rem: Option<usize> = None;

                for g in &self.saved {
                    ui.horizontal(|ui| {
                        ui.add(g.make_toolbox_widget());

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
            ui.set_min_height(TITLE_BAR_HEIGHT);
            ui.set_max_height(TITLE_BAR_HEIGHT);

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
                dnd(ui, "Primitive").show_vec(&mut self.prims, |ui, item, handle, _state| {
                    handle.ui(ui, |ui| {
                        let w = ui.add(item.make_toolbox_widget());
                        if w.is_pointer_button_down_on() {
                            self.dragging_kind = Some(item.kind.clone());
                            println!("Dragging kind: {:?}", self.dragging_kind);
                        } else if ui.input(|i| i.pointer.any_released()) {
                            if let Some(kind) = &self.dragging_kind {
                                // Check if pointer is over the PanArea
                                if let Some(pointer_pos) = ctx.pointer_hover_pos() {
                                    if let Some(pan_area_rect) = self.pan_area_rect {
                                        if pan_area_rect.contains(pointer_pos) {
                                            println!("Pointer is over PanArea, adding gate");
                                            let world_pos = pointer_pos + self.pan_center.to_vec2();
                                            let mut gate = Gate::create_gate_from_template(
                                                kind.clone(),
                                                world_pos,
                                            );

                                            gate.create_io(&mut self.live_data);

                                            self.live_data.insert(
                                                // Create a new gate at the world position
                                                gate.id,
                                                Box::new(gate),
                                            );

                                            println!("Added new gate: {:?}", self.dragging_kind);
                                            println!("Mouse position: {:?}", pointer_pos);
                                            println!("World position: {:?}", world_pos);
                                            println!("Pan center: {:?}", self.pan_center);
                                        }
                                    }
                                }
                            }
                            self.dragging_kind = None;
                        }
                    });
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut new_pan_center = self.pan_center; // copy the value (Pos2 is Copy)

            // Update pan_area_rect each frame so dragndrop items stay functional
            let available_rect = ui.available_rect_before_wrap();
            self.pan_area_rect = Some(available_rect);

            ui.add(PanArea::with_drag_blocker(
                &mut new_pan_center,
                &self.dragging_gate.is_some(),
                |ui: &mut egui::Ui, pan_center: Pos2| {
                    // draw logic here using `pan_center`
                    // Collect the keys first to avoid borrowing issues
                    let live_data_keys: Vec<usize> = self.live_data.keys().cloned().collect();
                    for i in live_data_keys {
                        // Use get_mut for mutable access
                        if let Some(pan_item) = self.live_data.get(&i) {
                            let kind = pan_item.get_kind();
                            match kind {
                                Logicals::Gate(_) => {
                                    // Get gate world position
                                    let world_pos: Pos2 = pan_item.get_position().unwrap();

                                    // Convert to screen-local position
                                    let screen_pos = world_pos - pan_center.to_vec2();

                                    //offset by halfsize of the widget
                                    let half_size = egui::vec2(50.0, 30.0);
                                    let screen_pos = screen_pos - half_size;

                                    // Place the widget at the screen position
                                    let rect = egui::Rect::from_min_size(
                                        screen_pos,
                                        egui::vec2(100.0, 60.0),
                                    ); // customize size
                                    let builder = UiBuilder::new().max_rect(rect);

                                    let r= ui.scope_builder(builder, |ui| {
                                        let response = pan_item.show(
                                            ui,
                                            &mut self.click_item,
                                            &self.live_data,
                                        );
                                        if response.drag_started()
                                            && ui.input(|i| !i.key_down(egui::Key::Space))
                                        {
                                            self.dragging_gate = Some(i);
                                        }

                                        if response.drag_stopped() {
                                            self.dragging_gate = None;
                                        }
                                    })
                                    .response;

                                    if r.clicked(){
                                        //if the gate was a pulse gate, toggle its state
                                        self.clicked_gate = Some(i);
                                    }
                                    r
                                }
                                Logicals::Wire => {
                                    // Get wire world position
                                    let wire = pan_item.as_any().downcast_ref::<Wire>().unwrap();
                                    //since the wire is a line with 2 points we need to offset both points by the pan center
                                    let p1 = wire.line.p1 - pan_center.to_vec2();
                                    let p2 = wire.line.p2 - pan_center.to_vec2();

                                    //create containing rect for the wire
                                    let rect = egui::Rect::from_min_max(p1, p2);
                                    let builder = UiBuilder::new().max_rect(rect);


                                    ui.scope_builder(builder, |ui| {
                                        pan_item.show(ui, &mut self.click_item, &self.live_data);
                                    })
                                    .response
                                }
                                Logicals::IO(_) => {
                                    ui.scope_builder(UiBuilder::new(), |_ui| {}).response //does nothing
                                }
                            };
                        }
                    }
                    self.pan_center = pan_center; // update AFTER the widget runs

                    if self.dragging_gate.is_some() {
                        // If dragging a gate, update its position
                        if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                            if let Some(pan_area_rect) = self.pan_area_rect {
                                if pan_area_rect.contains(pointer_pos) {
                                    // Update the position of the dragging gate
                                    if let Some(index) = self.dragging_gate {
                                        if let Some(gate) = self.live_data.get_mut(&index) {
                                            gate.set_position(pointer_pos + self.pan_center.to_vec2())
                                                .unwrap();
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if self.clicked_gate.is_some() {
                        // If a gate was clicked, toggle its state
                        if let Some(index) = self.clicked_gate {
                            if let Some(gate) = self.live_data.get_mut(&index) {
                                gate.click_on();
                            }
                        }
                        self.clicked_gate = None; // reset after clicking
                    }
                },
            ));

            //refresh all wire positions
            self.update_wires(ui);
        });



        assert!(self.clicked_gate.is_none(), "clicked_gate should be None after every frame");
    }
}
