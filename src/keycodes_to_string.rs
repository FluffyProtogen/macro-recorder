use std::borrow::Cow;

use winapi::um::winuser::*;

lazy_static::lazy_static! {
    pub static ref ALLOWED_KEYBOARD_KEYS: Vec<i32> = {
        let mut vec = vec![
            VK_BACK,
            VK_TAB,
            VK_CLEAR,
            VK_RETURN,
            VK_LSHIFT,
            VK_RSHIFT,
            VK_LCONTROL,
            VK_RCONTROL,
            VK_LMENU,
            VK_RMENU,
            VK_CAPITAL,
            VK_ESCAPE,
            VK_SPACE,
            VK_PRIOR,
            VK_NEXT,
            VK_END,
            VK_HOME,
            VK_LEFT,
            VK_UP,
            VK_RIGHT,
            VK_DOWN,
            VK_SELECT,
            VK_PRINT,
            VK_EXECUTE,
            VK_SNAPSHOT,
            VK_INSERT,
            VK_DELETE,
            VK_HELP,
            ];

        vec.append(&mut (0x30..=0x39).collect());
        vec.append(&mut (0x41..=0x5A).collect());
        vec.push(VK_LWIN);
        vec.push(VK_RWIN);
        vec.push(VK_APPS);
        vec.push(VK_SLEEP);
        vec.append(&mut (VK_NUMPAD0..=VK_NUMPAD9).collect());
        vec.push(VK_ADD);
        vec.push(VK_SUBTRACT);
        vec.push(VK_MULTIPLY);
        vec.push(VK_DIVIDE);
        vec.push(VK_SEPARATOR);
        vec.push(VK_DECIMAL);
        vec.append(&mut (VK_F1..=VK_F24).collect());
        vec.push(VK_NUMLOCK);
        vec.push(VK_SCROLL);
        vec
    };
}

pub fn key_code_to_string(code: i32) -> Cow<'static, str> {
    match code {
        VK_LBUTTON => "Left".into(),
        VK_RBUTTON => "Right".into(),
        VK_MBUTTON => "Middle".into(),
        VK_BACK => "Back".into(),
        VK_TAB => "Tab".into(),
        VK_CLEAR => "Clear".into(),
        VK_RETURN => "Enter".into(),
        VK_LSHIFT => "Left Shift".into(),
        VK_RSHIFT => "Right Shift".into(),
        VK_LCONTROL => "Left Control".into(),
        VK_RCONTROL => "Right Control".into(),
        VK_LMENU => "Left Alt".into(),
        VK_RMENU => "Right Alt".into(),
        VK_CAPITAL => "Caps Lock".into(),
        VK_ESCAPE => "Escape".into(),
        VK_SPACE => "Space".into(),
        VK_PRIOR => "Page Up".into(),
        VK_NEXT => "Page Down".into(),
        VK_END => "End".into(),
        VK_HOME => "Home".into(),
        VK_LEFT => "Left Arrow".into(),
        VK_UP => "Up Arrow".into(),
        VK_RIGHT => "Right Arrow".into(),
        VK_DOWN => "Down Arrow".into(),
        VK_SELECT => "Select".into(),
        VK_PRINT => "Print".into(),
        VK_EXECUTE => "Execute".into(),
        VK_SNAPSHOT => "Snapshot".into(),
        VK_INSERT => "Insert".into(),
        VK_DELETE => "Delete".into(),
        VK_HELP => "Help".into(),
        0x30..=0x39 => (code - 0x30).to_string().into(),
        0x41..=0x5A => (code as u8 as char).to_string().into(),
        VK_LWIN => "Left Windows".into(),
        VK_RWIN => "Right Windows".into(),
        VK_APPS => "Applications".into(),
        VK_SLEEP => "Sleep".into(),
        VK_NUMPAD0..=VK_NUMPAD9 => format!("Number Pad {}", code - VK_NUMPAD0).into(),
        VK_ADD => "Add".into(),
        VK_SUBTRACT => "Subtract".into(),
        VK_MULTIPLY => "Multiply".into(),
        VK_DIVIDE => "Divide".into(),
        VK_SEPARATOR => "Separator".into(),
        VK_DECIMAL => "Decimal".into(),
        VK_F1..=VK_F24 => format!("F{}", code - VK_F1 + 1).into(),
        VK_NUMLOCK => "Number Lock".into(),
        VK_SCROLL => "Scroll".into(),
        _ => format!("Key Code: {}", code).into(),
    }
}
