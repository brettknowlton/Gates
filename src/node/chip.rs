use crossbeam::channel::Sender;

use crate::{gate::GridVec2, MyApp};

use super::*;



#[derive(serde::Deserialize, serde::Serialize, Default, Clone, Debug)]
pub struct ChipDefenition {
    pub id: usize,
    pub name: String,
    pub position: Option<GridVec2>, // Position in the grid

    pub sub_gates: HashMap<usize, Gate>, // Sub-gates within the chip
    pub sub_wires: HashMap<usize, Wire>, // Wires within the chip
    pub sub_inputs: HashMap<usize, Input>, // Inputs within the chip
    pub sub_outputs: HashMap<usize, Output>, // Outputs within the chip
    pub sub_chips: HashMap<usize, ChipDefenition>, // Sub-chips within the chip

    pub n_in: usize,
    pub chip_ins: HashMap<usize, bool>, //bool represents the interpreted input state, this will be passed to the gate on its tick() function

    pub n_out: usize,
    pub chip_outs: HashMap<usize, bool>,

}

impl ChipDefenition{
    pub fn create_blank_chip(name: String) -> Self {
        ChipDefenition {
            id: MyApp::next_id(),
            name,
            position: None,
            sub_gates: HashMap::new(),
            sub_wires: HashMap::new(),
            sub_inputs: HashMap::new(),
            sub_outputs: HashMap::new(),
            sub_chips: HashMap::new(),



            n_in: 0,
            chip_ins: HashMap::new(),
            n_out: 0,
            chip_outs: HashMap::new(),
        }
        
    }

    pub fn make_toolbox_widget(&self) -> Button<'static> {
        //square selectable button that takes a label and number of inputs and outputs
        let var = Button::selectable(
            false, // or set to true if you want it selected by default
            self.name.clone(),
        )
        .min_size(vec2(110., 110.))
        .corner_radius(10.)
        .sense(Sense::drag())
        .sense(Sense::click());
        return var;
    }

    fn add_sub_gate(&mut self, gate: Gate) {
        let id = gate.get_id();
        self.sub_gates.insert(id, gate);
    }

    fn next_chip_id() -> usize {
        MyApp::next_id()
    }

    pub fn from_live_data(board_data: &HashMap<usize, Box<dyn Logical>>, name: String) -> Self {
        //for item in board_data

        //try to downcast to each logical type wire, gate, in, or out, and add the item to this chip's correct hashmap
        let mut chip = ChipDefenition::create_blank_chip(name);
        let io= ChipDefenition::get_io_from_gates(board_data);

        chip.chip_ins = io.0;
        chip.chip_outs = io.1;
        chip.n_in = chip.chip_ins.len();
        chip.n_out = chip.chip_outs.len();


        for (id, item) in board_data {
            match item.get_kind() {

                LogicalKind::Gate(_) => {
                    if let Some(gate) = item.as_any().downcast_ref::<Gate>() {
                        chip.add_sub_gate(gate.clone());
                    }
                }
                LogicalKind::Wire => {
                    if let Some(wire) = item.as_any().downcast_ref::<Wire>() {
                        chip.sub_wires.insert(*id, wire.clone());
                    }
                }
                LogicalKind::IO(IOKind::Input) => {
                    if let Some(input) = item.as_any().downcast_ref::<Input>() {
                        chip.sub_inputs.insert(*id, input.clone());
                    }
                }
                LogicalKind::IO(IOKind::Output) => {
                    if let Some(output) = item.as_any().downcast_ref::<Output>() {
                        chip.sub_outputs.insert(*id, output.clone());
                    }
                }
                _ => {
                    println!("Unknown logical kind: {:?}", item.get_kind());
                }
            }
        }

        chip
    }


    fn get_io_from_gates(board_data: &HashMap<usize, Box<dyn Logical>>)-> (HashMap<usize, bool>, HashMap<usize, bool>) {
        let (ins, outs) = board_data.iter().fold(
            (HashMap::new(), HashMap::new()),
            |(mut ins, mut outs), (id, item)| {
                match item.get_kind() {
                    LogicalKind::Gate(GateKind::Primitive(PrimitiveKind::TOGGLE)) => {
                        if let Some(input) = item.as_any().downcast_ref::<Input>() {
                            ins.insert(*id, input.signal);
                        }
                    }
                    LogicalKind::IO(IOKind::Output) => {
                        if let Some(output) = item.as_any().downcast_ref::<Output>() {
                            outs.insert(*id, output.signal);
                        }
                    }
                    _ => {}
                }
                (ins, outs)
            },
        );
        (ins, outs)
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
        LogicalKind::Gate(GateKind::Custom(self.name.clone()))
    }
}