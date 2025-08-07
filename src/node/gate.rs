use crate::node::io::*;
use crate::{MyApp, UiEvent};

use super::*;

use crossbeam::channel::Sender;
use eframe::egui::{Align, Align2, Layout, Rect, Stroke, TextStyle, Ui, Vec2, pos2};
use eframe::egui::{Color32, Pos2, StrokeKind, UiBuilder};

use std::collections::HashMap;
use std::hash::Hash;

const GATE_WIDTH: f32 = 100.0;

#[derive(serde::Deserialize, serde::Serialize, Default, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Gate {
    pub label: String,
    pub id: usize, //index of this gate in the PanArea
    pub position: GridVec2,
    pub size: GridVec2,

    //logical properties
    pub n_in: usize,
    pub ins: HashMap<usize, bool>, //bool represents the interpreted input state, this will be passed to the gate on its tick() function

    pub n_out: usize,
    pub outs: HashMap<usize, bool>, //bool represents the desired output state, this will be passed to the outputs on their tick() function

    pub kind: GateKind,
    pub state: bool,
}

impl Logical for Gate {
    fn tick(&mut self, ins: HashMap<usize, bool>) -> Result<HashMap<usize, bool>, Box<dyn Error>> {
        let k = self.kind.clone();
        match k {
            GateKind::Primitive(k) => {
                // For primitive gates, we can run their logic
                k.tick(self, ins)
            }
            _ => Err("Tick not implemented for this type".into()),
        }
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn get_position(&self) -> Result<Pos2, Box<(dyn Error + 'static)>> {
        Ok(self.position.to_pos2())
    }
    fn get_kind(&self) -> LogicalKind {
        LogicalKind::Gate(self.kind.clone())
    }
    fn set_position(&mut self, pos: Pos2) -> Result<(), Box<dyn Error>> {
        self.position = GridVec2::from(pos);
        Ok(())
    }
    fn show(
        &self,
        ui: &mut Ui,
        sender: Sender<UiEvent>,
        live_data: &HashMap<usize, Box<dyn Logical>>,
        colors: &HashMap<String, Color32>,
    ) -> Response {
        let size = Vec2::new(GATE_WIDTH, 50.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());

        let checkbox_height = ui.spacing().interact_size.y;

        let mut fill_color: Color32 = ui.style().visuals.widgets.inactive.bg_fill;
        let mut accent_color: Color32 = ui.style().visuals.widgets.inactive.weak_bg_fill;

        match self.kind {
            GateKind::Primitive(PrimitiveKind::HISIGNAL) => {
                fill_color = colors
                    .get(HI_SIGNAL_COLOR)
                    .cloned()
                    .unwrap_or(Color32::from_rgb(0, 255, 0));
                accent_color = colors
                    .get(HI_ACCENT_COLOR)
                    .cloned()
                    .unwrap_or(Color32::from_rgb(0, 127, 0));
            }
            GateKind::Primitive(PrimitiveKind::LOSIGNAL) => {
                fill_color = colors
                    .get(LO_SIGNAL_COLOR)
                    .cloned()
                    .unwrap_or(Color32::from_rgb(255, 0, 0));
                accent_color = colors
                    .get(LO_ACCENT_COLOR)
                    .cloned()
                    .unwrap_or(Color32::from_rgb(127, 0, 0));
            }
            GateKind::Primitive(PrimitiveKind::PULSE)
            | GateKind::Primitive(PrimitiveKind::TOGGLE) => {
                if self.state {
                    accent_color = ui.style().visuals.widgets.active.bg_fill;
                } else {
                    accent_color = ui.style().visuals.widgets.inactive.bg_fill;
                }
                fill_color = ui.style().visuals.widgets.inactive.weak_bg_fill;
            }
            GateKind::Primitive(PrimitiveKind::LIGHT) => {
                if self.state {
                    accent_color = ui.style().visuals.selection.bg_fill;
                    fill_color = ui.style().visuals.widgets.inactive.weak_bg_fill;
                }
            }
            _ => {}
        }

        // Draw the bounding box
        ui.painter().rect(
            rect,
            10.0,
            accent_color,
            Stroke::new(1.0, Color32::GRAY),
            StrokeKind::Middle,
        );

        // Layout for the gate's three sections
        let left_rect = Rect::from_min_max(
            rect.left_top(),
            pos2(rect.left() + checkbox_height, rect.bottom()),
        );
        let center_rect = Rect::from_min_max(
            pos2(rect.left() + checkbox_height, rect.top()),
            pos2(rect.right() - checkbox_height, rect.bottom()),
        );
        let right_rect = Rect::from_min_max(
            pos2(rect.right() - checkbox_height, rect.top()),
            rect.right_bottom(),
        );

        // LEFT SIDE - Input indicators
        ui.scope_builder(
            UiBuilder::new()
                .layout(Layout::top_down(Align::LEFT))
                .max_rect(left_rect),
            |ui| {
                let total_height = self.n_in as f32 * checkbox_height;
                let parent_height = left_rect.height();
                let top_padding = ((parent_height - total_height) / 2.0).max(0.0);

                ui.add_space(top_padding);
                ui.vertical(|ui| {
                    for (id, _) in &self.ins {
                        live_data.get(id).map(|input_logical| {
                            if let Some(input) = input_logical.as_any().downcast_ref::<Input>() {
                                ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                                    input.show(ui, sender.clone(), live_data, colors);
                                });
                            }
                        });
                    }
                });
            },
        );

