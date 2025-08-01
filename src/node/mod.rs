
pub mod gate;
use std::{default, fmt::Display};

pub use gate::Gate;
use super::*;

use egui::{
    Label, Response, SelectableLabel, Sense, Ui, Widget, text_selection::LabelSelectionState,
};

trait Logical {

    fn tick(self);
}

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
pub enum GateType {
    #[default]
    None,
    Primitive(PrimitiveKind),
    Custom,
}

impl Display for GateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateType::None => write!(f, "None"),
            GateType::Primitive(kind) => write!(f, "{}", kind),
            GateType::Custom => write!(f, "Custom"),
        }
    }
}

impl GateType {
    pub fn lookup_kind(name: &str) -> GateType {
        match name {
            "Button" => GateType::Primitive(PrimitiveKind::BUTTON),
            "Light" => GateType::Primitive(PrimitiveKind::LIGHT),
            _ => GateType::None,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
pub enum PrimitiveKind {
    #[default]
    None,
    BUTTON,
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
            PrimitiveKind::BUTTON => write!(f, "BUTTON"),
            PrimitiveKind::LIGHT=> write!(f, "LIGHT"),
            PrimitiveKind::BUFFER => write!(f, "BUFFER"),
            PrimitiveKind::NOT => write!(f, "NOT"),
            PrimitiveKind::OR => write!(f, "OR"),
            PrimitiveKind::AND => write!(f, "AND"),

        }
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
    pub fn new(n: String) -> Primitive {

        let k= GateType::lookup_kind(&n);

        Primitive {
            label: n,
            n_ins: 0,
            n_outs: 0,
            kind: k,
        }
    }

    pub fn kind_as_str(self) -> String {
        self.kind.to_string().clone()
    }

    pub fn from_values(label: &str, num_inputs: usize, num_outputs: usize) -> Primitive {
        let kind: GateType;
        match label {
            "BUTTON" => {
                kind = GateType::Primitive(PrimitiveKind::BUTTON);
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

    pub fn make_prim_widget(&self) -> egui::Button<'static> {
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
    fn ui(self, ui: &mut Ui) -> Response {
        todo!()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)]
pub struct Input {
    pub name: Option<String>,
    pub signal: bool,
    pub connected: bool,
}

impl Input {
    pub fn new() -> Self {
        Input {
            name: None,
            signal: false,
            connected: false,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Output {
    pub name: Option<String>,
    pub connected: bool,
    pub signal: bool,
    pub dests: Vec<Wire>,
}

impl Output {
    pub fn new() -> Self {
        Output {
            name: None,
            signal: false,
            dests: Vec::new(),
            connected: false,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Wire {
    signal: bool,
    source: Output,
    dest: Input,
}

impl Wire {
    fn on(mut self) {
        self.signal = true;
    }

    fn off(mut self) {
        self.signal = false
    }
}
