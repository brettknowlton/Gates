
use super::{Wire, Pos2, Gate};

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Output {
    pub id: usize,
    pub name: Option<String>,
    pub parent_gate_id: Option<usize>, // ID of the parent gate instead of a full clone

    pub connected: bool,
    pub signal: bool,
    pub wires: Vec<Wire>,
}

impl Output {
    pub fn new(n: usize, parent_gate: &Gate) -> Self {
        Output {
            id: n,
            parent_gate_id: Some(parent_gate.id),
            name: None,
            signal: false,
            wires: Vec::new(),
            connected: false,
        }
    }

    pub fn get_position(&self) -> Pos2 {
        // This method now requires the parent gate's current position to be passed
        // For backward compatibility, return default position
        // The caller should use get_position_with_parent instead
        Pos2 { x: 0.0, y: 0.0 }
    }
    
    pub fn get_position_with_parent(&self, parent_pos: Pos2, parent_n_out: usize) -> Pos2 {
        //if no parent, this is a wall-mounted output, so return a default position on the left wall of the PanArea
        if self.parent_gate_id.is_some() {
            // Calculate the position of the output based on the parent gate's position and the output's index
            let index = self.id; // id is an index for this gate's outputs
            let spacing = 30.0; // Vertical spacing between outputs

            let y_offset = (index as f32 - (parent_n_out as f32 - 1.0) / 2.0) * spacing;
            let p = Pos2 {
                x: parent_pos.x + 50.0, // Offset from the gate's position
                y: parent_pos.y + y_offset,
            };
            println!("Node Position: {:?}", p);
            p
        } else {
            Pos2 { x: 0.0, y: 0.0 }
        }
    }
}
