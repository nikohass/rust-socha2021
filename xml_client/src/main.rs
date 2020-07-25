#[allow(unused_imports)]
use game_sdk::actionlist::{ActionList, ActionListStack};
use game_sdk::gamestate::GameState;
#[allow(unused_imports)]
use rand::{rngs::SmallRng, RngCore, SeedableRng};
use std::time::Instant;

fn perft(state: &mut GameState, depth: usize, als: &mut ActionListStack) -> u64 {
    if depth == 0 {
        return 1;
    }
    als[depth].size = 0;
    state.get_possible_actions(&mut als[depth]);
    let mut nodes: u64 = 0;
    for i in 0..als[depth].size {
        state.do_action(als[depth][i]);
        nodes += perft(state, depth - 1, als);
        state.undo_action(als[depth][i]);
        if i > 10 {
            break;
        }
    }
    nodes
}

fn test() {
    let current_best: f64 = 3491.7866096726702;
    let tests = 100;

    let mut state = GameState::new();
    let depth: usize = 6;
    let mut nodes: u64 = 0;
    let start_time = Instant::now();
    let mut als = ActionListStack::with_size(depth + 1);
    for i in 0..tests {
        println!("{}/{}", i, tests);
        nodes += perft(&mut state, depth, &mut als);
    }
    let time_elapsed = start_time.elapsed().as_micros();
    let nps = (1000 * nodes) as f64 / time_elapsed as f64;

    println!("Nodes: {}\nTime: {}ms\nnps: {}", nodes, time_elapsed as f64 / 1000., nps);
    println!("{}%", nps / current_best * 100.);
}

fn main() {
    test();
    /*
    let mut rng = SmallRng::from_entropy();
    let mut al = ActionList::default();
    let mut state = GameState::new();
    println!("{}", state);
    for _ in 0..120 {
        al.size = 0;
        state.get_possible_actions(&mut al);
        let rand = rng.next_u64() as usize % al.size;
        state.do_action(al[rand]);
        println!("{}", state);
    }
    let res = state.game_result();
    println!("{}", res);*/
}
