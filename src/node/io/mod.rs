use super::*;
pub mod output;
pub use output::Output;
mod input;
pub use input::Input;
use serde::{Deserialize, Serialize};

pub use crate::{ MyApp, node::InvalidOperationError, UiEvent};

pub use eframe::egui::{Align, PointerButton};

pub use crossbeam::channel::Sender;

pub trait Io: Logical + AsAny {
    fn get_position(
        &self,
        opt_data: &HashMap<usize, Box<dyn Logical>>,
    ) -> Result<Pos2, Box<dyn std::error::Error>>;
    fn set_position(&mut self, pos: Pos2) -> Result<(), Box<dyn Error>>;
}



#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum IOKind {
    Input,
    Output,
}