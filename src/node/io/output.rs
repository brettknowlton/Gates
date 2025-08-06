

use super::*;

use std::collections::HashMap;
use std::error::Error;
use std::hash::Hash;

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Output {
    pub id: usize,
    pub index: usize, // index of the output in the parent gate

    pub name: Option<String>,
    pub parent_id: Option<usize>, // Optional parent gate, if this output belongs to a gate
    pub signal: bool,

    pub out_wire_ids: Vec<usize>, // outputs may have as many wires as they want
}

impl Output {
    pub fn new(parent_id: usize, index: usize) -> Self {
        let n = MyApp::next_id();
        Output {
            id: n,
            parent_id: Some(parent_id),
            name: None,
            index,
            signal: false,

            out_wire_ids: Vec::new(), // Initialize with an empty vector
        }
    }

    pub fn name_output(&mut self, name: String) {
        self.name = Some(name);
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
                    //search parent.ins for which output this is
                    let index = gp.outs.iter().position(|(i, _)| *i == self.id).unwrap_or(0);
                    let spacing = 30.0; // Vertical spacing between outputs

                    let y_offset = (index as f32 - (gp.n_out as f32 - 1.0) / 2.0) * spacing;

                    Ok(Pos2 {
                        x: pos.x + 50.0, // Offset from the gate's position
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

impl Logical for Output {
    fn tick(&mut self, ins: HashMap<usize, bool>) -> Result<HashMap<usize, bool>, Box<dyn Error>> {
        //output is updated by the gate it belongs to so just return a single output with the current signal state
        if ins.len() != 1 {
            return Err("Output can only have one in signal".into());
        }
        //check the signal in self.outs
        self.signal = ins.values().next().cloned().unwrap_or(false);

        if self.out_wire_ids.is_empty() {
            println!("Output with ID {} has no wires connected, returning empty signal map", self.id);
            return Ok(HashMap::new()); // If no wires, return nothing
        }

        let mut out_wire_signals = HashMap::new();
        //for every wire connected to this output, return the signal
        out_wire_signals.extend(self.out_wire_ids.iter().filter_map(|wire_id| {
            Some((*wire_id, self.signal)) // Assuming each wire connected to this output carries the same signal
        }));

        Ok(out_wire_signals)
    }


    fn get_id(&self) -> usize {
        self.id
    }


    fn get_kind(&self) -> LogicalKind {
        LogicalKind::IO(IOKind::Output)
    }

    fn set_position(&mut self, _pos: Pos2) -> Result<(), Box<dyn std::error::Error>> {
        print!("Setting position for Output is not allowed, set parent gate position instead");
        // Outputs do not have a position, so we return an error
        Err(Box::new(InvalidOperationError))
    }

    fn get_position(&self) -> Result<Pos2, Box<(dyn Error + 'static)>> {
        print!("Parent gate not found, use get_position with live data");
        Err(Box::new(InvalidOperationError))
    }

    fn show(
        &self,
        ui: &mut Ui,
        sender: Sender<UiEvent>,
        _live_data: &HashMap<usize, Box<dyn Logical>>,
    ) -> Response {
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            let button_color: Color32;
            if self.signal {
                button_color = HI_COLOR;
            } else {
                button_color = LO_COLOR;
            }

            let btn = Button::new(">")
                .fill(button_color)
                .min_size(vec2(18.0, 18.0));

            let mouse_pos = ui.ctx().input(|i| i.pointer.hover_pos()).unwrap_or_default();
            let response= ui.add(btn);
            if response.clicked_by(PointerButton::Primary) {
                sender
                    .try_send(UiEvent::ClickedIO(self.id, mouse_pos, true))
                    .unwrap_or_else(|_| {
                        println!("Failed to send ClickedIO event");
                    });
            }else if response.clicked_by(PointerButton::Secondary) {
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
