use egui::{Ui, Widget};

use super::{Input, LogicGateTemplate, Output};


#[derive(serde::Deserialize, serde::Serialize, Default, Hash, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Gate {
    pub label: String,
    pub n_in: usize,
    pub ins: Vec<super::Input>,

    pub n_out: usize,
    pub outs: Vec<Output>,
}

impl Gate {
    pub fn new(name: String) -> Gate {
        let n_ins= 0;
        let n_outs = 0;

        Gate {
            label: name,
            n_in: n_ins,
            ins: Self::create_inputs(n_ins),
            n_out: n_outs,
            outs: Self::create_outputs(n_outs),
        }
    }

    pub fn get_signal_in(&self) -> Vec<bool> {
        self.ins.iter().map(|i| i.signal).collect()
    }
    pub fn get_signal_out(&self) -> Vec<bool> {
        self.outs.iter().map(|o| o.signal).collect()
    }

    pub fn from_template(t: &mut LogicGateTemplate) -> Gate {
        let n_ins   = t.ins.len();
        let n_outs= t.outs.len();

        Gate {
            label: t.label.clone(),
            n_in: n_ins,
            ins: Self::create_inputs(n_ins),
            n_out: n_outs,
            outs: Self::create_outputs(n_outs),
        }
    }

    fn create_inputs(n_in: usize) -> Vec<Input>{
        let mut new_ins= Vec::<Input>::new();
        for n in 0.. n_in {
            new_ins.push(Input::new())
        }
        new_ins
    }

    fn create_outputs(n_out: usize) -> Vec<Output>{
        let mut new_outs = Vec::<Output>::new();
        for n in 0.. n_out {
            new_outs.push(Output::new())
        }
        new_outs
    }

    pub fn get_widget<'a>(&self, ui: impl FnOnce(&mut Ui)) -> egui::Button<'a> {
        egui::Button::selectable(
            false, // or set to true if you want it selected by default
            format!("{}: {} :{}", self.n_in, self.label, self.n_out),
        )
    }

    pub fn generate(label: String, n_ins: usize, n_outs: usize) -> Gate {
        Gate {
            label,
            n_in: n_ins,
            ins: Self::create_inputs(n_ins),
            n_out: n_outs,
            outs: Self::create_outputs(n_outs),
        }
    }

}
