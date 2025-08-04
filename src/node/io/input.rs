use super::*;

use std::collections::HashMap;

use egui::{Align, Button, vec2};

use crate::{gate::GridVec2, node::*};

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)]
pub struct Input {
    pub id: usize,
    pub name: Option<String>,
    pub parent_id: Option<usize>, // Optional parent gate, if this output belongs to a gate
    pub signal: bool,

    pub in_wire_id: Option<usize>, //inputs can only have one wire connected
    pub position: GridVec2,
}

impl Input {
    pub fn new(parent_id: usize) -> Self {
        let n = MyApp::next_id();
        Input {
            id: n,
            name: None,
            parent_id: Some(parent_id), // Optional parent gate, if this input belongs to a gate
            signal: false,

            in_wire_id: None,
            position: GridVec2::default(), // Initialize with a default position
        }
    }

    pub fn get_position(
        &self,
        opt_data: &HashMap<usize, Box<dyn Logical>>,
    ) -> Result<Pos2, Box<dyn std::error::Error>> {
        //if no parent, this is a wall-mounted output, so return a default position on the left wall of the PanArea
        if let Some(p_id) = self.parent_id {
            let parent_gen = opt_data.get(&p_id);
            // Calculate the position of the output based on the parent gate's position and the output's index

            if let Some(parent) = parent_gen {
                if let Some(gp) = parent.as_any().downcast_ref::<Gate>() {
                    let pos = gp.get_position().unwrap();
                    //search parent.ins for which input this is
                    let index = gp.ins.iter().position(|i| *i == self.id).unwrap_or(0);
                    let spacing = 30.0; // Vertical spacing between inputs

                    let y_offset = (index as f32 - (gp.n_in as f32 - 1.0) / 2.0) * spacing;

                    Ok(Pos2 {
                        x: pos.x - 50.0, // Offset from the gate's position
                        y: pos.y + y_offset,
                    })
                } else {
                    println!("Parent could not be downcast to a Gate, Operation is not allowed");
                    println!(
                        "Accessed live_data at index {}, returns: {:?}",
                        p_id,
                        opt_data.get(&p_id).unwrap().get_kind()
                    );
                    //print first 5 elements of live_data
                    for i in 0..5 {
                        if let Some(item) = opt_data.get(&(i as usize)) {
                            println!("Before: {:?}", item.get_kind());
                        }
                    }
                    Err(Box::new(InvalidOperationError))
                }
            } else {
                println!(
                    "Parent gate does not exist on this input, Operation is not allowed, implement wall mounted inputs"
                );
                Err(Box::new(InvalidOperationError))
            }
        } else {
            println!(
                "Parent gate does not exist on this output, Operation is not allowed, implement wall mounted outputs"
            );
            Err(Box::new(InvalidOperationError))
        }
    }
}

impl Logical for Input {
    fn tick(mut self) {
        //in an input's wire is not connected it's signal will always be false
        if let Some(_parent) = &self.parent_id {
            // If the input has a parent gate, signal it
        } else {
            self.signal = false;
        }
    }

    fn get_kind(&self) -> Logicals {
        Logicals::IO(IOKind::Input)
    }

    fn set_position(&mut self, _pos: Pos2) -> Result<(), Box<dyn Error>> {
        print!("Setting position for Input is not allowed, set parent gate position instead");
        // Inputs do not have a position, so we return an error
        Err(Box::new(InvalidOperationError))
    }

    fn get_position(&self) -> Result<egui::Pos2, Box<(dyn Error + 'static)>> {
        print!("Parent gate not found, use get_position with live data");
        Err(Box::new(InvalidOperationError))
    }

    fn show(
        &self,
        ui: &mut Ui,
        on_output_click: &mut Option<ClickItem>,
        _live_data: &HashMap<usize, Box<dyn Logical>>,
    ) -> Response {
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            let button_color = if self.signal {
                Color32::GREEN
            } else {
                Color32::DARK_RED
            };

            let btn = Button::new("<")
                .fill(button_color)
                .min_size(vec2(18.0, 18.0));

            if ui.add(btn).clicked() {
                let cursor_pos = ui
                    .ctx()
                    .input(|i| i.pointer.hover_pos().unwrap_or_default());
                *on_output_click = Some(ClickItem {
                    item_id: self.id,
                    screen_position: cursor_pos,
                    item_type: Logicals::IO(IOKind::Input),
                });
            }
        })
        .response
    }
}
