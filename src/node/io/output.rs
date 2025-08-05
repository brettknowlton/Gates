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
        if ins.len() > 1 {
            return Err(Box::new(InvalidOperationError));
        }
        if ins.is_empty() {
            self.signal = false; // If no inputs, signal is false
            return Ok(HashMap::from([(self.id, false)])); // If no inputs, return false
        }
        self.signal = ins.get(&self.id).cloned().unwrap_or(false);

        if self.out_wire_ids.is_empty() {
            return Ok(HashMap::new()); // If no wires, return the signal
        }
        let mut out_signals = HashMap::new();
        //for every wire connected to this output, return the signal
        for wire_id in &self.out_wire_ids {
            if let Some(wire_signal) = ins.get(wire_id) {
                out_signals.insert(*wire_id, *wire_signal);
            } else {
                return Err("Output signal not found".into());
            }
        }

        Ok(out_signals)
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
        click_item: &mut Option<ClickItem>,
        live_data: &HashMap<usize, Box<dyn Logical>>,
    ) -> Response {
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            let button_color: Color32; // Default color
            if self.signal {
                button_color = Color32::GREEN;
                println!("Showing output: {}, has signal:: {}", self.id, self.signal);
            } else {
                button_color = Color32::RED;
                println!("Output: {}, has signal:: {}", self.id, self.signal);
            }

            let btn = Button::new(">")
                .fill(button_color)
                .min_size(vec2(18.0, 18.0));

            if ui.add(btn).clicked() {
                *click_item = Some(ClickItem {
                    item_id: self.id,
                    screen_position: self.get_position(live_data).unwrap(),
                    item_type: LogicalKind::IO(IOKind::Output),
                });
            }
        })
        .response
    }
}