        // CENTER - Label only
        ui.painter().rect_filled(center_rect, 0.0, fill_color);
        ui.painter().text(
            center_rect.center(),
            Align2::CENTER_CENTER,
            self.label.clone(),
            TextStyle::Monospace.resolve(ui.style()),
            Color32::BLACK,
        );

        // RIGHT SIDE - Output buttons for wire creation
        ui.scope_builder(
            UiBuilder::new()
                .layout(Layout::top_down(Align::RIGHT))
                .max_rect(right_rect),
            |ui| {
                let total_height = self.outs.len() as f32 * checkbox_height;
                let parent_height = right_rect.height();
                let top_padding = ((parent_height - total_height) / 2.0).max(0.0);

                ui.add_space(top_padding);

                ui.vertical(|ui| {
                    for output in self.outs.iter() {
                        live_data.get(&output.0).map(|input_logical| {
                            if let Some(output) = input_logical.as_any().downcast_ref::<Output>() {
                                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                    output.show(ui, sender.clone(), live_data, colors);
                                });
                            }
                        });
                    }
                });
            },
        );
        response
    }
}

impl Gate {
    pub fn new(name: String, id: usize) -> Gate {
        let n_ins = 0;
        let n_outs = 0;

        let g = Gate {
            label: name,
            id,
            position: GridVec2::new(0.0, 0.0),
            size: GridVec2::new(150.0, 110.0),

            n_in: n_ins,
            ins: HashMap::new(),
            n_out: n_outs,
            outs: HashMap::new(),

            kind: GateKind::None,

            state: false,
        };
        g
    }

    pub fn click_on(&mut self) {
        match self.kind {
            GateKind::Primitive(PrimitiveKind::PULSE) => {
                self.state = !self.state;
            }
            GateKind::Primitive(PrimitiveKind::TOGGLE) => {
                self.state = !self.state;
            }
            _ => println!("This gate type does not support click actions"),
        }
    }

    fn from_template(t: &PrimitiveTemplate, pos: Pos2) -> Gate {
        let g = Gate {
            label: t.label.clone(),
            id: MyApp::next_id(),
            position: GridVec2::new(pos.x, pos.y),
            size: GridVec2::new(150.0, 110.0),

            n_in: t.n_ins as usize,
            ins: HashMap::new(),
            n_out: t.n_outs as usize,
            outs: HashMap::new(),

            kind: t.kind.get_gate_kind(),
            state: false,
        };
        g
    }

    pub fn create_gate_from_template(t: GateKind, pos: Pos2) -> Gate {
        print!("Creating gate from template ID: {:?}", t);
        let new_gate = match t {
            GateKind::Primitive(PrimitiveKind::HISIGNAL) => {
                Gate::from_template(&PrimitiveTemplate::from_values("HI-SIGNAL", 0, 1), pos)
            }
            GateKind::Primitive(PrimitiveKind::LOSIGNAL) => {
                Gate::from_template(&PrimitiveTemplate::from_values("LO-SIGNAL", 0, 1), pos)
            }
            GateKind::Primitive(PrimitiveKind::PULSE) => {
                Gate::from_template(&PrimitiveTemplate::from_values("PULSE", 0, 1), pos)
            }
            GateKind::Primitive(PrimitiveKind::TOGGLE) => {
                Gate::from_template(&PrimitiveTemplate::from_values("TOGGLE", 0, 1), pos)
            }
            GateKind::Primitive(PrimitiveKind::LIGHT) => {
                Gate::from_template(&PrimitiveTemplate::from_values("LIGHT", 1, 0), pos)
            }
            GateKind::Primitive(PrimitiveKind::BUFFER) => {
                Gate::from_template(&PrimitiveTemplate::from_values("BUFFER", 1, 1), pos)
            }
            GateKind::Primitive(PrimitiveKind::NOT) => {
                Gate::from_template(&PrimitiveTemplate::from_values("NOT", 1, 1), pos)
            }
            GateKind::Primitive(PrimitiveKind::OR) => {
                Gate::from_template(&PrimitiveTemplate::from_values("OR", 2, 1), pos)
            }
            GateKind::Primitive(PrimitiveKind::AND) => {
                Gate::from_template(&PrimitiveTemplate::from_values("AND", 2, 1), pos)
            }
            GateKind::Primitive(PrimitiveKind::XOR) => {
                Gate::from_template(&PrimitiveTemplate::from_values("XOR", 2, 1), pos)
            }
            GateKind::Primitive(PrimitiveKind::NAND) => {
                Gate::from_template(&PrimitiveTemplate::from_values("NAND", 2, 1), pos)
            }
            GateKind::Primitive(PrimitiveKind::NOR) => {
                Gate::from_template(&PrimitiveTemplate::from_values("NOR", 2, 1), pos)
            }
            _ => Gate::from_template(&PrimitiveTemplate::from_values("E: Not Found", 1, 1), pos),
        };

        println!("Created gate: {:?}", new_gate);
        new_gate
    }

