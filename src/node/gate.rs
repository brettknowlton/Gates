use super::*;
use eframe::glow::FILL;
use egui::{pos2, vec2, Align, Align2, Button, Checkbox, Layout, Rect, Stroke, TextStyle, Ui, Vec2};
use egui::{Color32, Pos2, StrokeKind, UiBuilder};

use std::hash::Hash;

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Gate {
    pub label: String,
    pub position: GridVec2,
    pub size: GridVec2,

    //logical properties
    pub n_in: usize,
    pub ins: Vec<Input>,

    pub n_out: usize,
    pub outs: Vec<Output>,

    pub kind: GateType,
}

impl Logical for Gate {
    fn tick(self) {
        println!("This is a generic gate being ticked: {}", self.label);
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

impl Widget for &mut Gate {

    fn ui(self, ui: &mut Ui) -> Response {
        let size = Vec2::new(150.0, 110.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());


        // Draw the bounding box
        ui.painter().rect(rect, 10.0, Color32::from_rgb(30, 30, 30), Stroke::new(1.0, Color32::GRAY), StrokeKind::Middle);

        // Layout for the gate's three sections
        let left_rect = Rect::from_min_max(
            rect.left_top(),
            pos2(rect.left() + 40.0, rect.bottom()),
        );
        let center_rect = Rect::from_min_max(
            pos2(rect.left() + 40.0, rect.top()),
            pos2(rect.right() - 40.0, rect.bottom()),
        );
        let right_rect = Rect::from_min_max(
            pos2(rect.right() - 40.0, rect.top()),
            rect.right_bottom(),
        );

        // LEFT SIDE - Input indicators
        ui.scope_builder(
            UiBuilder::new()
                .layout(Layout::top_down(Align::LEFT))
                .max_rect(left_rect),
            |ui| {
                let checkbox_height = ui.spacing().interact_size.y;
                let total_height = self.n_in as f32 * checkbox_height;
                let parent_height = left_rect.height();
                let top_padding = ((parent_height - total_height) / 2.0).max(0.0);

                ui.add_space(top_padding);
                ui.vertical(|ui| {
                    for i in 0..self.n_in {
                        ui.add_enabled(false, Checkbox::without_text(&mut self.ins[i].signal))
                            .highlight()
                            .on_disabled_hover_text("Input signal");
                    }
                });
            },
        );

        // CENTER - Label only
        ui.painter().rect_filled(center_rect, 0.0, Color32::DARK_GRAY);
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
                let checkbox_height = ui.spacing().interact_size.y;
                let total_height = self.outs.len() as f32 * checkbox_height;
                let parent_height = right_rect.height();
                let top_padding = ((parent_height - total_height) / 2.0).max(0.0);

                ui.add_space(top_padding);

                ui.vertical(|ui| {
                    for output in &self.outs {
                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                            let button_color = if output.signal {
                                Color32::GREEN
                            } else {
                                Color32::DARK_RED
                            };

                            let btn = Button::new(" ")
                                .fill(button_color)
                                .min_size(vec2(18.0, 18.0));

                            if ui.add(btn).clicked() {
                                // Create a new wire with points at this output and the pointer location, push it to the live_data vector
                                println!("Output clicked: {}", output.id);
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
    pub fn new(name: String) -> Gate {
        let n_ins = 0;
        let n_outs = 0;

        Gate {
            label: name,
            position: GridVec2::new(0.0, 0.0),
            size: GridVec2::new(150.0, 110.0),

            n_in: n_ins,
            ins: Self::create_inputs(n_ins),
            n_out: n_outs,
            outs: Self::create_outputs(n_outs),

            kind: GateType::None, // Default type, can be changed later
        }
    }

    pub fn get_signals_in(&self) -> Vec<bool> {
        self.ins.iter().map(|i| i.signal).collect()
    }
    pub fn get_signals_out(&self) -> Vec<bool> {
        self.outs.iter().map(|o| o.signal).collect()
    }

    fn from_template(t: &Primitive, pos: Pos2) -> Gate {
        Gate {
            label: t.label.clone(),
            position: GridVec2::new(pos.x, pos.y),
            size: GridVec2::new(150.0, 110.0),

            n_in: t.n_ins as usize,
            ins: Self::create_inputs(t.n_ins as usize),
            n_out: t.n_outs as usize,
            outs: Self::create_outputs(t.n_outs as usize),

            kind: t.kind.clone(),
        }
    }

    pub fn from_template_id(t: GateType, pos: Pos2) -> Gate {
        print!("Creating gate from template ID: {:?}", t);
        let new_gate = match t {
            GateType::Primitive(PrimitiveKind::TOGGLE) => {
                Gate::from_template(&Primitive::from_values("TOGGLE", 0, 1), pos)
            }
            GateType::Primitive(PrimitiveKind::LIGHT) => {
                Gate::from_template(&Primitive::from_values("LIGHT", 1, 0), pos)
            }
            GateType::Primitive(PrimitiveKind::BUFFER) => {
                Gate::from_template(&Primitive::from_values("BUFFER", 1, 1), pos)
            }
            GateType::Primitive(PrimitiveKind::NOT) => {
                Gate::from_template(&Primitive::from_values("NOT", 1, 1), pos)
            }
            GateType::Primitive(PrimitiveKind::OR) => {
                Gate::from_template(&Primitive::from_values("OR", 2, 1), pos)
            }
            GateType::Primitive(PrimitiveKind::AND) => {
                Gate::from_template(&Primitive::from_values("AND", 2, 1), pos)
            }
            _ => Gate::from_template(&Primitive::from_values("E: Not Found", 1, 1), pos),
        };

        println!("Created gate: {:?}", new_gate);
        new_gate
    }

    fn create_inputs(n_in: usize) -> Vec<Input> {
        let mut new_ins = Vec::<Input>::new();
        for n in 0..n_in {
            new_ins.push(Input::new(n))
        }
        new_ins
    }

    fn create_outputs(n_out: usize) -> Vec<Output> {
        let mut new_outs = Vec::<Output>::new();
        for n in 0..n_out {
            new_outs.push(Output::new(n))
        }
        new_outs
    }

    pub fn generate(label: String, n_ins: usize, n_outs: usize) -> Gate {
        let kind: GateType;
        match label.as_str() {
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

        Gate {
            label,
            position: GridVec2::new(0.0, 0.0),
            size: GridVec2::new(150.0, 110.0),

            n_in: n_ins,
            ins: Self::create_inputs(n_ins),
            n_out: n_outs,
            outs: Self::create_outputs(n_outs),
            kind,
        }
    }
}
