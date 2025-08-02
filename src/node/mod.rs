pub mod gate;
use super::*;
pub use gate::Gate;
use serde;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

use std::error::Error;

use egui::{Color32, Direction, Layout, Pos2, Response, Sense, Ui, Widget};

mod output;
pub use output::Output;

const LINE_THICKNESS: f32 = 2.0;

pub enum Logicals {
    Gate(GateType),
    Primitive(PrimitiveKind),
    Wire,
}

#[derive(Debug)]
struct InvalidOperationError;
impl Error for InvalidOperationError {}

impl Display for InvalidOperationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Cannot set position for this type")
    }
}

pub trait Logical {
    fn tick(self);
    fn get_position(&self) -> Pos2;
    fn set_position(&mut self, pos: Pos2) -> Result<(), Box<dyn Error>>;

    fn get_kind(&self) -> Logicals {
        Logicals::Gate(GateType::None)
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
pub enum GateType {
    #[default]
    None,
    Primitive(PrimitiveKind),
    Wire,
    Custom,
}

impl Display for GateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateType::None => write!(f, "None"),
            GateType::Primitive(kind) => write!(f, "{}", kind),
            GateType::Custom => write!(f, "Custom"),
            GateType::Wire => write!(f, "Wire"),
        }
    }
}

impl GateType {
    pub fn lookup_kind(name: &str) -> GateType {
        match name {
            "TOGGLE" => GateType::Primitive(PrimitiveKind::TOGGLE),
            "LIGHT" => GateType::Primitive(PrimitiveKind::LIGHT),
            "BUFFER" => GateType::Primitive(PrimitiveKind::BUFFER),
            "NOT" => GateType::Primitive(PrimitiveKind::NOT),
            "OR" => GateType::Primitive(PrimitiveKind::OR),
            "AND" => GateType::Primitive(PrimitiveKind::AND),
            _ => GateType::None,
        }
    }

    pub fn is_primitive(&self) -> bool {
        matches!(self, GateType::Primitive(_))
    }

    pub fn primitive_kind(&self) -> Option<PrimitiveKind> {
        if let GateType::Primitive(kind) = self {
            Some(kind.clone())
        } else {
            None
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
pub enum PrimitiveKind {
    #[default]
    None,
    TOGGLE,
    LIGHT,
    BUFFER,
    NOT,
    OR,
    AND,
}

impl Display for PrimitiveKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveKind::None => write!(f, "None"),
            PrimitiveKind::TOGGLE => write!(f, "TOGGLE"),
            PrimitiveKind::LIGHT => write!(f, "LIGHT"),
            PrimitiveKind::BUFFER => write!(f, "BUFFER"),
            PrimitiveKind::NOT => write!(f, "NOT"),
            PrimitiveKind::OR => write!(f, "OR"),
            PrimitiveKind::AND => write!(f, "AND"),
        }
    }
}

impl Widget for PrimitiveKind {
    fn ui(self, ui: &mut Ui) -> Response {
        let r = ui.add_enabled_ui(false, |ui| {
            ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                ui.label(self.to_string());
            });
        });
        r.response
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Primitive {
    pub label: String,
    pub kind: GateType,
    pub n_ins: usize,
    pub n_outs: usize,
}

impl Primitive {
    pub fn kind_as_str(self) -> String {
        self.kind.to_string().clone()
    }

    pub fn from_values(label: &str, num_inputs: usize, num_outputs: usize) -> Primitive {
        let kind: GateType;
        match label {
            "TOGGLE" => {
                kind = GateType::Primitive(PrimitiveKind::TOGGLE);
            }
            "LIGHT" => {
                kind = GateType::Primitive(PrimitiveKind::LIGHT);
            }
            "BUFFER" => {
                kind = GateType::Primitive(PrimitiveKind::BUFFER);
            }
            "NOT" => {
                kind = GateType::Primitive(PrimitiveKind::NOT);
            }
            "OR" => {
                kind = GateType::Primitive(PrimitiveKind::OR);
            }
            "AND" => {
                kind = GateType::Primitive(PrimitiveKind::AND);
            }
            _ => {
                kind = GateType::Primitive(PrimitiveKind::None);
            }
        }
        let var = Primitive {
            label: label.to_string(),
            n_ins: num_inputs,
            n_outs: num_outputs,
            kind,
        };
        var
    }

    pub fn make_toolbox_widget(&self) -> egui::Button<'static> {
        //square selectable button that takes a label and number of inputs and outputs
        let var = egui::Button::selectable(
            false, // or set to true if you want it selected by default
            self.label.clone(),
        )
        .min_size(egui::vec2(110., 110.))
        .corner_radius(10.)
        .sense(Sense::drag())
        .sense(Sense::click());
        return var;
    }
}

impl Widget for Primitive {
    fn ui(self, _ui: &mut Ui) -> Response {
        todo!()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)]
pub struct Input {
    pub id: usize,
    pub name: Option<String>,
    pub parent: Option<Gate>, // Optional parent gate, if this input belongs to a gate

    pub signal: bool,
    pub connected: bool,
}

impl Input {
    pub fn new(n: usize, parent_gate: &Gate) -> Self {
        Input {
            id: n,
            name: None,
            parent: Some(parent_gate.clone()), // Optional parent gate, if this input belongs to a gate

            signal: false,
            connected: false,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, Clone, Debug)]
struct WireLine {
    p1: Pos2,
    p2: Pos2,
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

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Wire {
    signal: bool,
    source: Output,
    dest: Option<Input>,
    line: WireLine,
}

impl Wire {
    fn new(source: Output, pos2: Pos2, color: Color32, smoothing: bool) -> Self {
        let pos1 = source.get_position();

        Wire {
            signal: false,
            source,
            dest: None,
            line: WireLine::new(pos1, pos2, color, smoothing),
        }
    }

    fn delete(mut self) {
        //disconnect both output and input
        if let Some(dest) = &mut self.dest {
            dest.connected = false;
        }
        self.source.connected = false;
        self.signal = false;
        self.dest = None;
    }

    fn get_kind(&self) -> Logicals {
        Logicals::Wire
    }

    fn on(mut self) {
        self.signal = true;
    }

    fn off(mut self) {
        self.signal = false
    }
}

impl Logical for Wire {
    fn tick(self) {
        if let Some(mut out) = self.dest {
            if self.signal {
                out.signal = true;
            } else {
                out.signal = false;
            }
        }
    }
    fn get_position(&self) -> Pos2 {
        self.line.p1
    }

    fn set_position(&mut self, pos: Pos2) -> Result<(), Box<dyn Error>> {
        Err(Box::new(InvalidOperationError))
    }
}
