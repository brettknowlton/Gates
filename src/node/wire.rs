use super::*;

use crate::{MyApp, UiEvent};
use crossbeam::channel::Sender;
use eframe::egui::{Rect, Stroke};

#[derive(serde::Deserialize, serde::Serialize, Default, Clone, Debug)]
pub struct WireLine {
    pub p1: Pos2,
    pub p2: Pos2,
    smoothing: bool,
}
impl WireLine {
    pub fn new(p1: Pos2, p2: Pos2, smoothing: bool) -> Self {
        WireLine { p1, p2, smoothing }
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
    signal: bool,

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
            line: WireLine::new(position, position, smoothing), //line begins at one point so both p1 and p2 are the same
        }
    }

    pub fn set_positions(&mut self, p1: Pos2, p2: Pos2) {
        self.line.p1 = p1;
        self.line.p2 = p2;
    }

    pub fn delete(&mut self) {
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
        Box::new(Wire::new(output_id, position, false))
    }

    pub fn on(mut self) {
        self.signal = true;
    }

    pub fn off(mut self) {
        self.signal = false
    }
}

impl Logical for Wire {
    ///Wire should take in only one input, and will output only one output IF it is connected to a destination
    ///If it is not connected, it will return nothing
    fn tick(
        &mut self,
        inputs: HashMap<usize, bool>,
    ) -> Result<HashMap<usize, bool>, Box<dyn Error>> {

        assert!(inputs.len() == 1, "Wires should only have one input");
        self.signal = *inputs.values().next().unwrap();


        self.connected = self.dest.is_some();
        if self.connected {
            let mut outputs = HashMap::new();
            outputs.insert(self.dest.unwrap(), self.signal);
            Ok(outputs)
        } else {
            Ok(HashMap::new())
        }
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

    ///WILL PANIC FOR WIRES\
    /// wire positions can not be set to a single point\
    /// use Wire::set_positions(p1, p2) instead
    fn set_position(&mut self, _pos: Pos2) -> Result<(), Box<dyn Error>> {
        eprintln!("Cannot set position for a wire");
        Err(Box::new(InvalidOperationError("Cannot set position for a wire this way \n use \"Wire::set_positions(p1,p2) instead\"".to_string())))
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
            colors
                .get(HI_SIGNAL_COLOR)
                .unwrap_or(&Color32::DARK_GREEN)
                .clone()
        } else {
            colors
                .get(LO_SIGNAL_COLOR)
                .unwrap_or(&Color32::GRAY)
                .clone()
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
