use macro_recorder::*;
fn main() {
    //sample();;

    //let action_list = r#"[{"Delay":615877},{"Keyboard":[17,true]},{"Delay":63868},{"Keyboard":[86,true]},{"Delay":120455},{"Keyboard":[17,false]},{"Delay":274},{"Delay":15306},{"Keyboard":[86,false]}]"#;

    //let action_list = serde_json::from_str::<Vec<_>>(action_list).unwrap();

    let action_list = record_actions(false);

    while !play_key_pressed() {}

    //println!("{}", action_list.len());;;;;;p
    play_back_actions(&action_list);

    //println!("{}", serde_json::to_string(&action_list).unwrap());ggvvv
}
