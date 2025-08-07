use super::*;

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PrimitiveTemplate {
    pub label: String,
    pub kind: PrimitiveKind,
    pub n_ins: usize,
    pub n_outs: usize,
}

impl PrimitiveTemplate {
    pub fn from_values(label: &str, num_inputs: usize, num_outputs: usize) -> PrimitiveTemplate {
        let kind: PrimitiveKind;
        match label {
            "HI-SIGNAL" => {
                kind = PrimitiveKind::HISIGNAL; // Assuming HI-SIGNAL is a type of pulse
            }
            "LO-SIGNAL" => {
                kind = PrimitiveKind::LOSIGNAL; // Assuming LO-SIGNAL is a type of pulse
            }
            "PULSE" => {
                kind = PrimitiveKind::PULSE;
            }
            "TOGGLE" => {
                kind = PrimitiveKind::TOGGLE;
            }
            "LIGHT" => {
                kind = PrimitiveKind::LIGHT;
            }
            "BUFFER" => {
                kind = PrimitiveKind::BUFFER;
            }
            "NOT" => {
                kind = PrimitiveKind::NOT;
            }
            "OR" => {
                kind = PrimitiveKind::OR;
            }
            "AND" => {
                kind = PrimitiveKind::AND;
            }
            "XOR" => {
                kind = PrimitiveKind::XOR;
            }
            "NAND" => {
                kind = PrimitiveKind::NAND;
            }
            "NOR" => {
                kind = PrimitiveKind::NOR;
            }
            _ => {
                kind = PrimitiveKind::None;
            }
        }
        let var = PrimitiveTemplate {
            label: label.to_string(),
            n_ins: num_inputs,
            n_outs: num_outputs,
            kind,
        };
        var
    }

    pub fn make_toolbox_widget(&self) -> Button<'static> {
        //square selectable button that takes a label and number of inputs and outputs
        let var = Button::selectable(
            false, // or set to true if you want it selected by default
            self.label.clone(),
        )
        .min_size(vec2(110., 110.))
        .corner_radius(10.)
        .sense(Sense::drag())
        .sense(Sense::click());
        return var;
    }
}


#[derive(serde::Deserialize, serde::Serialize, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveKind {
    #[default]
    None,
    HISIGNAL,
    LOSIGNAL,
    PULSE,
    TOGGLE,
    LIGHT,
    BUFFER,
    NOT,
    OR,
    AND,
    XOR,
    NAND,
    NOR,
}

