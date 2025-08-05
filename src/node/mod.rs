pub mod logical;
pub use logical::{AsAny, Logical, LogicalKind, InvalidOperationError};

pub mod gate;
pub use gate::{Gate, GateKind};

mod wire;
pub use wire::Wire;

mod primitive;
pub use primitive::{Primitive, PrimitiveKind};

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

const LINE_THICKNESS: f32 = 3.0;
const LINE_DEFAULT_COLOR: Color32 = Color32::from_rgb(0, 0, 0);


