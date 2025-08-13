use super::node::*;
mod ui_util;
use crossbeam::channel::{Receiver, Sender};
pub use ui_util::ClickItem;

mod pan_area;
use pan_area::PanArea;

mod data;
pub use data::*;

use std::collections::HashMap;
use std::path::PathBuf;

use eframe::{
    self,
    egui::{Align, Align2, Context, Popup, PopupCloseBehavior, Pos2, RectAlign, Ui, UiBuilder},
    *,
};

use egui_dnd::dnd;
use serde;

const TITLE_BAR_HEIGHT: f32 = 30.0;
const SIDE_PANEL_WIDTH: f32 = 200.0;

static mut NEXT_ID: usize = 0; // static variable to generate unique ids for gates and wires

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct MyApp {
    #[serde(skip)]
    data: Data,

    files_loaded: bool,
    theme_set: bool,

    current_chip: Option<PathBuf>,

    pan_center: Pos2,
    pan_area_rect: Option<egui::Rect>,


    #[serde(skip)]
    trying_save: bool,

    pub dragging_gate: Option<usize>,
    pub dragging_kind: Option<LogicalKind>, // kind of primitive we are dragging, if any


    pub holding_wire: Option<usize>, //id of the wire we are currently holding
    #[serde(skip)]
    pub event_sender: Sender<UiEvent>,
    #[serde(skip)]
    pub event_receiver: Receiver<UiEvent>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub enum UiEvent {
    ClickedGate(usize, Pos2, bool), // id of the gate that was clicked, its position, and if it was a primary click
    ClickedWire(usize, Pos2, bool), // id of the wire that was clicked, its position, and if it was a primary click
    ClickedIO(usize, Pos2, bool), // id of clicked input or output, its position, and if it was a primary click
}

impl Default for MyApp {
    fn default() -> Self {
        let (event_sender, event_receiver) = crossbeam::channel::unbounded();
        Self {
            data: Data::new(),
            // Example stuff:
            files_loaded: false,
            theme_set: false,

            current_chip: None,

            pan_center: Pos2::new(0.0, 0.0),
            pan_area_rect: None,

            trying_save: false,

            dragging_gate: None,
            dragging_kind: None, // No primitive kind being dragged initially
            holding_wire: None,  // No wire being held initially

            event_sender,
            event_receiver,
        }
    }
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (event_sender, event_receiver) = crossbeam::channel::unbounded();
        let mut new: Self;
        if let Some(_) = cc.storage {
            new = Default::default();
            // eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            new = Default::default();
        }

        new.event_sender = event_sender;
        new.event_receiver = event_receiver;
        new.data.load_app_data();
        new
    }

    

    pub fn set_theme(&mut self, ctx: &Context, theme: &SkeletonTheme) {
        theme.apply(ctx);
        self.data.color_values = theme.colors.clone();
    }

    

    pub fn next_id() -> usize {
        //generate new id incrementally from a static variable
        let id = unsafe { NEXT_ID };
        unsafe { NEXT_ID += 1 };
        id
    }

    fn update_wire_positions(&mut self, ui: &mut Ui, pan_center: Pos2) {
        //loop live data and collect all inputs and outputs into one HashMap and Wires into another
        // iterate all gates' inputs and outputs and collect their (id, positions)
        let io_positions: HashMap<usize, Pos2> = self.data
            .live_data
            .iter()
            .filter_map(|(id, item)| {
                if let Some(input) = item.as_any().downcast_ref::<Input>() {
                    Some((*id, input.get_position(&self.data.live_data).unwrap()))
                } else if let Some(output) = item.as_any().downcast_ref::<Output>() {
                    Some((*id, output.get_position(&self.data.live_data).unwrap()))
                } else {
                    None
                }
            })
            .collect();
        // println!("Found {} gates", gates.len());

        // Now iterate mutably through each IO item, update all connected wires using only the data collected above
        for (_id, wire) in self.data.live_data.iter_mut() {
            if let Some(w) = wire.as_any_mut().downcast_mut::<Wire>() {
                if w.id == self.holding_wire.unwrap_or(usize::MAX) {
                    // if we are looking at the currently held wire:
                    // set p2 to the cursor position
                    if let Some(cursor_pos) = ui.input(|i| i.pointer.hover_pos()) {
                        w.set_p2(cursor_pos);
                    }
                    //set p1 its source input's position
                    //offset postion with pan area
                    if let Some(source_pos) = io_positions.get(&w.source_id) {
                        let source_pos_moved = *source_pos - pan_center.to_vec2();
                        w.set_p1(source_pos_moved);
                    }
                }

                if w.connected {
                    if let Some(source_pos) = io_positions.get(&w.source_id) {
                        //offset postion with pan area
                        let source_pos_moved = *source_pos - pan_center.to_vec2();
                        let mut dest_pos_moved = w
                            .dest
                            .and_then(|dest_id| io_positions.get(&dest_id).cloned())
                            .unwrap_or(source_pos_moved);
                        dest_pos_moved = dest_pos_moved - pan_center.to_vec2();
                        w.set_positions(source_pos_moved, dest_pos_moved);
                    }
                }
            }
        }
    }

    fn apply_ui_events(&mut self) {
        // Process UI events from the receiver
        let mut queued_removal_id: Option<usize> = None;

        if let Ok(clicked) = self.event_receiver.try_recv() {
            // an output was clicked, so we want to create a wire if we are not currently holding a wire
            //lookup the type of the clicked IO by its id in the live_data map
            match clicked {
                UiEvent::ClickedGate(id, _, true) => {
                    // If a gate was clicked, toggle its state
                    if let Some(item) = self.data.live_data.get_mut(&id) {
                        if let Some(gate) = item.as_any_mut().downcast_mut::<Gate>() {
                            println!("Clicked on Gate: {:?}", id);
                            gate.click_on();
                        }
                    }
                }
                UiEvent::ClickedGate(_id, _, false) => {
                    // If a gate was clicked with a secondary click, show its context menu
                    // if let Some(item) = self.live_data.get_mut(&id) {
                    //     if let Some(gate) = item.as_any_mut().downcast_mut::<Gate>() {
                    //         println!("Right-clicked on Gate: {:?}", id);
                    //         gate.show_context_menu();
                    //     }
                    // }
                }
                UiEvent::ClickedIO(id, pos, true) => {
                    //primary click on an IO item
                    let kind = self.data.live_data.get(&id).unwrap().get_kind();

                    match kind {
                        LogicalKind::IO(IOKind::Input) => {
                            println!("Left-Clicked on Input: {:?}", kind);
                            //check if this input already has a wire connected
                            let in_wire_id: Option<usize> = self.data
                                .live_data
                                .get(&id)
                                .unwrap()
                                .as_any()
                                .downcast_ref::<Input>()
                                .unwrap()
                                .source_wire_id;

                            if in_wire_id.is_none() {
                                //if this input has no wire connected
                                if let Some(wire_id) = self.holding_wire.take() {
                                    //connect the wire to the input
                                    println!("Connecting wire to input: {:?}", kind);
                                    self.holding_wire = None;
                                    let wire = self.data
                                        .live_data
                                        .get_mut(&wire_id)
                                        .unwrap()
                                        .as_any_mut()
                                        .downcast_mut::<Wire>()
                                        .unwrap();
                                    //set wire's p2 to the clicked position
                                    // and set the wires destination to the input's id
                                    // wire.set_p2(clicked_io.screen_position);
                                    wire.dest = Some(id);
                                    wire.connected = true; // mark the wire as connected

                                    self.data.live_data
                                        .get_mut(&id)
                                        .unwrap()
                                        .as_any_mut()
                                        .downcast_mut::<Input>()
                                        .unwrap()
                                        .source_wire_id = Some(wire_id);
                                } else {
                                    //this input has a wire do nothing, as inputs may only
                                    println!(
                                        "Input: {:?} already has a wire connected, cannot connect another wire",
                                        id
                                    );
                                }
                            }
                        }
                        LogicalKind::IO(IOKind::Output) => {
                            // println!("Left-Clicked on Output: {:?}", id);
                            if self.holding_wire.is_none() {
                                // println!("Creating wire from clicked IO: {:?}", id);
                                let new_wire = Wire::from_io(id, pos);

                                self.data.live_data
                                    .get_mut(&id)
                                    .unwrap()
                                    .as_any_mut()
                                    .downcast_mut::<Output>()
                                    .unwrap()
                                    .out_wire_ids
                                    .push(new_wire.id);

                                self.holding_wire = Some(new_wire.id);
                                self.data.live_data.insert(new_wire.id, new_wire);
                            } else {
                                let wire_id = self.holding_wire.take().unwrap();
                                //reconnect the wire using this output as the new source
                                let old_output = self
                                    .data.live_data
                                    .get_mut(&wire_id)
                                    .unwrap()
                                    .as_any_mut()
                                    .downcast_mut::<Output>()
                                    .unwrap();

                                old_output.out_wire_ids.retain(|&x| x != wire_id); // Remove the wire from the old output

                                if let Some(wire_id) = self.holding_wire {
                                    println!("Reconnecting wire to output: {:?}", id);
                                    let wire = self.data
                                        .live_data
                                        .get_mut(&wire_id)
                                        .unwrap()
                                        .as_any_mut()
                                        .downcast_mut::<Wire>()
                                        .unwrap();

                                    wire.source_id = id; // Set the new source to the clicked output
                                }
                            }
                        }
                        _ => {
                            println!("Clicked on unknown IO type: {:?}", id);
                        }
                    }
                }
                UiEvent::ClickedIO(id, _pos, false) => {
                    //secondary click on an IO item
                    // If an IO was clicked with a secondary click, if a wire is connected, put wire in hand

                    if let Some(item) = self.data.live_data.get_mut(&id) {
                        if let Some(input) = item.as_any_mut().downcast_mut::<Input>() {
                            //if this successfully downcasts to an Input
                            // If an input was right-clicked, take the wire if it exists
                            if let Some(wire_id) = input.source_wire_id {
                                // If the input has a wire connected, take it
                                input.source_wire_id = None; // Disconnect the wire from the input
                                self.holding_wire = Some(wire_id);
                                println!("Taking wire from Input: {:?}", id);
                                //set the wires dest to none
                                if let Some(wire) = self.data.live_data.get_mut(&wire_id) {
                                    if let Some(wire) = wire.as_any_mut().downcast_mut::<Wire>() {
                                        println!("Wire taken from Input: {:?}", id);
                                        wire.dest = None; // Disconnect the wire from the input
                                        wire.connected = false; // mark the wire as disconnected
                                    }
                                }
                            } else {
                                println!("Input: {:?} has no wire connected", id);
                            }
                        } else if let Some(output) = item.as_any_mut().downcast_mut::<Output>() {
                            //if an output was right-clicked, remove the current held wire from its out_wire_ids
                            if let Some(wire_id) = self.holding_wire {
                                output.out_wire_ids.retain(|&x| x != wire_id);
                                self.holding_wire = None;
                                queued_removal_id = Some(wire_id);
                                println!("Released wire from Output: {:?}", id);
                            } else {
                                println!(
                                    "Output: {:?} no changes were made because no wire is being held",
                                    id
                                );
                            }
                        }
                    }
                }
                UiEvent::ClickedWire(_, _, _) => {}
            }
        }
        // If we have a queued removal id, remove the item from live
        if let Some(wire_id) = queued_removal_id {
            if let Some(mut item) = self.data.live_data.remove(&wire_id) {
                if let Some(wire) = item.as_any_mut().downcast_mut::<Wire>() {
                    // Remove the wire from any connected inputs
                    if let Some(input_id) = wire.dest {
                        if let Some(input) = self.data.live_data.get_mut(&input_id) {
                            if let Some(input) = input.as_any_mut().downcast_mut::<Input>() {
                                input.source_wire_id = None; // Disconnect the wire from the input
                            }
                        }
                    }
                    // Remove the wire from any connected outputs
                    if let Some(item2) = self.data.live_data.get_mut(&wire.source_id) {
                        if let Some(output) = item2.as_any_mut().downcast_mut::<Output>() {
                            output.out_wire_ids.retain(|&x| x != wire_id); // Remove the wire from the output
                        }
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
        let theme = self.data.available_themes.get("fennec.css").unwrap().clone();
        if !self.theme_set {
            self.set_theme(ctx, &theme);
            self.theme_set = true;
        }

        // determine outputs for all logicals based on their inputs and their TERM
        self.apply_ui_events();
        self.data.update_logicals(ctx);

        egui::SidePanel::left("Tools").show(ctx, |ui| {
            ui.set_max_width(SIDE_PANEL_WIDTH);
            ui.vertical_centered_justified(|ui| {
                //display title "Saved Gates" 3/4 the width of the panel, bold, and centered
                ui.heading("Saved Chips");
                //display all saved gates in a vertical list
                // Add a button to create a new gate
                if ui.button("New Chip").clicked() {
                    let new_chip = ChipDefenition::create_blank_chip("New Chip".to_string());
                    //would you like to save the current chip etc... todo!
                };

                //two "columns" first 80% th width for chip name, second 20% width for trash icon
                let mut idx = 0;
                let mut queue_rem: Option<usize> = None;

                for g in &self.data.saved_chips {
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
                            self.data.save_to_chip_file();
                        }
                        idx += 1;
                    });
                }
                
                // Remove the gate from the saved gates
                if queue_rem.is_some() {
                    self.data.saved_chips.remove(idx - 1);
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

                let mut next_themes = Vec::new();
                let themes = self.data.available_themes.clone();

                ui.menu_button("Themes", |ui| {
                    next_themes = themes
                        .iter()
                        .filter_map(|(name, x)| {
                            ui.add_space(16.0);
                            if ui.button(name).clicked() {
                                Some(x)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                });
                if let Some(theme) = next_themes.first() {
                    self.set_theme(ctx, theme);
                }
            });
        });

        egui::TopBottomPanel::top("Primitive Library").show(ctx, |ui| {
            ui.set_max_height(150.);
            ui.horizontal(|ui| {
                ui.set_min_height(15.);
                ui.label("Primitive Gates");
            });

            // println!("Primitive gates: {:?}", self.primitive_gates);
            ui.horizontal_centered(|ui| {
                dnd(ui, "Primitive").show(
                    &mut self.data.prim_templates.iter(),
                    |ui, item, handle, _state| {
                        handle.ui(ui, |ui| {
                            let w = ui.add(item.make_toolbox_widget());
                            if w.is_pointer_button_down_on() {
                                self.dragging_kind = Some(item.kind.get_logical_kind());
                            } else if ui.input(|i| i.pointer.any_released()) {
                                if let Some(kind) = &self.dragging_kind {
                                    // Check if pointer is over the PanArea
                                    if let Some(pointer_pos) = ctx.pointer_hover_pos() {
                                        if let Some(pan_area_rect) = self.pan_area_rect {
                                            if pan_area_rect.contains(pointer_pos) {
                                                println!("Pointer is over PanArea, adding gate");
                                                let world_pos =
                                                    pointer_pos + self.pan_center.to_vec2();
                                                let mut gate = Gate::create_gate_from_template(
                                                    kind.as_gate().unwrap(),
                                                    world_pos,
                                                );

                                                gate.create_io(&mut self.data.live_data);

                                                self.data.live_data.insert(
                                                    // Create a new gate at the world position
                                                    gate.id,
                                                    Box::new(gate),
                                                );

                                                println!(
                                                    "Added new gate: {:?}",
                                                    self.dragging_kind
                                                );
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
                    },
                );
            });
            ui.horizontal(|ui| {
                ui.set_min_height(15.);
                //create a left-justified button to clear the board

                //left half rect:
                let left_half_rect = ui.available_rect_before_wrap();
                let left_half_rect = egui::Rect::from_min_size(
                    left_half_rect.min,
                    egui::vec2(left_half_rect.width() / 2.0, left_half_rect.height()),
                );

                ui.scope_builder(UiBuilder::new().max_rect(left_half_rect), |ui| {
                    ui.with_layout(Layout::top_down(Align::Min), |ui| {
                        ui.vertical(|ui| {
                            let align = RectAlign {
                                parent: Align2::LEFT_BOTTOM,
                                child: Align2::LEFT_TOP,
                            };
                            let gap = 4.0;
                            let close_behavior = PopupCloseBehavior::CloseOnClickOutside;

                            let response = ui.button("Save Board");
                            if response.clicked(){
                                self.trying_save = true;
                                self.dragging_gate = None;
                                self.holding_wire = None;
                            }

                            
                            Popup::menu(&response).open(self.trying_save)
                                .gap(gap)
                                .align(align)
                                .close_behavior(close_behavior)
                                .show(|ui| { 
                                    ui.label("Are you sure?\n This will save a chip and clear the current board.");
                                    if ui.button("Yes").clicked() {
                                        self.trying_save = false;
                                        // Clear the live data
                                        self.data.live_data.clear();
                                        self.pan_center = Pos2::new(0.0, 0.0);
                                        self.dragging_gate = None;
                                        self.holding_wire = None;


                                        let new_chip = ChipDefenition::from_live_data(
                                            &self.data.live_data,
                                            "New Chip".to_string()
                                        );


                                        println!("Cleared the board");
                                    }
                                    if ui.button("No").clicked() {
                                        self.trying_save = false;
                                        // Clear the live data
                                        self.dragging_gate = None;
                                        self.holding_wire = None;
                                        println!("Save cancelled");
                                    }

                                })
                        ;
                        });
                    })
                });

                //right half rect
                let right_half_rect = ui.available_rect_before_wrap();
                let right_half_rect = egui::Rect::from_min_size(
                    right_half_rect.min + egui::vec2(left_half_rect.width(), 0.0),
                    egui::vec2(right_half_rect.width() / 2.0, right_half_rect.height()),
                );

                ui.scope_builder(UiBuilder::new().max_rect(right_half_rect), |ui| {
                    ui.with_layout(Layout::top_down(Align::Max), |ui| {
                        ui.vertical(|ui| {
                            if ui.button("Clear Board").clicked() {
                                // Clear the live data
                                self.data.live_data.clear();
                                self.pan_center = Pos2::new(0.0, 0.0);
                                self.dragging_gate = None;
                                self.holding_wire = None;
                                println!("Cleared the board");
                            }
                        });
                    })
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
                    // Pre-process: collect gate output positions for wire updates
                    // let mut gate_outputs = std::collections::HashMap::new();
                    // for (id, item) in self.live_data.iter() {
                    //     if let Some(gate) = item.as_any().downcast_ref::<Gate>() {
                    //         for (id, output) in gate.outs.iter() {
                    //             let output_pos = output.get_position_with_parent(gate.get_position(), gate.n_out);
                    //             gate_outputs.insert((gate.id, output_index), output_pos);
                    //         }
                    //     }
                    // }

                    // draw logic here using `pan_center`
                    // Collect the keys first to avoid borrowing issues
                    let live_data_keys: Vec<usize> = self.data.live_data.keys().cloned().collect();
                    for key in live_data_keys {
                        // Use get_mut for mutable access
                        if let Some(pan_item) = self.data.live_data.get(&key) {
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
                                            self.event_sender.clone(),
                                            &self.data.live_data,
                                            &self.data.color_values,
                                        );
                                        if response.drag_started()
                                            && ui.input(|i| !i.key_down(egui::Key::Space))
                                        {
                                            self.dragging_gate = Some(key);
                                        }

                                        if response.drag_stopped() {
                                            self.dragging_gate = None;
                                        }

                                        if response.clicked() {
                                            //if the item was a gate (should always be), set the clicked_gate to this id
                                            self.event_sender
                                                .try_send(UiEvent::ClickedGate(
                                                    key,
                                                    ui.ctx().input(|i| {
                                                        i.pointer.hover_pos().unwrap_or_default()
                                                    }),
                                                    true,
                                                ))
                                                .unwrap();
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
                                        pan_item.show(
                                            ui,
                                            self.event_sender.clone(),
                                            &self.data.live_data,
                                            &self.data.color_values,
                                        );
                                    })
                                    .response
                                }
                                LogicalKind::IO(_) => {
                                    ui.scope_builder(UiBuilder::new(), |_ui| {}).response //does nothing
                                }
                                LogicalKind::Chip(_) => {
                                    // Show the chip
                                    ui.scope_builder(UiBuilder::new(), |ui| {
                                        pan_item.show(
                                            ui,
                                            self.event_sender.clone(),
                                            &self.data.live_data,
                                            &self.data.color_values,
                                        );
                                    })
                                    .response
                                }
                            };
                        }
                    }
                    self.pan_center = pan_center; // update AFTER the widget runs

                    if let Some(gate_index)= self.dragging_gate {
                        // If dragging a gate, update its position
                        if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                            if let Some(pan_area_rect) = self.pan_area_rect {
                                if pan_area_rect.contains(pointer_pos) {
                                    // Update the position of the dragging gate
                                        if let Some(gate) = self.data.live_data.get_mut(&gate_index) {
                                            gate.set_position(
                                                pointer_pos + self.pan_center.to_vec2(),
                                            )
                                            .unwrap();
                                        }
                                }
                            }
                        }
                    }
                },
            ));
            //refresh all wire positions
            self.update_wire_positions(ui, self.pan_center);

        });
    }
}
