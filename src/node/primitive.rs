use super::*;

#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Primitive {
    pub label: String,
    pub kind: GateType,
    pub n_ins: usize,
    pub n_outs: usize,
}

impl Primitive {
    pub fn from_values(label: &str, num_inputs: usize, num_outputs: usize) -> Primitive {
        let kind: GateType;
        match label {
            "HI-SIGNAL" => {
                kind = GateType::Primitive(PrimitiveType::HISIGNAL); // Assuming HI-SIGNAL is a type of pulse
            }
            "LO-SIGNAL" => {
                kind = GateType::Primitive(PrimitiveType::LOSIGNAL); // Assuming LO-SIGNAL is a type of pulse
            }
            "PULSE" => {
                kind = GateType::Primitive(PrimitiveType::PULSE);
            }
            "LIGHT" => {
                kind = GateType::Primitive(PrimitiveType::LIGHT);
            }
            "BUFFER" => {
                kind = GateType::Primitive(PrimitiveType::BUFFER);
            }
            "NOT" => {
                kind = GateType::Primitive(PrimitiveType::NOT);
            }
            "OR" => {
                kind = GateType::Primitive(PrimitiveType::OR);
            }
            "AND" => {
                kind = GateType::Primitive(PrimitiveType::AND);
            }
            _ => {
                kind = GateType::Primitive(PrimitiveType::None);
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

impl Widget for Primitive {
    fn ui(self, _ui: &mut Ui) -> Response {
        todo!()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
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

impl PrimitiveType {
    pub fn tick(
        self,
        gate: &mut Gate,
        ins: HashMap<usize, bool>,
    ) -> Result<HashMap<usize, bool>, Box<dyn Error>> {
        println!("Ticking primitive type: {}", self);
        match self {
            PrimitiveType::HISIGNAL => {
                // HI-SIGNAL logic, for example, always outputs true
                let mut map = HashMap::new();
                for (id, _) in gate.outs.iter() {
                    map.extend(HashMap::from([(*id, true)]));
                } // Assuming single output
                Ok(map)
            }
            PrimitiveType::LOSIGNAL => {
                // LO-SIGNAL logic, for example, always outputs false
                let mut map = HashMap::new();
                for (id, _) in gate.outs.iter() {
                    map.extend(HashMap::from([(*id, false)]));
                } // Assuming single output
                Ok(map)
            }
            PrimitiveType::BUFFER => {
                // BUFFER logic, for example, passes the signal through
                //ensure only one input
                if ins.len() != 1 {
                    return Err("BUFFER requires exactly one input".into());
                }
                let mut map = HashMap::new();
                if let Some((out_id, _)) = gate.outs.iter().next() {//get the  id of the (only) output
                    if let Some((_, signal)) = ins.iter().next() {
                        map.extend(HashMap::from([(*out_id, *signal)])); // Assuming single output at index 0
                    } else {
                        return Err("Input signal not found".into());
                    }
                }
                Ok(map)
            }
            _ => Err(Box::new(InvalidOperationError)), // Other types not implemented yet
        }
    }
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveType::None => write!(f, "None"),
            PrimitiveType::HISIGNAL => write!(f, "HI-SIGNAL"),
            PrimitiveType::LOSIGNAL => write!(f, "LO-SIGNAL"),
            PrimitiveType::PULSE => write!(f, "PULSE"),
            PrimitiveType::LIGHT => write!(f, "LIGHT"),
            PrimitiveType::BUFFER => write!(f, "BUFFER"),
            PrimitiveType::NOT => write!(f, "NOT"),
            PrimitiveType::OR => write!(f, "OR"),
            PrimitiveType::AND => write!(f, "AND"),
        }
    }
}

impl Widget for PrimitiveType {
    fn ui(self, ui: &mut Ui) -> Response {
        let r = ui.add_enabled_ui(false, |ui| {
            ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                ui.label(self.to_string());
            });
        });
        r.response
    }
}
