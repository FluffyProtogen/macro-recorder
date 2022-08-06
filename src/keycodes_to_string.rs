use winapi::um::winuser::*;

pub fn key_code_to_string(code: i32) -> String {
    match code {
        VK_LBUTTON => "Left".into(),
        VK_RBUTTON => "Right".into(),
        VK_MBUTTON => "Middle".into(),
        _ => code.to_string(),
    }
}
