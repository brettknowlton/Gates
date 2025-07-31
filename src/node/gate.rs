
use super::*;
use egui::{epaint::RectShape, Vec2, Ui};
use egui::{Color32, Pos2, Rect, Stroke, StrokeKind};

use std::hash::Hash;

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Gate {
    pub label: String,
    pub position: GridVec2,
    pub size: GridVec2,

    pub n_in: usize,
    pub ins: Vec<Input>,

    pub n_out: usize,
    pub outs: Vec<Output>,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct GridVec2{
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


impl Widget for Gate {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut response = ui.allocate_response(ui.available_size(), Sense::click_and_drag());

        if response.clicked() {
            response.mark_changed();

        let rect = Rect::from_min_size(ui.min_rect().min, Vec2::new(150., 110.));

        let rs: RectShape = RectShape::new(
            rect,
            10.0,
            Color32::from_rgb(200, 200, 200),
            Stroke::new(1.0, Color32::BLACK),
            StrokeKind::Outside,
        );

        ui.painter().add(rs);

    }
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
        }
    }

    pub fn get_signal_in(&self) -> Vec<bool> {
        self.ins.iter().map(|i| i.signal).collect()
    }
    pub fn get_signal_out(&self) -> Vec<bool> {
        self.outs.iter().map(|o| o.signal).collect()
    }

    pub fn from_template(t: &mut ToolboxItem) -> Gate {
        let n_ins = t.ins.len();
        let n_outs = t.outs.len();

        Gate {
            label: t.label.clone(),
            position: GridVec2::new(0.0, 0.0),
            size: GridVec2::new(150.0, 110.0),

            n_in: n_ins,
            ins: Self::create_inputs(n_ins),
            n_out: n_outs,
            outs: Self::create_outputs(n_outs),
        }
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

    pub fn get_widget<'a>(&self, ui: impl FnOnce(&mut Ui)) -> egui::Button<'a> {
        egui::Button::selectable(
            false, // or set to true if you want it selected by default
            format!("{}: {} :{}", self.n_in, self.label, self.n_out),
        )
        .min_size(Vec2::new(150., 110.))
    }

    pub fn generate(label: String, n_ins: usize, n_outs: usize) -> Gate {
        Gate {
            label,
            position: GridVec2::new(0.0, 0.0),
            size: GridVec2::new(150.0, 110.0),

            n_in: n_ins,
            ins: Self::create_inputs(n_ins),
            n_out: n_outs,
            outs: Self::create_outputs(n_outs),
        }
    }
}