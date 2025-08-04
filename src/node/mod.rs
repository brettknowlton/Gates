pub mod gate;
pub use gate::Gate;

mod wire;
pub use wire::Wire;

mod io;
pub use io::{IOKind, Input, Io, Output};

pub use super::app::ClickItem;
pub use eframe::egui::{Button, Color32, Direction, Layout, Pos2, Response, Sense, Ui, Widget, vec2};

use serde;
use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::{Display, Formatter};

use std::error::Error;


const LINE_THICKNESS: f32 = 2.0;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Logicals {
    Gate(GateType),
    Wire,
    IO(IOKind),
}

#[derive(Debug)]
pub struct InvalidOperationError;
impl Error for InvalidOperationError {}

impl Display for InvalidOperationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Cannot set position for this type")
    }
}

pub trait Logical: AsAny {
    /// Ticks the logical element, updating its state.
    /// This is where the logic of the element is processed.
    fn tick(self);
    fn get_position(&self) -> Result<Pos2, Box<dyn Error>> {
        println!("get_position not implemented for this type");
        Err(Box::new(InvalidOperationError))
    }
    fn set_position(&mut self, pos: Pos2) -> Result<(), Box<dyn Error>>;

    fn get_kind(&self) -> Logicals;

    fn show(
        &self,
        ui: &mut Ui,
        click_item: &mut Option<ClickItem>,
        live_data: &HashMap<usize, Box<dyn Logical>>,
    ) -> Response;
    fn click_on(&mut self) {
        // Default implementation, can be overridden by specific logical types
        println!("Click on not implemented for this type");
    }
}

// Define a trait to allow downcasting
pub trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

// Implement AsAny for all types that implement Logical
impl<T: Logical + 'static> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, serde::Deserialize, serde::Serialize)]
pub enum GateType {
    #[default]
    None,
    Primitive(PrimitiveType),
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

#[derive(serde::Deserialize, serde::Serialize, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    #[default]
    None,
    PULSE,
    LIGHT,
    BUFFER,
    NOT,
    OR,
    AND,
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveType::None => write!(f, "None"),
            PrimitiveType::PULSE => write!(f, "PULSE"),
            PrimitiveType::LIGHT => write!(f, "LIGHT"),
            PrimitiveType::BUFFER => write!(f, "BUFFER"),
            PrimitiveType::NOT => write!(f, "NOT"),
            PrimitiveType::OR => write!(f, "OR"),
            PrimitiveType::AND => write!(f, "AND"),
        }
    }
}

impl Widget for PrimitiveType {
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
    pub fn from_values(label: &str, num_inputs: usize, num_outputs: usize) -> Primitive {
        let kind: GateType;
        match label {
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
        let var = Primitive {
            label: label.to_string(),
            n_ins: num_inputs,
            n_outs: num_outputs,
            kind,
        };
        var
    }

    pub fn make_toolbox_widget(&self) -> Button<'static> {
        //square selectable button that takes a label and number of inputs and outputs
        let var = Button::selectable(
            false, // or set to true if you want it selected by default
            self.label.clone(),
        )
        .min_size(vec2(110., 110.))
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
