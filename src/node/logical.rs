use crossbeam::channel::Sender;
use serde::{Deserialize, Serialize};

use crate::UiEvent;

use super::*;

pub trait Logical: AsAny {
    /// Ticks the logical element, updating its state.
    /// This is where the logic of the element is processed.
    fn tick(&mut self, _: HashMap<usize, bool>) -> Result<HashMap<usize, bool>, Box<dyn Error>> {
        // Default implementation, can be overridden by specific logical types
        println!();
        Err("Tick not implemented for this type".into())
    }
    
    fn get_position(&self) -> Result<Pos2, Box<dyn Error>> {
        Err(Box::new(InvalidOperationError("get_position not implemented".to_string())))
    }

    fn get_id(&self) -> usize {
        usize::MAX // Default ID, should be overridden by specific logical types
    }

    fn set_position(&mut self, pos: Pos2) -> Result<(), Box<dyn Error>>;
    fn get_kind(&self) -> LogicalKind;
    fn show(
        &self,
        ui: &mut Ui,
        sender: Sender<UiEvent>,
        live_data: &HashMap<usize, Box<dyn Logical>>,
        colors: &HashMap<String, Color32>,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum LogicalKind {
    Gate(GateKind),
    Wire,
    IO(IOKind),
    Chip(String), // Chip kind with a name
}
impl LogicalKind {
    pub fn is_gate(&self) -> bool {
        matches!(self, LogicalKind::Gate(_))
    }
    pub fn is_primitive(&self) -> bool {
        matches!(self, LogicalKind::Gate(GateKind::Primitive(_)))
    }

    pub fn is_primitive_kind(&self, kind: PrimitiveKind) -> bool {
        if let LogicalKind::Gate(GateKind::Primitive(primitive_kind)) = self {
            *primitive_kind == kind
        } else {
            false
        }
    }

    pub fn is_wire(&self) -> bool {
        matches!(self, LogicalKind::Wire)
    }
    pub fn is_io(&self) -> bool {
        matches!(self, LogicalKind::IO(_))
    }
    pub fn is_input(&self) -> bool {
        matches!(self, LogicalKind::IO(IOKind::Input))
    }
    pub fn is_output(&self) -> bool {
        matches!(self, LogicalKind::IO(IOKind::Output))
    }


    pub fn as_gate(&self) -> Result<GateKind, Box<dyn Error>> {
        if let LogicalKind::Gate(gate_kind) = self {
            Ok(gate_kind.clone())
        } else {
            Err("Not a gate kind".into())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct InvalidOperationError(pub String);
impl InvalidOperationError {
    pub fn new(msg: &str) -> Self {
        InvalidOperationError(msg.to_string())
    }
}
impl Error for InvalidOperationError {}
impl Display for InvalidOperationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        format!("{}", self.0)
            .fmt(f)
    }
}
