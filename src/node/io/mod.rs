use super::*;
pub mod output;
pub use output::Output;
mod input;
pub use input::Input;

pub use crate::{ MyApp, node::InvalidOperationError};

pub use eframe::egui::{Align};


pub trait Io: Logical + AsAny {
    fn get_position(
        &self,
        opt_data: &HashMap<usize, Box<dyn Logical>>,
    ) -> Result<Pos2, Box<dyn std::error::Error>>;
    fn set_position(&mut self, pos: Pos2) -> Result<(), Box<dyn Error>>;
}



#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IOKind {
    Input,
    Output,
}