use game_sdk::{ActionList, ActionListStack, GameState};
use rand::{rngs::SmallRng, RngCore, SeedableRng};
use std::time::Instant;

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

fn random_perft() {
    let start_time = Instant::now();
    let mut rng = SmallRng::from_entropy();

    for _ in 0..10_000 {
        let mut state = GameState::random();
        while !state.is_game_over() {
            let random_action = state.get_random_possible_action(&mut rng, state.ply < 16, 40);
            state.do_action(random_action);
        }
    }
    let time_elapsed = start_time.elapsed().as_micros();
    println!(
        "Random action generation: {}ms",
        time_elapsed as f64 / 1000.
    );

    let start_time = Instant::now();
    let mut al = ActionList::default();
    for _ in 0..10_000 {
        let mut state = GameState::random();
        while !state.is_game_over() {
            state.get_possible_actions(&mut al);
            state.do_action(al[rng.next_u64() as usize % al.size]);
        }
    }
    let time_elapsed = start_time.elapsed().as_micros();
    println!(
        "Complete action genteration: {}ms",
        time_elapsed as f64 / 1000.
    );
}

fn test() {
    let current_best: f64 = 22_061_650.;
    let depth = 3;
    let start_time = Instant::now();
    let mut als = ActionListStack::with_size(depth + 1);
    let mut nodes: u64 = 0;
    let fens = vec![
        "9488 1813758321899637372028928 98304 31901482040045200628318736031602966529 162259508943118303423338611999184 10384593717069655257060992658440192 0 0 14680065 170141507979487117894522954291043368963 17179881472 996921076066887197892070253015345152 1952305837197645587728919239017365504 0 0 0 68719509504 9304611499219250726980198399157469184",
        "14096 6654190920398850590723072 98304 31901482040045200628318736031602966529 20282409835765575363979011887727056 93461620752214586704661989910642688 0 0 42535316147536582995760855127085285377 170141507984438882183735147901579427843 17179881472 996921076067189429491089201464125440 1952305854528819124263596185110970368 0 0 0 73014483968 9470764998692365211093174290282477568",
        "17168 6732109985381697757862914 884736 31901482040045200655988913714818449409 20282409835765575363979011887727056 93461620752214586704661989910642688 0 0 42535316147536582995760855127085285377 170141548549277432327859950371488137219 17179881472 996921076067190019787743985368344704 1952305854528819124263596185110970368 0 0 0 2535303278298107582477523524608 9470764998692365211093174290282477568",
        "18194 6732109985390493852982274 884736 31901482040045200655988913714818449409 20282409835765575363979011887727056 93461620752214586704661989910642688 0 131072 42535316147536582995760855127085285377 170141548549277432327859950371488137219 17179881472 996921076067190019787743985469008000 1952305854528819124263596185110970368 0 0 0 2535303278298107582477523524608 9470764998692365211093174290282477568",
    ];

    for fen in fens.iter() {
        let mut state = GameState::from_fen((*fen).to_string());
        nodes += perft(&mut state, depth, &mut als);
    }

    let time_elapsed = start_time.elapsed().as_micros();
    let nps = (nodes * 1_000_000) as f64 / time_elapsed as f64;
    println!(
        "{:.2}%, Nodes: {}, Nodes/s: {}",
        nps / current_best * 100.,
        nodes,
        nps as u64
    );
}

fn main() {
    for _ in 0..3 {
        test();
    }
    random_perft();
}
