// use super::*;

// pub struct LogicalButton {
//     label: String,
//     outs: Vec<Wire>,
//     pressed: bool,
// }


// impl LogicalButton {
//     pub fn new(label: String) -> Self {
//         Self {
//             label,
//             outs: Vec::new(),
//         }
//     }

//     pub fn tick(&mut self) {
//         if self.pressed{
//             for wire in &mut self.outs {
//                 wire.on();
//             }
//         }else {
//             for wire in &mut self.outs {
//                 wire.off();
//             }
//         }
//     }
// }