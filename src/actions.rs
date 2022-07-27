use serde::*;
use winapi::shared::windef::*;
#[derive(Serialize, Deserialize)]
#[serde(remote = "POINT")]
struct POINTDef {
    x: i32,
    y: i32,
}

#[derive(Serialize, Deserialize)]
pub enum MouseActionKind {
    #[serde(with = "POINTDef")]
    Moved(POINT),
    Button(MouseActionButton),
}

#[derive(Serialize, Deserialize)]
pub struct MouseActionButton {
    #[serde(with = "POINTDef")]
    pub point: POINT,
    pub button: i32,
    pub pressed: bool,
}

#[derive(Serialize, Deserialize)]
pub enum Action {
    Delay(u64),
    Mouse(MouseActionKind),
    Keyboard(i32, bool),
}
