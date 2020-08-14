use game_sdk::actionlist::ActionList;
use game_sdk::gamestate::GameState;
use player::search::search_action;
use rand::{rngs::SmallRng, RngCore, SeedableRng};

fn main() {
    let mut rng = SmallRng::from_entropy();
    let mut action_list = ActionList::default();
    let mut state = GameState::new();

    while !state.is_game_over() {
        action_list.size = 0;

        if state.ply % 4 == 2 || state.ply % 4 == 3 {
            state.get_possible_actions(&mut action_list);
            let rand = rng.next_u64() as usize % action_list.size;
            state.do_action(action_list[rand]);
        } else {
            state.do_action(search_action(&state));
        }
        debug_assert!(state == GameState::from_fen(state.to_fen()));
        println!("{}", state);
    }
    println!("{}", state);
    println!("result: {}", state.game_result());
}
