use super::*;    
pub enum Data{
    Load,
    Save
}

impl Data{

    ///loads live_data from a file (currently only returns a vector of gates)
    pub fn load_data(path: &str) -> Vec<Gate> {
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
    pub fn load_chips() -> Vec<PrimitiveTemplate> {
        let gates = Vec::<PrimitiveTemplate>::new();

        //read saves directory for each file add a gate to the vector
        let dir = std::fs::read_dir("./saves").unwrap();
        for entry in dir {
            print!("Loading gate: ");
            let entry = entry.unwrap();
            if entry.path().is_file() {
                println!("{}", entry.path().display());
                // Check if the file name ends with ".gate"
                let file_name = entry.file_name().into_string().unwrap();
                if file_name.ends_with(".gate") {
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
}

    