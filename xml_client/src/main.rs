use game_sdk::actionlist::ActionList;
use game_sdk::gamestate::GameState;
use rand::{rngs::SmallRng, RngCore, SeedableRng};

fn main() {
    let mut rng = SmallRng::from_entropy();
    let mut action_list = ActionList::default();
    let mut state = GameState::new();

    while !state.is_game_over() {
        println!("{}", state.skipped);
        action_list.size = 0;
        state.get_possible_actions(&mut action_list);
        let rand = rng.next_u64() as usize % action_list.size;
        println!("{}", action_list[rand].to_string());
        state.do_action(action_list[rand]);
    }
    println!("{}", state);
    println!("result: {}", state.game_result());
    println!("{}", state == GameState::from_fen(state.to_fen()));
}
