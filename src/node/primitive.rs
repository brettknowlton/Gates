use super::*;

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Primitive {
    pub label: String,
    pub kind: GateKind,
    pub n_ins: usize,
    pub n_outs: usize,
}

impl Primitive {
    pub fn from_values(label: &str, num_inputs: usize, num_outputs: usize) -> Primitive {
        let kind: GateKind;
        match label {
            "HI-SIGNAL" => {
                kind = GateKind::Primitive(PrimitiveKind::HISIGNAL); // Assuming HI-SIGNAL is a type of pulse
            }
            "LO-SIGNAL" => {
                kind = GateKind::Primitive(PrimitiveKind::LOSIGNAL); // Assuming LO-SIGNAL is a type of pulse
            }
            "PULSE" => {
                kind = GateKind::Primitive(PrimitiveKind::PULSE);
            }
            "LIGHT" => {
                kind = GateKind::Primitive(PrimitiveKind::LIGHT);
            }
            "BUFFER" => {
                kind = GateKind::Primitive(PrimitiveKind::BUFFER);
            }
            "NOT" => {
                kind = GateKind::Primitive(PrimitiveKind::NOT);
            }
            "OR" => {
                kind = GateKind::Primitive(PrimitiveKind::OR);
            }
            "AND" => {
                kind = GateKind::Primitive(PrimitiveKind::AND);
            }
            _ => {
                kind = GateKind::Primitive(PrimitiveKind::None);
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
    LIGHT,
    BUFFER,
    NOT,
    OR,
    AND,
}

impl PrimitiveKind {
    pub fn tick(
        self,
        gate: &mut Gate,
        ins: HashMap<usize, bool>,
    ) -> Result<HashMap<usize, bool>, Box<dyn Error>> {
        println!("Ticking primitive type: {}", self);

        if ins.len() > self.get_n_desired_inputs() {
            return Err(format!("{} requires exactly {} or less inputs", self, self.get_n_desired_inputs()).into());
        }

        match self {
            PrimitiveKind::HISIGNAL => {
                //HI-SIGNAL always outputs true
                let (id, _) = gate.outs.iter().next().unwrap();
                gate.state = true; // Set gate state to true
                Ok(HashMap::from([(*id, true)]))
            }
            PrimitiveKind::LOSIGNAL => {
                // LO-SIGNAL always outputs false
                let (id, _) = gate.outs.iter().next().unwrap();
                gate.state = false; // Set gate state to false
                Ok(HashMap::from([(*id, false)]))
            }
            PrimitiveKind::BUFFER => {
                // BUFFER logic, for example, passes the signal through
                //ensure only one input
                let (out_id, _) = gate.outs.iter().next().expect("BUFFER was ticked but did not have an output"); //get the  id of the (only) output
                if let Some((_, signal)) = ins.iter().next() {
                    Ok(HashMap::from([(*out_id, *signal)])) // Assuming single output at index 0
                } else {
                    return Err("Input signal not found".into());
                }
            }
            PrimitiveKind::LIGHT => {
                // LIGHT has no outputs, for example, always outputs empty map
                gate.state = ins.values().next().cloned().unwrap_or(false); // Set gate state based on input
                Ok(HashMap::new())
            }
            PrimitiveKind::PULSE => {

                let (out_id, _) = gate.outs.iter().next().expect("PULSE was ticked but did not have an output"); //get the  id of the (only) output
                if gate.state {
                    //set state to false and send a true signal
                    gate.state = false;
                    Ok(HashMap::from([(*out_id, true)])) // Assuming single output at index 0
                } else {
                    //send a false signal
                    Ok(HashMap::from([(*out_id, false)])) // Assuming single output at index 0
                }
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

            _ => Err(Box::new(InvalidOperationError)), // Other types not implemented yet
        }
    }


    fn get_n_desired_inputs(&self) -> usize {
        match self {
            PrimitiveKind::HISIGNAL | PrimitiveKind::LOSIGNAL | PrimitiveKind::PULSE => 0,
            PrimitiveKind::LIGHT => 1,
            PrimitiveKind::BUFFER | PrimitiveKind::NOT => 1,
            PrimitiveKind::OR | PrimitiveKind::AND => 2,
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
