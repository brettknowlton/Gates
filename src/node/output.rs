
use super::{Wire, Pos2, Gate, Logical};

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Output {
    pub id: usize,
    pub name: Option<String>,
    pub parent: Option<Gate>, // Optional parent gate, if this output belongs to a gate

    pub connected: bool,
    pub signal: bool,
    pub wires: Vec<Wire>,
}

impl Output {
    pub fn new(n: usize, parent_gate: &Gate) -> Self {
        Output {
            id: n,
            parent: Some(parent_gate.clone()),
            name: None,
            signal: false,
            wires: Vec::new(),
            connected: false,
        }
    }

    pub fn get_position(&self) -> Pos2 {
        //if no parent, this is a wall-mounted output, so return a default position on the left wall of the PanArea
        if let Some(parent) = &self.parent {
            // Calculate the position of the output based on the parent gate's position and the output's index
            let pos = parent.get_position();
            let index = self.id; // id is an index for this gate's outputs
            let spacing = 30.0; // Vertical spacing between outputs

            let y_offset = (index as f32 - (parent.n_out as f32 - 1.0) / 2.0) * spacing;
            Pos2 {
                x: pos.x + 50.0, // Offset from the gate's position
                y: pos.y + y_offset,
            }
        }else{
            Pos2 { x: 0.0, y: 0.0 }
        }
    }
}
