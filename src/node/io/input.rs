use super::*;

use std::collections::HashMap;

use crate::{gate::GridVec2, node::*};

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)]
pub struct Input {
    pub id: usize,
    pub index: usize, // index of the input in the parent gate
    pub name: Option<String>,
    pub parent_id: Option<usize>, // Optional parent gate, if this output belongs to a gate
    pub signal: bool,

    pub source_wire_id: Option<usize>, //inputs can only have one wire connected
    pub position: GridVec2,
}

impl Input {
    pub fn new(parent_id: usize, index: usize) -> Self {
        let n = MyApp::next_id();
        Input {
            id: n,
            index,
            name: None,
            parent_id: Some(parent_id), // Optional parent gate, if this input belongs to a gate
            signal: false,

            source_wire_id: None, //source wire id is optional, as inputs can be wall-mounted
            position: GridVec2::default(), // Initialize with a default position
        }
    }

    pub fn from_gate(gate: &Gate, n_index: usize) -> Self {
        //assert this is a light gate
        assert!(
            gate.get_kind().is_primitive_kind(PrimitiveKind::TOGGLE),
            "Output can only be created from a light gate"
        );
        //create an output from a gate, this is used when creating chips
        // Outputs in gates are always at index 0
        // and have the same ID as the gate

        Input {
            id: gate.get_id(),
            index: n_index, // Outputs in gates are always at index 0
            name: Some(gate.name.clone()),
            parent_id: Some(gate.get_id()),
            signal: false,
            source_wire_id: None, // Initialize with None
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
                    let spacing = 30.0; // Vertical spacing between inputs

                    let y_offset = (self.index as f32 - (gp.n_in as f32 - 1.0) / 2.0) * spacing;

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
                    Err(Box::new(InvalidOperationError(
                        "Parent could not be downcast to a Gate".to_string(),
                    )))
                }
            } else {
                Err(Box::new(InvalidOperationError("Parent gate does not exist on this input, Operation is not allowed, implement wall mounted inputs".into())))
            }
        } else {
            Err(Box::new(InvalidOperationError("Parent gate does not exist on this output, Operation is not allowed, implement wall mounted outputs".into())))
        }
    }
}

impl Logical for Input {
    fn tick(&mut self, ins: HashMap<usize, bool>) -> Result<HashMap<usize, bool>, Box<dyn Error>> {
        //in an input's wire is not connected it's signal will always be false
        if ins.len() != 1 {
            return Err("Inputs must have exactly one wire connected".into());
        }

        if let Some((_, signal)) = ins.iter().next() {
            self.signal = *signal;
        } else {
            self.signal = false;
        }
        //if input is provided, set the signal to the input's value
        Ok(HashMap::new())
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn get_kind(&self) -> LogicalKind {
        LogicalKind::IO(IOKind::Input)
    }

    fn set_position(&mut self, _pos: Pos2) -> Result<(), Box<dyn Error>> {
        // Inputs do not have a position, so we return an error
        Err(Box::new(InvalidOperationError(
            "Setting position for Input directly is not allowed, use enum Logicals to match and set parent gate position instead".to_string(),
        )))
    }

    fn get_position(&self) -> Result<Pos2, Box<(dyn Error + 'static)>> {
        Err(Box::new(InvalidOperationError(
            "Getting position for Input directly is not allowed, use enum Logicals to match and set parent gate position instead".to_string(),
        )))
    }

    fn show(
        &self,
        ui: &mut Ui,
        sender: Sender<UiEvent>,
        _live_data: &HashMap<usize, Box<dyn Logical>>,
        colors: &HashMap<String, Color32>,
    ) -> Response {
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            let button_color = if self.signal {
                colors
                    .get(HI_SIGNAL_COLOR)
                    .cloned()
                    .unwrap_or(Color32::from_rgb(0, 255, 0))
            } else {
                colors
                    .get(LO_SIGNAL_COLOR)
                    .cloned()
                    .unwrap_or(Color32::from_rgb(255, 0, 0))
            };

            let btn = Button::new("<")
                .fill(button_color)
                .min_size(vec2(18.0, 18.0));
            let response = ui.add(btn);
            let mouse_pos = ui
                .ctx()
                .input(|i| i.pointer.hover_pos().unwrap_or_default());

            if response.clicked_by(PointerButton::Primary) {
                // Handle secondary click (right-click)
                sender
                    .try_send(UiEvent::ClickedIO(self.id, mouse_pos, true))
                    .unwrap_or_else(|_| {
                        println!("Failed to send ClickedIO event");
                    });
            } else if response.clicked_by(PointerButton::Secondary) {
                // Handle secondary click (right-click)
                sender
                    .try_send(UiEvent::ClickedIO(self.id, mouse_pos, false))
                    .unwrap_or_else(|_| {
                        println!("Failed to send ClickedIO event");
                    });
            }
        })
        .response
    }
}
