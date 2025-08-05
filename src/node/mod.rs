pub mod gate;
pub use gate::Gate;

mod wire;
pub use wire::Wire;

mod primitive;
pub use primitive::{Primitive, PrimitiveType};

mod io;
pub use io::{IOKind, Input, Io, Output};

pub use super::app::ClickItem;
pub use eframe::egui::{
    Button, Color32, Direction, Layout, Pos2, Response, Sense, Ui, Widget, vec2,
};

use serde;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

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
    fn tick(&mut self, _: HashMap<usize, bool>) -> Result<HashMap<usize, bool>, Box<dyn Error>> {
        // Default implementation, can be overridden by specific logical types
        println!();
        Err("Tick not implemented for this type".into())
    }
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
