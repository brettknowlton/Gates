pub mod gate;
use crate::gate::OutputClick;

use super::*;
pub use gate::Gate;
use serde;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

use std::error::Error;

use egui::{Color32, Direction, Layout, Pos2, Response, Sense, Stroke, Ui, Widget};

mod output;
pub use output::Output;

const LINE_THICKNESS: f32 = 2.0;

pub enum Logicals {
    Gate(GateType),
    Wire,
}

#[derive(Debug)]
struct InvalidOperationError;
impl Error for InvalidOperationError {}

impl Display for InvalidOperationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Cannot set position for this type")
    }
}

pub trait Logical {
    fn tick(self);
    fn get_position(&self) -> Pos2;
    fn set_position(&mut self, pos: Pos2) -> Result<(), Box<dyn Error>>;

    fn get_kind(&self) -> Logicals;

    fn show(&mut self, ui: &mut Ui, on_output_click: &mut Option<OutputClick>) -> Response;
    
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
pub enum GateType {
    #[default]
    None,
    Primitive(PrimitiveKind),
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

impl GateType {
    pub fn lookup_kind(name: &str) -> GateType {
        match name {
            "TOGGLE" => GateType::Primitive(PrimitiveKind::TOGGLE),
            "LIGHT" => GateType::Primitive(PrimitiveKind::LIGHT),
            "BUFFER" => GateType::Primitive(PrimitiveKind::BUFFER),
            "NOT" => GateType::Primitive(PrimitiveKind::NOT),
            "OR" => GateType::Primitive(PrimitiveKind::OR),
            "AND" => GateType::Primitive(PrimitiveKind::AND),
            _ => GateType::None,
        }
    }

    pub fn is_primitive(&self) -> bool {
        matches!(self, GateType::Primitive(_))
    }

    pub fn primitive_kind(&self) -> Option<PrimitiveKind> {
        if let GateType::Primitive(kind) = self {
            Some(kind.clone())
        } else {
            None
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
pub enum PrimitiveKind {
    #[default]
    None,
    TOGGLE,
    LIGHT,
    BUFFER,
    NOT,
    OR,
    AND,
}

impl Display for PrimitiveKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveKind::None => write!(f, "None"),
            PrimitiveKind::TOGGLE => write!(f, "TOGGLE"),
            PrimitiveKind::LIGHT => write!(f, "LIGHT"),
            PrimitiveKind::BUFFER => write!(f, "BUFFER"),
            PrimitiveKind::NOT => write!(f, "NOT"),
            PrimitiveKind::OR => write!(f, "OR"),
            PrimitiveKind::AND => write!(f, "AND"),
        }
    }
}

impl Widget for PrimitiveKind {
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
    pub fn kind_as_str(self) -> String {
        self.kind.to_string().clone()
    }

    pub fn from_values(label: &str, num_inputs: usize, num_outputs: usize) -> Primitive {
        let kind: GateType;
        match label {
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
        let var = Primitive {
            label: label.to_string(),
            n_ins: num_inputs,
            n_outs: num_outputs,
            kind,
        };
        var
    }

    pub fn make_toolbox_widget(&self) -> egui::Button<'static> {
        //square selectable button that takes a label and number of inputs and outputs
        let var = egui::Button::selectable(
            false, // or set to true if you want it selected by default
            self.label.clone(),
        )
        .min_size(egui::vec2(110., 110.))
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

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)]
pub struct Input {
    pub id: usize,
    pub name: Option<String>,
    pub parent: Option<Gate>, // Optional parent gate, if this input belongs to a gate

