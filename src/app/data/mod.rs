use super::*;    

mod theme;
pub use theme::SkeletonTheme;
pub struct Data{
    pub live_data: HashMap<usize, Box<dyn Logical>>, // (id, position, id)
    pub available_themes: HashMap<String, SkeletonTheme>,
    pub color_values: HashMap<String, Color32>,

    pub prim_templates: Vec<PrimitiveTemplate>,
    pub saved_chips: Vec<ChipDefenition>,
}


impl Data{
    pub fn new() -> Self {
        Data {
            live_data: HashMap::new(),

            available_themes: HashMap::new(),
            color_values: HashMap::new(),

            prim_templates: Vec::new(),
            saved_chips: Vec::new(),
        }
    }

    pub fn load_data(&mut self) -> &mut Self {
        self.load_themes();

        self.prim_templates = data::Data::load_prims();
        println!("Loaded prims: {}", self.prim_templates.len());

        self.saved_chips = data::Data::load_chips();
        println!("Loaded chips: {}", self.saved_chips.len());

        self
    }

    pub fn load_themes(&mut self) {
        //for item in ../../assets/*
        let theme_dir = PathBuf::from("assets/");
        if let Ok(entries) = std::fs::read_dir(theme_dir) {
            for entry in entries.flatten() {
                if let Some(path) = entry.path().to_str() {
                    if path.ends_with(".css") {
                        if let Ok(theme) = SkeletonTheme::from_css_file(path) {
                            println!("Loaded theme: {}", path);
                            self.available_themes.insert(theme.name.clone(), theme);
                        } else {
                            eprintln!("Failed to load theme from: {}", path);
                        }
                    }
                }
            }
        } else {
            eprintln!("Failed to read theme directory");
        }
    }


    ///loads live_data from a file (currently only returns a vector of gates)
    pub fn load_gate_data(path: &str) -> Vec<Gate> {
        //for every line in the file, create a gate
        let data = std::fs::read_to_string(path).unwrap();
        let lines = data.lines();
        let mut gates = Vec::<Gate>::new();
        for l in lines {
            let split = l.split(":");
            let mut parts = Vec::<&str>::new();

            for n in split {
                parts.push(n)
            }
            if parts.len() < 3 {
                continue; // skip invalid lines
            }
            let label = parts[0].to_string();
            let n_ins = parts[1]
                .split("[")
                .nth(1)
                .and_then(|s| s.split("]").next())
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            let n_outs = parts[2]
                .split("[")
                .nth(1)
                .and_then(|s| s.split("]").next())
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            println!(
                "Creating gate: {} with {} inputs and {} outputs",
                label, n_ins, n_outs
            );
            println!("Parts: {:?}", parts);

            let gate = Gate::generate(label, n_ins, n_outs);
            gates.push(gate);
        }
        gates
    }

    ///loads primitives from a file that is used in the primitive menu
    ///returns a vector of Primitive
    ///each line in the file should be in the format: name:n_ins:n_outs
    pub fn load_prims() -> Vec<PrimitiveTemplate> {
        let mut prims: Vec<PrimitiveTemplate> = Vec::new();

        //read saves directory for each file add a gate to the vector
        let data = std::fs::read_to_string("./saves/primitives").unwrap();
        let lines = data.lines();
        for l in lines {
            print!("Loading gate: ");
            let split = l.split(":");
            let mut parts = Vec::<&str>::new();

            for n in split {
                parts.push(n)
            }
            let n1 = parts[1].parse::<usize>();
            let n2 = parts[2].parse::<usize>();
            if let Ok(n1) = n1 {
                if let Ok(n2) = n2 {
                    let new_gate = PrimitiveTemplate::from_values(parts[0], n1, n2);
                    prims.push(new_gate);
                }
            }
        }
        println!("Loaded {} gates", prims.len());
        prims
    }

    ///loads saved chips from the saves directory
    ///returns a vector of Primitive right now but this NEEDS to be changed to a vector of Gate(Custom)
    pub fn load_chips() -> Vec<ChipDefenition> {
        let gates = Vec::<ChipDefenition>::new();

        //read saves directory for each file add a gate to the vector
        let dir = std::fs::read_dir("./saves").unwrap();
        for entry in dir {
            print!("Loading gate: ");
            let entry = entry.unwrap();
            if entry.path().is_file() {
                println!("{}", entry.path().display());
                // Check if the file name ends with ".gate"
                let file_name = entry.file_name().into_string().unwrap();
                if file_name.ends_with(".chip") {
                    // Load the gate from the file
                    // let gate_template = Primitive::new(file_name);
                    // gates.push(gate_template);
                }
            }
        }
        println!("Loaded {} gates", gates.len());
        gates
    }

