pub mod logical;
pub use logical::{AsAny, Logical, LogicalKind, InvalidOperationError};

pub mod gate;
pub use gate::{Gate, GateKind};

mod wire;
pub use wire::Wire;

mod primitive;
pub use primitive::{PrimitiveTemplate, PrimitiveKind};

mod io;
pub use io::{IOKind, Input, Io, Output};

pub use super::app::UiEvent;
pub use eframe::egui::{
    Button, Color32, Direction, Layout, Pos2, Response, Sense, Ui, Widget, vec2,
};

use serde;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

use std::error::Error;

const LINE_THICKNESS: f32 = 3.0;

const HI_SIGNAL_COLOR: &str = "color-success-500";
const HI_ACCENT_COLOR: &str = "color-success-900";


const LO_SIGNAL_COLOR: &str = "color-error-500";
const LO_ACCENT_COLOR: &str = "color-error-900";