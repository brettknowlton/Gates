use egui::{pos2, vec2, Pos2, Rect, Response, Sense, StrokeKind, Ui, UiBuilder, Vec2};

/// A pannable area that supports dragging the entire contents by holding space and clicking.
/// It also draws a dynamic grid of 9 cells that updates as the center moves.
pub struct PanArea<'a> {
    content: Box<dyn FnOnce(&mut Ui, Pos2) + 'a>,
    center: &'a mut Pos2,
}

impl<'a> PanArea<'a> {
    pub fn new<F>(center: &'a mut Pos2, content: F) -> Self
    where
        F: FnOnce(&mut Ui, Pos2) + 'a,
    {
        Self {
            content: Box::new(content),
            center,
        }
    }
}


impl<'a> egui::Widget for PanArea<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let available_size = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available_size, Sense::click_and_drag());

        let painter = ui.painter_at(rect);
        let mut pan_delta = Vec2::ZERO;

        ui.input(|i| {
            if i.key_down(egui::Key::Space) && i.pointer.primary_down() {
                if let Some(delta) = Some(i.pointer.delta()) {
                    pan_delta = delta;
                }
            }
        });

        *self.center -= pan_delta;

        // Draw grid lines
        let grid_size = 400.0;
        let center_cell_x = (self.center.x / grid_size).round();
        let center_cell_y = (self.center.y / grid_size).round();

        for dy in -2..=2 {
            for dx in -3..=2 {
                let cell_x = center_cell_x + dx as f32;
                let cell_y = center_cell_y + dy as f32;

                let top_left = pos2(
                    rect.center().x + (cell_x * grid_size - self.center.x),
                    rect.center().y + (cell_y * grid_size - self.center.y),
                );

                let grid_rect = Rect::from_min_size(top_left, vec2(grid_size, grid_size));

                painter.rect(
                    grid_rect,
                    0.0,
                    egui::Color32::from_gray(20),
                    egui::Stroke::new(1.0, egui::Color32::from_gray(60)),
                    StrokeKind::Outside,
                );
            }
        }

        let builder: UiBuilder = UiBuilder::new()
            .max_rect(rect)
            .sense(Sense::click_and_drag());

        let mut child_ui = ui.new_child(builder);
        let center_pos = *self.center; // copy out the value to avoid double borrow
        (self.content)(&mut child_ui, center_pos);

        response
    }
}
