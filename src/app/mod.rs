use super::node::*;
mod ui_util;
pub use ui_util::ClickItem;

mod pan_area;
use pan_area::PanArea;

mod data;
pub use data::*;

use std::collections::HashMap;
use std::path::PathBuf;

use eframe::{
    self,
    egui::{Pos2, Ui, UiBuilder},
    *,
};

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
    dragging_kind: Option<GateKind>,
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
        self.saved = data::Data::load_chips();
        println!("Loaded chips: {}", self.saved.len());

        self.prims = data::Data::load_prims();
        println!("Loaded prims: {}", self.prims.len());

        self.files_loaded = true;
        self
    }

    pub fn next_id() -> usize {
        //generate new id incrementally from a static variable
        let id = unsafe { NEXT_ID };
        unsafe { NEXT_ID += 1 };
        id
    }

    fn update_wire_positions(&mut self, ui: &mut Ui, pan_center: Pos2) {
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
                        //offset postion with pan area
                        let source_pos_moved = *source_pos - pan_center.to_vec2();
                        let mut dest_pos_moved = w
                            .dest
                            .and_then(|dest_id| gates.get(&dest_id).cloned())
                            .unwrap_or(source_pos_moved);
                        dest_pos_moved = dest_pos_moved - pan_center.to_vec2();
                        w.set_positions(source_pos_moved, dest_pos_moved);
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

    fn update_logicals(&mut self) {
        // Update the logical states of all gates and wires

        let mut pre_gates = HashMap::new();
        let mut pre_outs = HashMap::new();
        let mut pre_wires = HashMap::new();
        let mut pre_ins = HashMap::new();

        for pair in self.live_data.iter_mut() {
            let item = pair.1;

            let kind = item.get_kind();

            match kind {
                //sort and borrow into the correct vector
                LogicalKind::Gate(_) => {
                    // Get gate world position
                    if let Some(gate) = item.as_any_mut().downcast_mut::<Gate>() {
                        pre_gates.insert(pair.0, gate);
                    }
                }
                LogicalKind::IO(IOKind::Output) => {
                    if let Some(output) = item.as_any_mut().downcast_mut::<Output>() {
                        pre_outs.insert(pair.0, output);
                    }
                }
                LogicalKind::Wire => {
                    if let Some(wire) = item.as_any_mut().downcast_mut::<Wire>() {
                        pre_wires.insert(pair.0, wire);
                    }
                }
                LogicalKind::IO(IOKind::Input) => {
                    if let Some(input) = item.as_any_mut().downcast_mut::<Input>() {
                        pre_ins.insert(pair.0, input);
                    }
                }
            }
        }
        //now that we have things all separated we can tick things in order of type,
        //gates first based on relevant inputs(the ones in the gate.ins.0 vec items)

        if !pre_gates.is_empty() {
            //tick all gates

            //hashmap for collecting what we want to pass to outputs
            let mut changed_outputs = HashMap::new();

            for (id, gate) in pre_gates {
                //for every gate

                //hashmap of input ids and their values that belong to this gate
                let mut cleared_ins = HashMap::new();

                for (&input_id, input) in pre_ins.iter() {
                    //find all inputs who's parent_id matches the gate's id
                    if let Some(input_parent_id) = input.parent_id {
                        //check that input actually has a parent
                        if input_parent_id == *id {
                            //and that the parent is THIS gate
                            //if the input's parent id matches the gate's id, add it to the cleared inputs
                            cleared_ins.insert(*input_id, input.signal);
                        }
                    }
                }
                //tick the gate with the inputs and collect what we want to send to outputs

                //HashMap<usize: id of an output, bool: desired state of that output
                changed_outputs.extend(gate.tick(cleared_ins.clone()).unwrap_or(HashMap::new()));
            }

            //now we have collected a map of outputs and their desired states
            //tick every output, wire and input item in this list, this will return the same signal we pass in
            for (output_id, signal) in changed_outputs {
                //tick the output
                let mut desired_wire_changes = HashMap::new();
                if let Some(item) = pre_outs.get_mut(&output_id) {
                    println!("Ticking output with ID: {}, signal: {}", output_id, signal);
                    desired_wire_changes.extend(
                        item.tick(HashMap::from([(output_id, signal)]))
                            .unwrap_or_default(),
                    );
                }

                //hash map for tracking which wires we
                let mut desired_input_changes = HashMap::new();

                //tick every wire that has this output as its source_id
                for (_, wire) in pre_wires.iter_mut() {
                    if wire.source_id == output_id {
                        //tick the wire with the signal
                        println!("Ticking wire with ID: {}, signal: {}", wire.id, signal);
                        desired_input_changes.extend(
                            wire.tick(HashMap::from([(output_id.clone(), signal.clone())]))
                                .unwrap_or_default(),
                        );
                    }
                }

                //tick every input in requested input changes
                for (&input_id, input) in pre_ins.iter_mut() {
                    if let Some(signal) = desired_input_changes.get(input_id) {
                        //tick the input with the signal
                        println!("Ticking input with ID: {}, signal: {}", input.id, signal);
                        input
                            .tick(HashMap::from([(*input_id, *signal)]))
                            .unwrap_or_default();
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
                LogicalKind::IO(IOKind::Input) => {
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
                    } else {
                        println!("No wire to connect to input, holding_wire is None");
                    }
                }
                LogicalKind::IO(IOKind::Output) => {
                    println!("Clicked on Output: {:?}", clicked_io);
                    if self.holding_wire.is_none() {
                        println!("Creating wire from clicked IO: {:?}", clicked_io);
                        self.holding_wire = Some(Wire::from_io(
                            clicked_io.item_id,
                            clicked_io.screen_position,
                        ));
                    } else {
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

        // determine outputs for all logicals based on their inputs and their TERM
        self.update_logicals();

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
                    data::Data::save_chip(&self.live_data);
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
                            data::Data::save_chip(&self.live_data);
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
                                LogicalKind::Gate(_) => {
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
                                    let builder =
                                        UiBuilder::new().max_rect(rect).sense(Sense::click());

                                    ui.scope_builder(builder, |ui| {
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

                                        if response.clicked() {
                                            //if the item was a gate (should always be), set the clicked_gate to this id
                                            if let Some(_) =
                                                pan_item.as_any().downcast_ref::<Gate>()
                                            {
                                                self.clicked_gate = Some(i);
                                            }
                                        }
                                    })
                                    .response
                                }
                                LogicalKind::Wire => {
                                    // Get wire world position
                                    // //create containing rect for the wire
                                    // let rect = egui::Rect::from_min_max(p1, p2);
                                    let builder = UiBuilder::new(); //.max_rect(rect);

                                    ui.scope_builder(builder, |ui| {
                                        pan_item.show(ui, &mut self.click_item, &self.live_data);
                                    })
                                    .response
                                }
                                LogicalKind::IO(_) => {
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
                                            gate.set_position(
                                                pointer_pos + self.pan_center.to_vec2(),
                                            )
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
                            if let Some(item) = self.live_data.get_mut(&index) {
                                print!(
                                    "Gate was clicked: {:?}",
                                    item.as_any().downcast_ref::<Gate>()
                                );
                                if let Some(gate) = item.as_any_mut().downcast_mut::<Gate>() {
                                    gate.click_on();
                                }
                            }
                        }
                        self.clicked_gate = None; // reset after clicking
                    }
                },
            ));

            //refresh all wire positions
            self.update_wire_positions(ui, self.pan_center);
        });

        assert!(
            self.clicked_gate.is_none(),
            "clicked_gate should be None after every frame"
        );
    }
}
