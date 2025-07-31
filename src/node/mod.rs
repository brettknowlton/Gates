
pub mod gate;
pub use gate::Gate;
use super::*;

use egui::{
    Label, Response, SelectableLabel, Sense, Ui, Widget, text_selection::LabelSelectionState,
};

trait Logical {
    fn new() -> impl Logical;

    fn tick(self);
}

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ToolboxItem {
    pub label: String,

    pub ins: Vec<Input>,
    pub outs: Vec<Output>,
}

impl ToolboxItem {
    pub fn new(n: String) -> ToolboxItem {
        ToolboxItem {
            label: n,
            ins: Vec::<Input>::new(),
            outs: Vec::<Output>::new(),
        }
    }

    pub fn toolbox_from_save_file(&self) -> egui::Button<'static> {
        egui::Button::selectable(
            false, // or set to true if you want it selected by default
            format!("{}: {} :{}", self.ins.len(), self.label, self.outs.len()),
        )
        .sense(Sense::click())
    }

    pub fn toolbox_from_primitive(label: &str, num_inputs: i32, num_outputs: i32) -> ToolboxItem {
        let mut ins = Vec::new();
        let mut outs = Vec::new();

        for _ in 0..num_inputs {
            ins.push(Input::new());
        }
        for _ in 0..num_outputs {
            outs.push(Output::new());
        }

        let full_label = format!("{}: {} :{}", num_inputs, label, num_outputs);
        let var = ToolboxItem {
            label: full_label,
            ins,
            outs,
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
        .sense(Sense::drag());

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
