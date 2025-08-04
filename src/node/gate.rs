use crate::MyApp;
use crate::node::io::*;

use super::*;

use eframe::egui::{Align, Align2, Layout, Rect, Stroke, TextStyle, Ui, Vec2, pos2};
use eframe::egui::{Color32, Pos2, StrokeKind, UiBuilder};

use std::collections::HashMap;
use std::hash::Hash;

const GATE_WIDTH: f32 = 100.0;

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Gate {
    pub label: String,
    pub id: usize, //index of this gate in the PanArea
    pub position: GridVec2,
    pub size: GridVec2,

    //logical properties
    pub n_in: usize,
    pub ins: Vec<usize>,

    pub n_out: usize,
    pub outs: Vec<usize>,

    pub kind: GateType,
    state: bool,
}

impl Logical for Gate {
    fn tick(self) {
        println!("This is a generic gate being ticked: {}", self.label);
    }
    fn get_position(&self) -> Result<Pos2, Box<(dyn Error + 'static)>> {
        Ok(self.position.to_pos2())
    }
    fn get_kind(&self) -> Logicals {
        Logicals::Gate(self.kind.clone())
    }
    fn set_position(&mut self, pos: Pos2) -> Result<(), Box<dyn Error>> {
        self.position = GridVec2::from(pos);
        Ok(())
    }
    fn show(
        &self,
        ui: &mut Ui,
        click_item: &mut Option<ClickItem>,
        live_data: &HashMap<usize, Box<dyn Logical>>,
    ) -> Response {
        let size = Vec2::new(GATE_WIDTH, 50.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());

        let checkbox_height = ui.spacing().interact_size.y;

        // Draw the bounding box
        ui.painter().rect(
            rect,
            10.0,
            Color32::from_rgb(30, 30, 30),
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
                    for input in &self.ins {
                        live_data.get(input).map(|input_logical| {
                            if let Some(input) = input_logical.as_any().downcast_ref::<Input>() {
                                ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                                    input.show(ui, click_item, live_data);
                                });
                            }
                        });
                    }
                });
            },
        );

        // CENTER - Label only
        ui.painter()
            .rect_filled(center_rect, 0.0, Color32::DARK_GRAY);
        ui.painter().text(
            center_rect.center(),
            Align2::CENTER_CENTER,
            self.label.clone(),
            TextStyle::Button.resolve(ui.style()),
            Color32::WHITE,
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
                    for (_, output) in self.outs.iter().enumerate() {
                        live_data.get(output).map(|input_logical| {
                            if let Some(output) = input_logical.as_any().downcast_ref::<Output>() {
                                ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                                    output.show(ui, click_item, live_data);
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
            ins: Vec::new(),
            n_out: n_outs,
            outs: Vec::new(),

            kind: GateType::None,

            state: false,
        };
        g
    }

    pub fn click_on(&mut self) {
        match self.kind {
            GateType::Primitive(PrimitiveType::PULSE) => {
                self.state = !self.state;
            }
            _ => println!("This gate type does not support click actions"),
        }
    }

    fn from_template(t: &Primitive, pos: Pos2) -> Gate {
        let g = Gate {
            label: t.label.clone(),
            id: MyApp::next_id(),
            position: GridVec2::new(pos.x, pos.y),
            size: GridVec2::new(150.0, 110.0),

            n_in: t.n_ins as usize,
            ins: Vec::new(),
            n_out: t.n_outs as usize,
            outs: Vec::new(),

            kind: t.kind.clone(),
            state: false,
        };
        g
    }

    pub fn next_id() -> usize {
        static mut ID: usize = 0;
        unsafe {
            ID += 1;
            ID
        }
    }

    pub fn create_gate_from_template(t: GateType, pos: Pos2) -> Gate {
        print!("Creating gate from template ID: {:?}", t);
        let new_gate = match t {
            GateType::Primitive(PrimitiveType::PULSE) => {
                Gate::from_template(&Primitive::from_values("PULSE", 0, 1), pos)
            }
            GateType::Primitive(PrimitiveType::LIGHT) => {
                Gate::from_template(&Primitive::from_values("LIGHT", 1, 0), pos)
            }
            GateType::Primitive(PrimitiveType::BUFFER) => {
                Gate::from_template(&Primitive::from_values("BUFFER", 1, 1), pos)
            }
            GateType::Primitive(PrimitiveType::NOT) => {
                Gate::from_template(&Primitive::from_values("NOT", 1, 1), pos)
            }
            GateType::Primitive(PrimitiveType::OR) => {
                Gate::from_template(&Primitive::from_values("OR", 2, 1), pos)
            }
            GateType::Primitive(PrimitiveType::AND) => {
                Gate::from_template(&Primitive::from_values("AND", 2, 1), pos)
            }
            _ => Gate::from_template(&Primitive::from_values("E: Not Found", 1, 1), pos),
        };

        println!("Created gate: {:?}", new_gate);
        new_gate
    }

    pub fn create_io(&mut self, live_data: &mut HashMap<usize, Box<dyn Logical>>) {
        self.create_inputs(live_data);
        self.create_outputs(live_data);
    }

    pub fn create_inputs(&mut self, live_data: &mut HashMap<usize, Box<dyn Logical>>) {
        let mut new_ins = Vec::<usize>::new();
        for _ in 0..self.n_in {
            let new_input = Input::new(self.id);
            live_data.insert(new_input.id, Box::new(new_input.clone()));
            new_ins.push(new_input.id);
        }
        self.ins = new_ins;
    }

    pub fn create_outputs(&mut self, live_data: &mut HashMap<usize, Box<dyn Logical>>) {
        let mut new_outs = Vec::<usize>::new();
        for _ in 0..self.n_out {
            let new_output = Output::new(self.id);
            live_data.insert(new_output.id, Box::new(new_output.clone()));
            new_outs.push(new_output.id);
        }
        self.outs = new_outs;
    }

    pub fn generate(label: String, n_ins: usize, n_outs: usize) -> Gate {
        let kind: GateType;
        let id = Gate::next_id();
        match label.as_str() {
            "PULSE" => {
                kind = GateType::Primitive(PrimitiveType::PULSE);
            }
            "LIGHT" => {
                kind = GateType::Primitive(PrimitiveType::LIGHT);
            }
            "BUFFER" => {
                kind = GateType::Primitive(PrimitiveType::BUFFER);
            }
            "NOT" => {
                kind = GateType::Primitive(PrimitiveType::NOT);
            }
            "OR" => {
                kind = GateType::Primitive(PrimitiveType::OR);
            }
            "AND" => {
                kind = GateType::Primitive(PrimitiveType::AND);
            }
            _ => {
                kind = GateType::Primitive(PrimitiveType::None);
            }
        }

        let g = Gate {
            label,
            position: GridVec2::new(0.0, 0.0),
            id: id,

            size: GridVec2::new(150.0, 110.0),

            n_in: n_ins,
            ins: Vec::new(),
            n_out: n_outs,
            outs: Vec::new(),
            kind,

            state: false,
        };
        g
    }
}
