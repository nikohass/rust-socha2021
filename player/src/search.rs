use super::evaluation::evaluate;
use game_sdk::action::Action;
use game_sdk::actionlist::ActionList;
use game_sdk::gamestate::GameState;

pub fn search_action(state: &GameState) -> Action {
    let mut state = state.clone();
    let mut action_list = ActionList::default();
    state.get_possible_actions(&mut action_list);
    let mut best_action = action_list[0];
    let mut best_value: i16 = -1_000;

    for i in 0..action_list.size {
        state.do_action(action_list[i]);
        let value = -evaluate(&state);
        state.undo_action(action_list[i]);
        if value > best_value {
            best_value = value;
            best_action = action_list[i];
        }
    }
    best_action
}
