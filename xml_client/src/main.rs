use game_sdk::action::Action;
use game_sdk::actionlist::ActionList;
use game_sdk::gamestate::GameState;
//use player::search::search_action;
use rand::{rngs::SmallRng, RngCore, SeedableRng};

fn main() {
    let mut rng = SmallRng::from_entropy();
    let mut action_list = ActionList::default();

    let mut state = GameState::new();
    while !state.is_game_over() {
        println!("{}", state);

        action_list.size = 0;
        state.get_possible_actions(&mut action_list);
        for i in 0..action_list.size {
            debug_assert!(action_list[i] == Action::deserialize(action_list[i].serialize()));
        }
        let rand = rng.next_u64() as usize % action_list.size;
        state.do_action(action_list[rand]);
        debug_assert!(state == GameState::from_fen(state.to_fen()));
    }
    println!("{}", state);
    println!("result: {}", state.game_result());
}
