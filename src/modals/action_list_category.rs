use egui::{vec2, Align, Align2, Context, Key, Layout, Rect, Ui, Window};
use std::cell::RefCell;
use strum_macros::EnumIter;
use winapi::um::winuser::GetAsyncKeyState;

use crate::{actions::Action, gui::Recorder, modals::ModalWindow};

#[derive(Clone, Copy, Debug)]
pub enum ActionListCategory {
    Wait,
    If,
    Repeat,
}

#[derive(Clone, Copy, Debug, EnumIter)]
enum SubCategory {
    Delay,
    WaitForImage,
    WaitForPixel,
    IfImage,
    IfPixel,
    Else,
    EndIf,
    Repeat,
    EndRepeat,
    Break,
}

impl ActionListCategory {
    fn get_categories(&self) -> &[SubCategory] {
        use SubCategory::*;
        match *self {
            ActionListCategory::Wait => &[Delay, WaitForImage, WaitForPixel],
            ActionListCategory::If => &[IfImage, IfPixel, Else, EndIf],
            ActionListCategory::Repeat => &[Repeat, EndRepeat, Break],
        }
    }
}

impl SubCategory {
    fn get_default_action(&self) -> Action {
        use SubCategory::*;
        match *self {
            Delay => Action::Delay(0),
            WaitForImage => Action::WaitForImage(Default::default()),
            WaitForPixel => Action::WaitForPixel(Default::default()),
            IfImage => Action::IfImage(Default::default()),
            IfPixel => Action::IfPixel(Default::default()),
            Else => Action::Else,
            EndIf => Action::EndIf,
            Repeat => Action::Repeat(0),
            EndRepeat => Action::EndRepeat,
            Break => Action::Break,
        }
    }
}

impl ToString for SubCategory {
    fn to_string(&self) -> String {
        use SubCategory::*;
        match *self {
            Delay => "Delay".into(),
            WaitForImage => "Wait For Image".into(),
            WaitForPixel => "Wait For Pixel".into(),
            IfImage => "If Image Found".into(),
            IfPixel => "If Pixel Found".into(),
            Else => "Else".into(),
            EndIf => "End If".into(),
            Repeat => "Repeat".into(),
            EndRepeat => "End Repeat".into(),
            Break => "Break".into(),
        }
    }
}

pub struct ActionListWindow {
    data: RefCell<ActionListWindowData>,
}

impl ActionListWindow {
    pub fn new(category: ActionListCategory, index: i32) -> Self {
        Self {
            data: RefCell::new(ActionListWindowData {
                category,
                key_lock: Some(index),
            }),
        }
    }

    fn setup(&self, drag_bounds: Rect) -> Window {
        Window::new(format!("{:?} Selection", self.data.borrow().category))
            .collapsible(false)
            .resizable(false)
            .drag_bounds(drag_bounds)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
    }
}

struct ActionListWindowData {
    category: ActionListCategory,
    key_lock: Option<i32>,
}

impl ModalWindow for ActionListWindow {
    fn update(
        &self,
        recorder: &mut Recorder,
        ctx: &Context,
        ui: &mut Ui,
        drag_bounds: Rect,
        _frame: &mut eframe::Frame,
    ) {
        if ui.input().key_pressed(Key::Escape) {
            recorder.modal = None;
        }

        let window = self.setup(drag_bounds);

        window.show(ctx, |ui| {
            let mut data = self.data.borrow_mut();

            if let Some(key) = data.key_lock {
                if !get_pressed_numbers()
                    .iter()
                    .any(|key_pressed| *key_pressed == key)
                {
                    data.key_lock = None;
                }
            }

            for number in get_pressed_numbers() {
                if Some(number) != data.key_lock {
                    if let Some(category) = data.category.get_categories().get(number as usize - 1)
                    {
                        recorder.create_action_window(
                            category.get_default_action(),
                            vec2(drag_bounds.width(), drag_bounds.height()),
                            ctx,
                        );
                    }
                }
            }

            ui.allocate_space(vec2(0.0, 10.0));

            for category in data.category.get_categories() {
                if ui.button(category.to_string()).clicked() {
                    recorder.create_action_window(
                        category.get_default_action(),
                        vec2(drag_bounds.width(), drag_bounds.height()),
                        ctx,
                    );
                }

                ui.allocate_space(vec2(0.0, 10.0));
            }

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.add_space(25.0);

                if ui.button("Cancel").clicked() {
                    recorder.modal = None;
                }
            });
        });
    }
}

fn get_pressed_numbers() -> Vec<i32> {
    (0x31..=0x39)
        .filter_map(|code| {
            if unsafe { GetAsyncKeyState(code) < 0 } {
                Some(code - 0x30)
            } else {
                None
            }
        })
        .collect()
}
