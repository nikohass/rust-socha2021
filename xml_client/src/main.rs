use game_sdk::actionlist::ActionList;
use game_sdk::gamestate::GameState;

fn main() {
    let mut gs = GameState::new();
    println!("{}", gs);
    for _ in 0..8 {
        let mut al = ActionList::default();
        gs.get_possible_actions(&mut al);
        for i in 0..al.size {
            println!("{}", al[i].to_string());
        }
        gs.do_action(al[0]);
        println!("{}", gs);
    }
}
