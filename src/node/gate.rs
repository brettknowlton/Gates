use super::*;
use egui::{Button, Layout, Ui, Vec2};
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

    pub fn to_vec2(&self) -> Vec2 {
        self.vec
    }

    pub fn to_pos2(&self) -> Pos2 {
        Pos2::new(self.vec.x, self.vec.y)
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

        // Draw the outer button background
        let button_fill = Color32::from_rgb(30, 30, 30);
        ui.painter().rect(
            rect,
            10.0,
            button_fill,
            egui::Stroke::new(3., Color32::DARK_RED),
            StrokeKind::Inside,
        );

        // Split the rect into three equal vertical sections
        let third_width = rect.width() / 3.0;
        let left_rect = egui::Rect::from_min_max(
            rect.left_top(),
            rect.left_top() + Vec2::new(third_width, rect.height()),
        );
        let center_rect = egui::Rect::from_min_max(
            rect.left_top() + Vec2::new(third_width, 0.0),
            rect.left_top() + Vec2::new(2.0 * third_width, rect.height()),
        );
        let right_rect = egui::Rect::from_min_max(
            rect.left_top() + Vec2::new(2.0 * third_width, 0.0),
            rect.right_bottom(),
        );

        ui.scope_builder(
            UiBuilder::new()
                .layout(Layout::left_to_right(egui::Align::Center))
                .max_rect(rect)
                .sense(Sense::click_and_drag()),
            |ui| {
                    match self.kind {
                    GateType::Primitive(PrimitiveKind::BUTTON) => {
                        let btn = ui.add(Button::new("BTN").min_size(size).sense(Sense::click()));
                        if btn.clicked() {
                            //set this gate's output 1 signal to true
                            self.outs[0].signal = true;
                        }

                    }
                    GateType::Primitive(PrimitiveKind::LIGHT) => {
                        let btn = ui.add(Button::new("LIT").min_size(size).sense(Sense::click()));
                        if btn.clicked() {
                            //set this gate's output 1 signal to true
                            self.outs[0].signal = true;
                        }
                    }
                    GateType::Primitive(PrimitiveKind::BUFFER) => {
                        let btn = ui.add(Button::new("BUF").min_size(size).sense(Sense::click()));
                        if btn.clicked() {
                            //set this gate's output 1 signal to true
                            self.outs[0].signal = self.ins[0].signal;
                        }
                    }
                    _ => {
                        ui.label("Unknown Gate Type");
                    }
                }
        });
        // Left: n_in checkboxes, aligned left
        ui.scope_builder(
            UiBuilder::new()
                .layout(Layout::top_down(egui::Align::LEFT))
                .max_rect(left_rect),
            |ui| {
                for input in &mut self.ins {
                    ui.add(egui::Checkbox::without_text(&mut input.signal));
                }
            },
        );

        // Right: n_out checkboxes, aligned right
        ui.scope_builder(
            UiBuilder::new()
                .layout(Layout::top_down(egui::Align::RIGHT))
                .max_rect(right_rect),
            |ui| {
                for output in &mut self.outs {
                    ui.add(egui::Checkbox::without_text(&mut output.signal));
                }
            },
        );

        // // Center: label, centered
        // ui.allocate_ui_at_rect(center_rect, |ui| {
        //     ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
        //         ui.label(
        //             egui::RichText::new(self.label.clone())
        //                 .size(16.0)
        //                 .color(Color32::WHITE),
        //         );
        //     });
        // });

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

    pub fn get_signal_in(&self) -> Vec<bool> {
        self.ins.iter().map(|i| i.signal).collect()
    }
    pub fn get_signal_out(&self) -> Vec<bool> {
        self.outs.iter().map(|o| o.signal).collect()
    }

    pub fn from_template(t: &Primitive, pos: Pos2) -> Gate {
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
            GateType::Primitive(PrimitiveKind::BUTTON) => {
                Gate::from_template(&Primitive::from_values("BUTTON", 0, 1), pos)
            }
            GateType::Primitive(PrimitiveKind::LIGHT) => {
                Gate::from_template(&Primitive::from_values("LIGHT", 1, 0), pos)
            }
            GateType::Primitive(PrimitiveKind::BUFFER) => {
                Gate::from_template(&Primitive::from_values("BUFFER", 1, 1), pos)
            }
            _ => Gate::from_template(&Primitive::from_values("BUFFER", 1, 1), pos),
        };
        println!("Created gate: {:?}", new_gate);
        new_gate
    }

    fn create_inputs(n_in: usize) -> Vec<Input> {
        let mut new_ins = Vec::<Input>::new();
        for n in 0..n_in {
            new_ins.push(Input::new())
        }
        new_ins
    }

    fn create_outputs(n_out: usize) -> Vec<Output> {
        let mut new_outs = Vec::<Output>::new();
        for n in 0..n_out {
            new_outs.push(Output::new())
        }
        new_outs
    }

    pub fn generate(label: String, n_ins: usize, n_outs: usize) -> Gate {
        let kind: GateType;
        match label.as_str() {
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
