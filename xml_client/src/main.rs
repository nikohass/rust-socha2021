use game_sdk::actionlist::ActionList;
use game_sdk::gamestate::GameState;

fn main() {
    let mut gs = GameState::new();
    gs.board[0].four = 1 << 25;
    println!("{}", gs);
    let mut al = ActionList::default();
    gs.get_possible_actions(&mut al);
    for i in 0..al.size {
        println!("{}", al[i].to_string());
    }
}
