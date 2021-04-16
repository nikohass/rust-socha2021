use game_sdk::{ActionListStack, GameState};
use player::mcts::RaveTable;
use player::playout::playout;
use rand::{rngs::SmallRng, SeedableRng};
use std::time::Instant;

const TEST_FENS: [&str; 4] = [
    "9488 1813758321899637372028928 98304 31901482040045200628318736031602966529 162259508943118303423338611999184 10384593717069655257060992658440192 0 0 14680065 170141507979487117894522954291043368963 17179881472 996921076066887197892070253015345152 1952305837197645587728919239017365504 0 0 0 68719509504 9304611499219250726980198399157469184",
    "14096 6654190920398850590723072 98304 31901482040045200628318736031602966529 20282409835765575363979011887727056 93461620752214586704661989910642688 0 0 42535316147536582995760855127085285377 170141507984438882183735147901579427843 17179881472 996921076067189429491089201464125440 1952305854528819124263596185110970368 0 0 0 73014483968 9470764998692365211093174290282477568",
    "17168 6732109985381697757862914 884736 31901482040045200655988913714818449409 20282409835765575363979011887727056 93461620752214586704661989910642688 0 0 42535316147536582995760855127085285377 170141548549277432327859950371488137219 17179881472 996921076067190019787743985368344704 1952305854528819124263596185110970368 0 0 0 2535303278298107582477523524608 9470764998692365211093174290282477568",
    "18194 6732109985390493852982274 884736 31901482040045200655988913714818449409 20282409835765575363979011887727056 93461620752214586704661989910642688 0 131072 42535316147536582995760855127085285377 170141548549277432327859950371488137219 17179881472 996921076067190019787743985469008000 1952305854528819124263596185110970368 0 0 0 2535303278298107582477523524608 9470764998692365211093174290282477568",
];

fn perft(state: &mut GameState, depth: usize, als: &mut ActionListStack) -> u64 {
    if depth == 0 || state.is_game_over() {
        return 1;
    }
    state.get_possible_actions(&mut als[depth]);
    let mut nodes: u64 = 0;
    for i in 0..als[depth].size {
        state.do_action(als[depth][i]);
        nodes += perft(state, depth - 1, als);
        state.undo_action(als[depth][i]);
    }
    nodes
}

fn run_perft() {
    let current_best: f64 = 22_415.25;
    let depth = 3;
    let start_time = Instant::now();
    let mut als = ActionListStack::with_size(depth + 1);
    let mut nodes: u64 = 0;

    for fen in TEST_FENS.iter() {
        let mut state = GameState::from_fen((*fen).to_string());
        nodes += perft(&mut state, depth, &mut als);
    }

    let time_elapsed = start_time.elapsed().as_micros();
    let nps = (nodes * 1_000) as f64 / time_elapsed as f64;
    println!("Nodes/ms: {:.2} ({:.2}%)", nps, nps / current_best * 100.);
}

fn rollout_perft() {
    let mut rng = SmallRng::from_entropy();
    let mut rave_table = RaveTable::default();
    let start_time = Instant::now();
    let mut playouts: usize = 0;
    for fen in TEST_FENS.iter() {
        let state = GameState::from_fen((*fen).to_string());
        for _ in 0..100_000 {
            playout(&mut state.clone(), &mut rng, &mut rave_table);
        }
        playouts += 100_000;
    }
    let elapsed = start_time.elapsed().as_millis() as f64;
    let mut sum_actions: usize = 0;
    for values in rave_table.get_actions().iter() {
        let (_, n) = values;
        sum_actions += *n as usize;
    }
    println!(
        "{:.3} rollouts/ms, {:.3} actions/ms",
        playouts as f64 / elapsed,
        sum_actions as f64 / elapsed,
    );
}

fn main() {
    for _ in 0..10 {
        rollout_perft();
    }
    for _ in 0..10 {
        run_perft();
    }
}