    pub fn create_io(&mut self, live_data: &mut HashMap<usize, Box<dyn Logical>>) {
        self.create_inputs(live_data);
        self.create_outputs(live_data);
    }

    pub fn create_inputs(&mut self, live_data: &mut HashMap<usize, Box<dyn Logical>>) {
        let mut new_ins = HashMap::<usize, bool>::new();
        for i in 0..self.n_in {
            let new_input = Input::new(self.id, i);
            live_data.insert(new_input.id, Box::new(new_input.clone()));
            new_ins.insert(new_input.id, false);
        }
        self.ins = new_ins;
    }

    pub fn create_outputs(&mut self, live_data: &mut HashMap<usize, Box<dyn Logical>>) {
        let mut new_outs = HashMap::<usize, bool>::new();
        for i in 0..self.n_out {
            let new_output = Output::new(self.id, i);
            live_data.insert(new_output.id, Box::new(new_output.clone()));
            new_outs.insert(new_output.id, false); // Initialize with false signal
        }
        self.outs = new_outs;
    }

    pub fn generate(label: String, n_ins: usize, n_outs: usize) -> Gate {
        let kind: GateKind;
        let id = MyApp::next_id();
        match label.as_str() {
            "HI-SIGNAL" => {
                kind = GateKind::Primitive(PrimitiveKind::HISIGNAL);
            }
            "LO-SIGNAL" => {
                kind = GateKind::Primitive(PrimitiveKind::LOSIGNAL);
            }
            "PULSE" => {
                kind = GateKind::Primitive(PrimitiveKind::PULSE);
            }
            "TOGGLE" => {
                kind = GateKind::Primitive(PrimitiveKind::TOGGLE);
            }
            "LIGHT" => {
                kind = GateKind::Primitive(PrimitiveKind::LIGHT);
            }
            "BUFFER" => {
                kind = GateKind::Primitive(PrimitiveKind::BUFFER);
            }
            "NOT" => {
                kind = GateKind::Primitive(PrimitiveKind::NOT);
            }
            "OR" => {
                kind = GateKind::Primitive(PrimitiveKind::OR);
            }
            "AND" => {
                kind = GateKind::Primitive(PrimitiveKind::AND);
            }
            "XOR" => {
                kind = GateKind::Primitive(PrimitiveKind::XOR);
            }
            "NAND" => {
                kind = GateKind::Primitive(PrimitiveKind::NAND);
            }
            "NOR" => {
                kind = GateKind::Primitive(PrimitiveKind::NOR);
            }
            "Custom" => {
                kind = GateKind::Custom;
            }
            _ => {
                kind = GateKind::Primitive(PrimitiveKind::None);
            }
        }

        let g = Gate {
            label,
            position: GridVec2::new(0.0, 0.0),
            id: id,

            size: GridVec2::new(150.0, 110.0),

            n_in: n_ins,
            ins: HashMap::new(),
            n_out: n_outs,
            outs: HashMap::new(),
            kind,

            state: false,
        };
        g
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, serde::Deserialize, serde::Serialize)]
pub enum GateKind {
    #[default]
    None,
    Primitive(PrimitiveKind),
    Custom,
}

impl Display for GateKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateKind::None => write!(f, "None"),
            GateKind::Primitive(kind) => write!(f, "{}", kind),
            GateKind::Custom => write!(f, "Custom"),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct GridVec2 {
    pub vec: Vec2,
}

impl GridVec2 {
    pub fn new(x: f32, y: f32) -> Self {
        GridVec2 {
            vec: Vec2::new(x, y),
        }
    }

    pub fn to_vec2(self) -> Vec2 {
        self.vec
    }

    pub fn to_pos2(&self) -> Pos2 {
        Pos2::new(self.vec.x, self.vec.y)
    }

    pub fn from(pos: Pos2) -> Self {
        GridVec2 {
            vec: Vec2::new(pos.x, pos.y),
        }
    }
    pub fn from_vec(vec: Vec2) -> Self {
        GridVec2 { vec }
    }
}

impl Hash for GridVec2 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vec.x.to_bits().hash(state);
        self.vec.y.to_bits().hash(state);
    }
}
