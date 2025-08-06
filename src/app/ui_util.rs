use super::*;

#[derive(Debug, Clone)]
pub struct ClickItem {
    pub item_id: usize,
    pub screen_position: egui::Pos2,
    pub item_type: LogicalKind,
}