    pub fn save_chip(data: &HashMap<usize, Box<dyn Logical>>) {
        // Save each gate to a file in the saves directory, overwriting existing files
        std::fs::create_dir_all("./saves").unwrap();
        for (id, item) in data {
            //match on the item type and serialize all gates and wires
            //gate
            if let Some(gate) = item.as_any().downcast_ref::<Gate>() {
                let file_path = format!("./saves/{}.gate", id);
                let serialized_gate = serde_json::to_string(gate).unwrap();
                std::fs::write(file_path, serialized_gate).unwrap();
            }
            //wire
            else if let Some(wire) = item.as_any().downcast_ref::<Wire>() {
                let file_path = format!("./saves/{}.wire", id);
                let serialized_wire = serde_json::to_string(wire).unwrap();
                std::fs::write(file_path, serialized_wire).unwrap();
            }
        }
    }


    pub fn update_logicals(&mut self, ctx: &Context) {
        // Update the logical states of all gates and wires

        // Step 1: Collect all inputs and their current states
        let input_states = self.collect_input_states();

        // Step 2: Process all gates and collect their outputs
        let gate_outputs = self.process_gates(&input_states);

        // Step 3: Update outputs and propagate through wires
        let wire_signals = self.update_outputs_and_wires(&gate_outputs);

        // Step 4: Apply wire signals to inputs
        self.apply_wire_signals_to_inputs(&wire_signals);

        ctx.request_repaint();
    }

    fn collect_input_states(&self) -> HashMap<usize, bool> {
        self.live_data
            .iter()
            .filter_map(|(id, item)| {
                item.as_any()
                    .downcast_ref::<Input>()
                    .map(|input| (*id, input.signal))
            })
            .collect()
    }

    fn process_gates(&mut self, input_states: &HashMap<usize, bool>) -> HashMap<usize, bool> {
        let mut gate_outputs = HashMap::new();

        for (_gate_id, item) in self.live_data.iter_mut() {
            if let Some(gate) = item.as_any_mut().downcast_mut::<Gate>() {
                // Gather inputs for this specific gate
                let gate_inputs: HashMap<usize, bool> = gate
                    .ins
                    .keys()
                    .filter_map(|input_id| {
                        input_states
                            .get(input_id)
                            .map(|&signal| (*input_id, signal))
                    })
                    .collect();

                // Process gate and collect outputs
                if let Ok(outputs) = item.tick(gate_inputs) {
                    gate_outputs.extend(outputs);
                }
            }
        }

        gate_outputs
    }

    fn update_outputs_and_wires(
        &mut self,
        gate_outputs: &HashMap<usize, bool>,
    ) -> HashMap<usize, bool> {
        let mut input_signals = HashMap::new();

        for (&output_id, &signal) in gate_outputs {
            // Update the output signal
            if let Some(output) = self.get_output_mut(output_id) {
                output.signal = signal;

                // Process all wires connected to this output
                let wire_ids = output.out_wire_ids.clone();
                for wire_id in wire_ids {
                    if let Some(wire) = self.get_wire_mut(wire_id) {
                        wire.set_signal(signal);

                        // If wire has a destination, record the signal for that input
                        if let Some(dest_input_id) = wire.dest {
                            input_signals.insert(dest_input_id, signal);
                        }
                    }
                }
            }
        }

        input_signals
    }

    fn apply_wire_signals_to_inputs(&mut self, wire_signals: &HashMap<usize, bool>) {
        //every input must be updated, even if signals are not present in wire_signals
        for (input_id, input) in self.live_data.iter_mut() {
            if let Some(input) = input.as_any_mut().downcast_mut::<Input>() {
                // Set the input signal based on wire signals or default to false
                input.signal = wire_signals.get(&input_id).cloned().unwrap_or(false);
            }
        }
    }

    // Helper methods for cleaner access
    fn get_output_mut(&mut self, id: usize) -> Option<&mut Output> {
        self.live_data
            .get_mut(&id)?
            .as_any_mut()
            .downcast_mut::<Output>()
    }

    fn get_wire_mut(&mut self, id: usize) -> Option<&mut Wire> {
        self.live_data
            .get_mut(&id)?
            .as_any_mut()
            .downcast_mut::<Wire>()
    }

    fn get_input_mut(&mut self, id: usize) -> Option<&mut Input> {
        self.live_data
            .get_mut(&id)?
            .as_any_mut()
            .downcast_mut::<Input>()
    }


}

    