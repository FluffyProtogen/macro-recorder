use macro_recorder::*;
fn main() {
    //let action_list = serde_json::from_str::<Vec<_>>(action_list).unwrap();

    let action_list = record_actions(true);

    while !play_key_pressed() {}

    //println!("{}", action_list.len());;;;;;
    play_back_actions(&action_list);

    //println!("{}", serde_json::to_string(&action_list).unwrap());
}
