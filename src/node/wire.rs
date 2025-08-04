use super::*;

use egui::{Rect, Stroke};
use crate::MyApp;


#[derive(serde::Deserialize, serde::Serialize, Default, Clone, Debug)]
pub struct WireLine {
    pub p1: Pos2,
    pub p2: Pos2,
    color: Color32,
    smoothing: bool,
}
impl WireLine {
    pub fn new(p1: Pos2, p2: Pos2, color: Color32, smoothing: bool) -> Self {
        WireLine {
            p1,
            p2,
            color,
            smoothing,
        }
    }
}

impl Hash for WireLine {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.p1.x.to_bits().hash(state);
        self.p1.y.to_bits().hash(state);
        self.p2.x.to_bits().hash(state);
        self.p2.y.to_bits().hash(state);
        self.color.hash(state);
        self.smoothing.hash(state);
    }
}

#[derive(serde::Deserialize, serde::Serialize, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Wire {
    pub id: usize,
    pub signal: bool,
    pub source_id: usize,
    pub dest: Option<usize>,

    pub connected: bool, // whether the wire is connected to an output AND an input
    pub line: WireLine,
}

impl Wire {
    fn new(source_id: usize, position: Pos2, color: Color32, smoothing: bool) -> Self {
        Wire {
            id: MyApp::next_id(),
            signal: false,
            source_id,
            dest: None,

            connected: false,
            line: WireLine::new(position, position, color, smoothing),//line begins at one point
        }
    }

    pub fn set_positions(&mut self, p1: Pos2, p2: Pos2) {
        self.line.p1 = p1;
        self.line.p2 = p2;
    }

    pub fn set_p1(&mut self, p1: Pos2) {
        self.line.p1 = p1;
    }
    pub fn set_p2(&mut self, p2: Pos2) {
        self.line.p2 = p2;
    }

    pub fn from_io(output_id: usize, position: Pos2) -> Box<Self> {
        Box::new(Wire::new(output_id, position, Color32::from_rgb(0, 0, 0), false))
    }
    pub fn on(mut self) {
        self.signal = true;
    }

    pub fn off(mut self) {
        self.signal = false
    }
}

impl Logical for Wire {
    fn tick(self) {
        if let Some(mut _out) = self.dest {
            // If the wire has a destination, signal it
        }
    }

    fn get_kind(&self) -> Logicals {
        Logicals::Wire
    }

    fn get_position(&self) -> Result<egui::Pos2, Box<(dyn Error + 'static)>> {
        Ok(self.line.p1)
    }

    fn set_position(&mut self, _pos: Pos2) -> Result<(), Box<dyn Error>> {
        Err(Box::new(InvalidOperationError))
    }

    fn show(&self, ui: &mut Ui, _on_output_click: &mut Option<ClickItem>, _live_data: &HashMap<usize, Box<dyn Logical>>) -> Response {
        let response = ui.allocate_rect(Rect::from_min_max(self.line.p1, self.line.p2), Sense::hover());

        //if wire is connected, update the line's end points to be the current source -> destination positions
        if self.connected {
            
        }
        // Draw the wire line
        ui.painter().line_segment([self.line.p1, self.line.p2], Stroke::new(LINE_THICKNESS, self.line.color));

        response
    }

}

impl Default for Wire {
    fn default() -> Self {
        Wire {
            id: MyApp::next_id(),
            source_id: 0,
            signal: false,
            dest: None,

            connected: false,
            line: WireLine::default(),
        }
    }
}