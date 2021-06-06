/*use game_sdk::{Action, ActionList, GameState};
use player::mcts::heuristics::*;
use player::mcts::node::Node;
use std::thread;

const THREADS: usize = 10;

fn get_action_with_parameters(
    state: &GameState,
    al: &mut ActionList,
    params: &[f32; N_PARAMS],
) -> Action {
    let mut node = Node::empty();
    state.get_possible_actions(al);
    if al[0].is_skip() {
        return al[0];
    }
    node.children = Vec::with_capacity(al.size);
    expand_node(&mut node, state, al, params);
    let mut best_action = al[0];
    let mut best_value = std::f32::NEG_INFINITY;
    for child_node in node.children.iter() {
        let heuristic_value = child_node.get_value();
        if heuristic_value > best_value {
            best_value = heuristic_value;
            best_action = child_node.action;
        }
    }
    best_action
}

fn run_evaluation_thread(
    one: &[f32; N_PARAMS],
    two: &[f32; N_PARAMS],
    games: usize,
) -> (usize, usize) {
    let mut al = ActionList::default();
    let mut one_wins: usize = 0;
    let mut two_wins: usize = 0;
    for game_index in 0..games {
        let mut state = GameState::random();
        while !state.is_game_over() {
            let action = get_action_with_parameters(
                &state,
                &mut al,
                if game_index % 2 == state.ply as usize % 2 {
                    one
                } else {
                    two
                },
            );
            state.do_action(action);
        }
        state.ply = game_index as u8;
        match state.game_result() * state.get_team() {
            r if r > 0 => one_wins += 1,
            r if r < 0 => two_wins += 1,
            _ => {}
        };
    }
    (one_wins, two_wins)
}

fn evaluate(one: &[f32; N_PARAMS], two: &[f32; N_PARAMS], games: usize) -> f32 {
    let mut children = vec![];
    for _ in 0..THREADS {
        let one = *one;
        let two = *two;
        children.push(thread::spawn(move || {
            run_evaluation_thread(&one, &two, games / THREADS)
        }));
    }
    let mut one_wins: usize = 0;
    let mut two_wins: usize = 0;
    for child in children {
        let result = child.join().unwrap();
        one_wins += result.0;
        two_wins += result.1;
    }
    one_wins as f32 / two_wins as f32
}

fn tune(params: [f32; N_PARAMS], mut delta: f32, games: usize) {
    let mut best_params = params;
    loop {
        let mut improved = false;
        for i in 0..N_PARAMS {
            let mut test_params = best_params;
            test_params[i] += delta;
            let value = evaluate(&test_params, &best_params, games);
            if value > 1.01 {
                improved = true;
                best_params = test_params;
            } else {
                test_params[i] -= delta * 2.0;
                let value = evaluate(&test_params, &best_params, games);
                if value > 1.01 {
                    best_params = test_params;
                    improved = true;
                }
            }
        }
        println!("{} {:?}", delta, best_params);
        if !improved {
            delta /= 2.0;
            if delta < 0.0001 {
                println!("{} {:?}", delta, best_params);
                break;
            }
        }
    }
}*/

fn main() {
    /*let initial_guess = [
        0.35, 1.375, 2.375, 4.15, 2.15, 2.225, 1.325, 1.425, 1.425, 2.025, 2.20,
    ]; //[1.; N_PARAMS];
    tune(initial_guess, 0.1, 10_000);
    */
}
