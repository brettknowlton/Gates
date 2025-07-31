pub mod hi_signal;
use egui::{
    Label, Response, SelectableLabel, Sense, Ui, Widget, text_selection::LabelSelectionState,
};

trait Logical {
    fn new() -> impl Logical;

    fn tick(self);
}

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct LogicGateTemplate {
    pub label: String,

    ins: Vec<Input>,
    outs: Vec<Output>,
}

impl LogicGateTemplate {
    pub fn new(n: String) -> LogicGateTemplate {
        LogicGateTemplate {
            label: n,
            ins: Vec::<Input>::new(),
            outs: Vec::<Output>::new(),
        }
    }

    pub fn make_selectable_item(&self) -> egui::Button<'static> {
        egui::Button::selectable(
            false, // or set to true if you want it selected by default
            format!("{}: {} :{}", self.ins.len(), self.label, self.outs.len()),
        )
        .sense(Sense::click())
    }

    pub fn primitive_from(label: &str, num_inputs: i32, num_outputs: i32) -> LogicGateTemplate {
        let mut ins = Vec::new();
        let mut outs = Vec::new();

        for _ in 0..num_inputs {
            ins.push(Input::new());
        }
        for _ in 0..num_outputs {
            outs.push(Output::new());
        }

        let full_label = format!("{}: {} :{}", num_inputs, label, num_outputs);
        let var = LogicGateTemplate {
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

impl Widget for LogicGateTemplate {
    fn ui(self, ui: &mut Ui) -> Response {
        todo!()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Input {
    signal: bool,
    connected: bool,
}

impl Input {
    pub fn new() -> Self {
        Input {
            signal: false,
            connected: false,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Output {
    signal: bool,
    dests: Vec<Wire>,
    connected: bool,
}

impl Output {
    pub fn new() -> Self {
        Output {
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
