
pub mod gate;
use std::default;

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

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
pub enum PrimitiveKind {
    #[default]
    None,
    Button,
    Light,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ToolboxItem {
    pub label: String,
    pub kind: GateType,
    pub n_ins: i32,
    pub n_outs: i32,
}

impl ToolboxItem {
    pub fn new(n: String) -> ToolboxItem {
        ToolboxItem {
            label: n,
            n_ins: 0,
            n_outs: 0,
            kind: GateType::None,
        }
    }


    pub fn toolbox_from_values(label: &str, num_inputs: i32, num_outputs: i32) -> ToolboxItem {
        let kind: GateType;
        match label {
            "Button" => {
                kind = GateType::Primitive(PrimitiveKind::Button);
            }
            "Light" => {
                kind = GateType::Primitive(PrimitiveKind::Light);
            }
            _ => {
                kind = GateType::Primitive(PrimitiveKind::None);
            }
        }

        let full_label = format!("{}: {} :{}", num_inputs, label, num_outputs);
        let var = ToolboxItem {
            label: full_label,
            n_ins: num_inputs,
            n_outs: num_outputs,
            kind,
        };
        var
    }

    pub fn make_primitive(&self) -> egui::Button<'static> {
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

impl Widget for ToolboxItem {
    fn ui(self, ui: &mut Ui) -> Response {
        todo!()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
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