impl PrimitiveKind {
    pub fn tick(
        self,
        gate: &mut Gate,
        ins: HashMap<usize, bool>,
    ) -> Result<HashMap<usize, bool>, Box<dyn Error>> {
        // println!("Ticking primitive type: {}", self);

        if ins.len() > self.get_n_desired_inputs() {
            return Err(format!("{} requires exactly {} or less inputs", self, self.get_n_desired_inputs()).into());
        }

        match self {
            PrimitiveKind::HISIGNAL => {
                //HI-SIGNAL always outputs true
                // println!("Ticking HISIGNAL, setting output to true");
                let (out_id, _) = gate.outs.iter().next().unwrap();
                gate.state = true; // Set gate state to true
                Ok(HashMap::from([(*out_id, true)]))
            }
            PrimitiveKind::LOSIGNAL => {
                // LO-SIGNAL always outputs false
                let (out_id, _) = gate.outs.iter().next().unwrap();
                gate.state = false; // Set gate state to false
                Ok(HashMap::from([(*out_id, false)]))
            }
            PrimitiveKind::BUFFER => {
                // BUFFER logic, for example, passes the signal through
                //ensure only one input
                let (out_id, _) = gate.outs.iter().next().unwrap(); //get the  id of the (only) output
                if let Some((_, signal)) = ins.iter().next() {
                    Ok(HashMap::from([(*out_id, *signal)])) // Assuming single output at index 0
                } else {
                    return Err("Input signal not found".into());
                }
            }
            PrimitiveKind::LIGHT => {
                gate.state = ins.values().next().cloned().unwrap_or(false); // Set gate state based on input
                Ok(HashMap::new())// No output, just update state
            }
            PrimitiveKind::PULSE => {
                // 1-Tick pulse creator, on rising edge of any pulse (or click) it will send a true signal for one tick
                // and then set self state to false
                let (out_id, _) = gate.outs.iter().next().expect("PULSE was ticked but did not have an output"); //get the  id of the (only) output
                if gate.state {//set state to false and send a true signal
                    gate.state = false;
                    Ok(HashMap::from([(*out_id, true)])) // Assuming single output at index 0
                } else {
                    //send a false signal
                    Ok(HashMap::from([(*out_id, false)])) // Assuming single output at index 0
                }
            }
            PrimitiveKind::TOGGLE => {
                // pretty much the same as PULSE but doesnt handle its own state, instead state is handled externally by user input
                let (out_id, _) = gate.outs.iter().next().expect("TOGGLE was ticked but did not have an output");
                Ok(HashMap::from([(*out_id, gate.state)])) // Assuming single output at index 0
            }

            PrimitiveKind::NOT => {
                // NOT logic, inverts the input signal
                let (out_id, _) = gate.outs.iter().next().expect("NOT was ticked but did not have an output"); //get the  id of the (only) output
                if let Some((_, signal)) = ins.iter().next() {
                    Ok(HashMap::from([(*out_id, !signal)])) // Assuming single output at index 0
                } else {
                    return Err("Input signal not found".into());
                }
            }
            PrimitiveKind::OR => {
                // OR logic, returns true if any input is true
                let (out_id, _) = gate.outs.iter().next().expect("OR was ticked but did not have an output"); //get the  id of the (only) output
                let result = ins.values().any(|&v| v);
                gate.state = result; // Set gate state based on input
                Ok(HashMap::from([(*out_id, result)])) // Assuming single output at index 0
            }
            PrimitiveKind::AND => {
                // AND logic, returns true if all inputs are true
                let (out_id, _) = gate.outs.iter().next().expect("AND was ticked but did not have an output"); //get the  id of the (only) output
                let result = ins.values().all(|&v| v);
                gate.state = result; // Set gate state based on input
                Ok(HashMap::from([(*out_id, result)])) // Assuming single output at index 0
            }
            PrimitiveKind::XOR => {
                // XOR logic, returns true if an odd number of inputs are true
                let (out_id, _) = gate.outs.iter().next().expect("XOR was ticked but did not have an output"); //get the  id of the (only) output
                let result = ins.values().filter(|&&v| v).count() % 2 == 1;
                gate.state = result; // Set gate state based on input
                Ok(HashMap::from([(*out_id, result)])) // Assuming single output at index 0
            }
            PrimitiveKind::NAND => {
                // NAND logic, returns true if not all inputs are true
                let (out_id, _) = gate.outs.iter().next().expect("NAND was ticked but did not have an output"); //get the  id of the (only) output
                let result = !ins.values().all(|&v| v);
                gate.state = result; // Set gate state based on input
                Ok(HashMap::from([(*out_id, result)])) // Assuming single output at index 0
            }
            PrimitiveKind::NOR => {
                // NOR logic, returns true if all inputs are false
                let (out_id, _) = gate.outs.iter().next().expect("NOR was ticked but did not have an output"); //get the  id of the (only) output
                let result = !ins.values().any(|&v| v);
                gate.state = result; // Set gate state based on input
                Ok(HashMap::from([(*out_id, result)])) // Assuming single output at index 0
            }
            _ => Err(Box::new(InvalidOperationError("Could not determine primitive type".to_string()))), // Other types not implemented yet
        }
    }

    pub fn get_gate_kind(&self) -> GateKind {
        GateKind::Primitive(self.clone())
    }
    pub fn get_logical_kind(&self) ->LogicalKind{
        LogicalKind::Gate(self.get_gate_kind())
    }
    fn get_n_desired_inputs(&self) -> usize {
        match self {
            PrimitiveKind::HISIGNAL | PrimitiveKind::LOSIGNAL | PrimitiveKind::PULSE | PrimitiveKind::TOGGLE => 0,
            PrimitiveKind::LIGHT => 1,
            PrimitiveKind::BUFFER | PrimitiveKind::NOT => 1,
            PrimitiveKind::OR | PrimitiveKind::AND | PrimitiveKind::XOR | PrimitiveKind::NAND | PrimitiveKind::NOR => 2,
            PrimitiveKind::None => 0, // Default case
        }
    }
}

impl Display for PrimitiveKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveKind::None => write!(f, "None"),
            PrimitiveKind::HISIGNAL => write!(f, "HI-SIGNAL"),
            PrimitiveKind::LOSIGNAL => write!(f, "LO-SIGNAL"),
            PrimitiveKind::PULSE => write!(f, "PULSE"),
            PrimitiveKind::TOGGLE => write!(f, "TOGGLE"),
            PrimitiveKind::LIGHT => write!(f, "LIGHT"),
            PrimitiveKind::BUFFER => write!(f, "BUFFER"),
            PrimitiveKind::NOT => write!(f, "NOT"),
            PrimitiveKind::OR => write!(f, "OR"),
            PrimitiveKind::AND => write!(f, "AND"),
            PrimitiveKind::XOR => write!(f, "XOR"),
            PrimitiveKind::NAND => write!(f, "NAND"),
            PrimitiveKind::NOR => write!(f, "NOR"),
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
