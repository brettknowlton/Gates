use super::*;

use crate::{MyApp, UiEvent};
use crossbeam::channel::Sender;
use eframe::{egui::{Rect, Stroke}};

#[derive(serde::Deserialize, serde::Serialize, Default, Clone, Debug)]
pub struct WireLine {
    pub p1: Pos2,
    pub p2: Pos2,
    smoothing: bool,
}
impl WireLine {
    pub fn new(p1: Pos2, p2: Pos2, smoothing: bool) -> Self {
        WireLine {
            p1,
            p2,
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
    fn new(source_id: usize, position: Pos2, smoothing: bool) -> Self {
        Wire {
            id: MyApp::next_id(),
            signal: false,
            source_id,
            dest: None,

            connected: false,
            line: WireLine::new(position, position, smoothing), //line begins at one point
        }
    }

    pub fn set_positions(&mut self, p1: Pos2, p2: Pos2) {
        self.line.p1 = p1;
        self.line.p2 = p2;
    }

    pub fn delete(&mut self){
        self.connected = false;
        self.dest = None;
        self.signal = false;
    }

    pub fn set_signal(&mut self, signal: bool) {
        self.signal = signal;
    }

    pub fn set_p1(&mut self, p1: Pos2) {
        self.line.p1 = p1;
    }
    pub fn set_p2(&mut self, p2: Pos2) {
        self.line.p2 = p2;
    }

    pub fn from_io(output_id: usize, position: Pos2) -> Box<Self> {
        Box::new(Wire::new(
            output_id,
            position,
            false,
        ))
    }

    pub fn on(mut self) {
        self.signal = true;
    }

    pub fn off(mut self) {
        self.signal = false
    }
}

impl Logical for Wire {
    fn tick(&mut self, _: HashMap<usize, bool>) -> Result<HashMap<usize, bool>, Box<dyn Error>> {
        Err("Wires cannot be ticked directly, they are passive holders of a signal".into())
    }

    fn get_kind(&self) -> LogicalKind {
        LogicalKind::Wire
    }

    fn get_position(&self) -> Result<Pos2, Box<(dyn Error + 'static)>> {
        Ok(self.line.p1)
    }
    fn get_id(&self) -> usize {
        self.id
    }

    fn set_position(&mut self, _pos: Pos2) -> Result<(), Box<dyn Error>> {
        Err(Box::new(InvalidOperationError))
    }

    fn show(
        &self,
        ui: &mut Ui,
        _sender: Sender<UiEvent>,
        _live_data: &HashMap<usize, Box<dyn Logical>>,
        colors: &HashMap<String, Color32>,
    ) -> Response {
        let response = ui.allocate_rect(
            Rect::from_min_max(self.line.p1, self.line.p2),
            Sense::hover(),
        );

        let color = if self.signal {
            colors.get(HI_SIGNAL_COLOR).unwrap_or(&Color32::DARK_GREEN).clone()
        } else {
            colors.get(LO_SIGNAL_COLOR).unwrap_or(&Color32::GRAY).clone()
        };
        //if wire is connected, update the line's end points to be the current source -> destination positions
        if self.connected {}
        // Draw the wire line
        ui.painter().line_segment(
            [self.line.p1, self.line.p2],
            Stroke::new(LINE_THICKNESS, color),
        );

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
