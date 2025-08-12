use crossbeam::channel::Sender;

use crate::gate::GridVec2;

use super::*;




pub struct ChipDefenition {
    pub id: usize,
    pub name: String,
    pub position: Option<GridVec2>, // Position in the grid

    pub sub_data: HashMap<usize, Box<dyn Logical>>, // Sub-gates within the chip

    pub n_in: usize,
    pub ins: HashMap<usize, bool>, //bool represents the interpreted input state, this will be passed to the gate on its tick() function

    pub n_out: usize,
    pub outs: HashMap<usize, bool>,

}

impl ChipDefenition{
    fn create_chip_from_board(board_data: HashMap<usize, Box<dyn Logical>>, name: String) -> Self {


        let mut c= ChipDefenition {
            id: usize::MAX, // Default ID, should be set later
            name,
            position: None,
            sub_data: board_data,
            n_in: 0,
            ins: HashMap::new(),
            n_out: 0,
            outs: HashMap::new(),
        };
        c= c.create_io_from_primitives();
        c
    }


    fn create_io_from_primitives(mut self)->Self{
        let mut ins= HashMap::new();
        let mut outs= HashMap::new();

        for (id, logical) in &self.sub_data {
            if logical.get_kind().is_gate() {
                match logical.get_kind() {
                    LogicalKind::Gate(GateKind::Primitive(primitive_kind)) => {
                        match primitive_kind {
                            PrimitiveKind::TOGGLE => {
                                // Create an input for the toggle
                                let state= logical.as_any().downcast_ref::<Gate>().unwrap().state;
                                ins.insert(id.clone(), state); // Default to false
                            }
                            PrimitiveKind::LIGHT => {
                                // Create an output for the light
                                let state = logical.as_any().downcast_ref::<Gate>().unwrap().state;
                                outs.insert(id.clone(), state); // Default to false
                            }
                            _ => {}
                        }
                    },
                    _ => {}
                }
            } 
        }
        self.n_in = ins.len();
        self.ins = ins;
        self.n_out = outs.len();
        self.outs = outs;
        self
        
    }
}


impl Logical for ChipDefenition {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_position(&self) -> Result<Pos2, Box<dyn Error>> {
        if let Some(position) = &self.position {
            Ok(position.to_pos2())
        } else {
            Err(Box::new(InvalidOperationError::new("Position not set for ChipDefenition")))
        }
    }

    fn set_position(&mut self, pos: Pos2) -> Result<(), Box<dyn Error>> {
        self.position = Some(GridVec2::from(pos));
        Ok(())
    }

    

    fn show(
        &self,
        ui: &mut Ui,
        _sender: Sender<UiEvent>,
        _live_data: &HashMap<usize, Box<dyn Logical>>,
        _colors: &HashMap<String, Color32>,
    ) -> eframe::egui::Response {
        let response = ui.label(self.name.clone());
        response
    }


    fn get_kind(&self) -> LogicalKind {
        LogicalKind::Chip(self.name.clone())
    }
}