use game_sdk::{Action, ActionList, GameState};
use player::mcts::MCTS;
use std::fs::OpenOptions;
use std::io::prelude::*;

fn save(states: &[GameState], actions: &mut ActionList, result: i16, values: &mut [f32; 100]) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("datasets/dataset.txt")
        .unwrap();
    for (i, state) in states.iter().enumerate() {
        let fen = format!(
            "{} {} {} {}",
            state.to_fen(),
            actions[i].serialize(),
            values[i],
            result * state.current_color.team_i16()
        );
        if let Err(e) = writeln!(file, "{}", fen) {
            println!("Couldn't write to file: {}", e);
        }
    }
}

fn main() {
    let mut mcts = MCTS::new(3100);
    let mut actions: ActionList = ActionList::default();
    let mut values: [f32; 100] = [0.; 100];
    let mut states: Vec<GameState> = Vec::with_capacity(100);
    let mut states_searched: u64 = 0;
    let mut games_played: u64 = 0;
    let mut sum_results: i64 = 0;
    let mut sum_plies: u64 = 0;
    let mut one_wins: u64 = 0;
    let mut draws: u64 = 0;

    loop {
        let mut state = GameState::new();
        while !state.is_game_over() {
            println!("{}", state.to_fen());
            let (action, value) = mcts.search_action(&state);
            if action == Action::Skip {
                state.do_action(Action::Skip);
                continue;
            }
            states_searched += 1;
            values[state.ply as usize] = value;
            states.push(state.clone());
            actions.push(action);
            state.do_action(action);
        }
        sum_plies += state.ply as u64;
        games_played += 1;
        let result = state.game_result();
        match result {
            0 => draws += 1,
            r if r > 0 => one_wins += 1,
            _ => {}
        }
        sum_results += result as i64;
        save(&states, &mut actions, state.game_result(), &mut values);
        actions.clear();
        states.truncate(0);

        println!(
            "Games: {} Searched: {} Average game length: {} plies Average result: {} Sum results: {} One: {} Draws: {} Two: {}",
            games_played,
            states_searched,
            sum_plies as f64 / games_played as f64,
            sum_results as f64 / games_played as f64,
            sum_results,
            one_wins,
            draws,
            games_played - one_wins - draws
        );
    }
}