    pub signal: bool,
    pub connected: bool,
}

impl Input {
    pub fn new(n: usize, parent_gate: &Gate) -> Self {
        Input {
            id: n,
            name: None,
            parent: Some(parent_gate.clone()), // Optional parent gate, if this input belongs to a gate

            signal: false,
            connected: false,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, Clone, Debug)]
struct WireLine {
    p1: Pos2,
    p2: Pos2,
    color: Color32,
    smoothing: bool,
}
impl WireLine {
    pub fn new(p1: Pos2, p2: Pos2, color: Color32, smoothing: bool) -> Self {
        WireLine {
            p1,
            p2,
            color,
            smoothing,
        }
    }
}

impl Hash for WireLine {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.p1.x.to_bits().hash(state);
        self.p1.y.to_bits().hash(state);
        self.p2.x.to_bits().hash(state);
        self.p2.y.to_bits().hash(state);
        self.color.hash(state);
        self.smoothing.hash(state);
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Wire {
    signal: bool,
    pub source_gate_id: usize,      // ID of the source gate
    pub source_output_index: usize, // Index of the output on the source gate
    dest: Option<Input>,
    line: WireLine,
    being_held: bool, // Indicates if the wire is currently being dragged
    position: Pos2, // World position of the wire (for tracking and transforms)
    #[serde(skip)]
    rect: egui::Rect, // Bounding rect that encompasses the wire line
}

impl Default for Wire {
    fn default() -> Self {
        Wire {
            signal: false,
            source_gate_id: 0,
            source_output_index: 0,
            dest: None,
            line: WireLine::default(),
            being_held: false,
            position: Pos2::default(),
            rect: egui::Rect::NOTHING,
        }
    }
}

impl Hash for Wire {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.signal.hash(state);
        self.source_gate_id.hash(state);
        self.source_output_index.hash(state);
        self.dest.hash(state);
        self.line.hash(state);
        self.being_held.hash(state);
        self.position.x.to_bits().hash(state);
        self.position.y.to_bits().hash(state);
        // Note: rect is not hashed since it's computed and skipped in serialization
    }
}

impl Wire {
    pub fn new(source_gate_id: usize, source_output_index: usize, pos1: Pos2, pos2: Pos2, color: Color32, smoothing: bool) -> Self {
        // Calculate bounding rect that encompasses both points
        let min_x = pos1.x.min(pos2.x);
        let max_x = pos1.x.max(pos2.x);
        let min_y = pos1.y.min(pos2.y);
        let max_y = pos1.y.max(pos2.y);
        
        // Add some padding to ensure the line is fully contained
        let padding = 5.0;
        let rect = egui::Rect::from_min_max(
            egui::pos2(min_x - padding, min_y - padding),
            egui::pos2(max_x + padding, max_y + padding),
        );
        
        // Position is the center of the wire for tracking purposes
        let position = egui::pos2((pos1.x + pos2.x) / 2.0, (pos1.y + pos2.y) / 2.0);

        Wire {
            signal: false,
            source_gate_id,
            source_output_index,
            dest: None,
            line: WireLine::new(pos1, pos2, color, smoothing),
            being_held: true, //upon creation the wire is being held by the cursor until it is dropped/click again.
            position,
            rect,
        }
    }
    
    /// Stops the wire from being held (completes the wire placement)
    pub fn release(&mut self) {
        self.being_held = false;
    }
    
    /// Updates the wire's p1 position based on the source gate's output position
    pub fn update_source_position(&mut self, source_position: Pos2) {
        self.line.p1 = source_position;
    }
    
    /// Recalculates the bounding rect and position based on current line points
    fn update_rect_and_position(&mut self) {
        let min_x = self.line.p1.x.min(self.line.p2.x);
        let max_x = self.line.p1.x.max(self.line.p2.x);
        let min_y = self.line.p1.y.min(self.line.p2.y);
        let max_y = self.line.p1.y.max(self.line.p2.y);
        
        let padding = 5.0;
        self.rect = egui::Rect::from_min_max(
            egui::pos2(min_x - padding, min_y - padding),
            egui::pos2(max_x + padding, max_y + padding),
        );
        
        self.position = egui::pos2((self.line.p1.x + self.line.p2.x) / 2.0, (self.line.p1.y + self.line.p2.y) / 2.0);
    }

    fn delete(mut self) {
        //disconnect both output and input
        if let Some(dest) = &mut self.dest {
            dest.connected = false;
        }
        // Note: We can't directly access the source gate from here anymore
        // This would need to be handled by the containing system
        self.signal = false;
        self.dest = None;
    }

    fn get_kind(&self) -> Logicals {
        Logicals::Wire
    }

    fn on(mut self) {
        self.signal = true;
    }

    fn off(mut self) {
        self.signal = false
    }
}

impl Logical for Wire {
    fn tick(self) {
        if let Some(mut out) = self.dest {
            if self.signal {
                out.signal = true;
            } else {
                out.signal = false;
            }
        }
    }
    fn get_position(&self) -> Pos2 {
        self.position
    }

    fn set_position(&mut self, pos: Pos2) -> Result<(), Box<dyn Error>> {
        // For wires, we generally don't want to move them directly via set_position
        // since p1 should always follow the source output. However, we can move p2
        // if the wire is being held or move both points if it's a free-floating wire.
        
        if self.being_held {
            // If being held, only move p2 (the free end)
            let offset = pos - self.position;
            self.line.p2 += offset;
        } else {
            // If not being held, this might be a completed wire that should move both points
            let offset = pos - self.position;
            self.line.p1 += offset;
            self.line.p2 += offset;
        }
        
        // Update position and rect using helper method
        self.update_rect_and_position();
        
        Ok(())
    }
    fn get_kind(&self) -> Logicals {
        Logicals::Wire
    }

    fn show(&mut self, ui: &mut Ui, _on_output_click: &mut Option<OutputClick>) -> Response {
        // Note: The caller will need to update p1 before calling show()
        // since we can't access the gates list from here
        
        // If being held, update the second point to follow the cursor
        if self.being_held {
            if let Some(cursor_pos) = ui.input(|i| i.pointer.hover_pos()) {
                self.line.p2 = cursor_pos;
            }
        }
        
        // Update position and rect after updating line points
        self.update_rect_and_position();
        
        // Allocate the rect area but with no sense (no interaction)
        let (_rect, response) = ui.allocate_exact_size(self.rect.size(), Sense::hover());
        
        // Draw the wire line using the painter
        ui.painter().line_segment(
            [self.line.p1, self.line.p2],
            Stroke::new(LINE_THICKNESS, self.line.color),
        );
        
        response
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
