use std::rc::Rc;

use eframe::egui::*;

use crate::{actions::*, gui::Recorder};

pub struct ActionRightClickDialog {
    pub position: Pos2,
}

impl Action {
    pub fn get_right_click_dialog(&self, position: Pos2) -> ActionRightClickDialog {
        ActionRightClickDialog { position }
    }
}

impl ActionRightClickDialog {
    pub fn update(
        &self,
        recorder: &mut Recorder,
        ctx: &Context,
        ui: &mut Ui,
        screen_dimensions: Vec2,
    ) {
        let window = Window::new("Actions")
            .fixed_pos(self.position)
            .collapsible(false)
            .resizable(false);

        window.show(ctx, |ui| {
            ui.allocate_space(vec2(0.0, 5.0));
            let button = Button::new("Edit").fill(Color32::from_rgba_premultiplied(0, 0, 0, 0));
            let edit_response = button.ui(ui);
            ui.allocate_space(vec2(0.0, 5.0));
            let button = Button::new("Delete").fill(Color32::from_rgba_premultiplied(0, 0, 0, 0));
            let delete_response = button.ui(ui);
            ui.allocate_space(vec2(0.0, 5.0));
            let button =
                Button::new("Select All").fill(Color32::from_rgba_premultiplied(0, 0, 0, 0));
            let select_all_response = button.ui(ui);

            if edit_response.clicked() {
                recorder.modify_command_window =
                    Some(Rc::new(
                        recorder.action_list[recorder.selected_row.unwrap()]
                            .get_modify_command_window(false, self.position, ctx),
                    ));
                recorder.right_click_dialog = None;
            }

            if delete_response.clicked() {
                recorder.action_list.remove(recorder.selected_row.unwrap());
                recorder.right_click_dialog = None;
                recorder.selected_row = None;
            }
        });
    }
}
