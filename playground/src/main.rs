use game_sdk::*;
use player::heuristics::*;
use player::mcts::Mcts;
//use player::simple_client::SimpleClient;
//use std::time::Instant;

fn main() {
    let mut games: f64 = 0.;
    let mut sum_results: f64 = 0.;
    let mut color: u8 = 0;
    //let mut sc = SimpleClient::default();
    let mut player = HeuristicPlayer::default();
    let mut opponent = Mcts::default();
    opponent.set_iteration_limit(Some(1000));
    opponent.set_time_limit(None);

    //2.92

    /*
    let mut state = GameState::from_fen("7392 765025752630225718232064 17179883520 1495381574486230650230997900620466176 1163074729044538219175288598071083008 0 0 16 2596149667208375436981149938745345 162259354200502712224788707803143 32768 37218406699698119960604429879356686351 649037107316853453566312041152512 0 0 0 132226297549164264833024 2658457259221036439540898241362526208".to_string());
    while !state.is_game_over() {
        println!("{}", state);
        {
            let mut f = String::new();
            std::io::stdin().read_line(&mut f).expect("Can't read line");
        }
        state.do_action(player.on_move_request(&state));
        println!("{}", state);
        {
            let mut f = String::new();
            std::io::stdin().read_line(&mut f).expect("Can't read line");
        }
        state.do_action(opponent.on_move_request(&state));
    }*/

    for _ in 0..100 {
        // 63.87564293062064
        //let mut state = GameState::from_fen("7392 765025752630225718232064 17179883520 1495381574486230650230997900620466176 1163074729044538219175288598071083008 0 0 16 2596149667208375436981149938745345 162259354200502712224788707803143 32768 37218406699698119960604429879356686351 649037107316853453566312041152512 0 0 0 132226297549164264833024 2658457259221036439540898241362526208".to_string());

        let mut state = GameState::random();
        while !state.is_game_over() {
            /*{
                let mut f = String::new();
                std::io::stdin().read_line(&mut f).expect("Can't read line");
                println!("{}", state);
            }*/
            if state.ply % 2 == color {
                let best_action = player.on_move_request(&state);
                state.do_action(best_action);
            } else {
                state.do_action(opponent.on_move_request(&state));
            }
        }
        println!("{}", state);
        sum_results += match color {
            0 => state.game_result() as f64,
            _ => -state.game_result() as f64,
        };
        color ^= 1;
        games += 1.;
        //println!("{}", state);
        println!("{}", sum_results / games);
    }
}

//use packed_simd::*;

//fn main() {
/*
let a = f32x4::new(1., 2., 3., 4.);
let b = f32x4::new(5., 6., 7., 8.);
println!("{:?}", a + b);

let a = f32x8::new(1., 2., 3., 4., 1., 2., 3., 4.);
let b = f32x8::new(5., 6., 7., 8., 1., 2., 3., 4.);
println!("{:?}", a + b);

let a = f32x16::new(
    1., 2., 3., 4., 1., 2., 3., 4., 1., 2., 3., 4., 1., 2., 3., 4.,
);
let b = f32x16::new(
    5., 6., 7., 8., 1., 2., 3., 4., 1., 2., 3., 4., 1., 2., 3., 4.,
);
println!("{:?}", a + b);
*/
//}

/*#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use core::{arch::x86_64::*, mem::size_of};
*/
//fn main() {
/*unsafe {
    let b: __m256i = _mm256_set_epi32(1, 2, 3, 4, 5, 6, 7, 8);
    let res = _mm256_add_epi32(b, b);
    println!("{:?}", res);
}

unsafe {
    //let zeros = _mm256_setzero_ps();
    //let ones = _mm256_set1_ps(1.0);
    let mut floats = _mm256_set_ps(1.0, 2.0, 3.0, 4.0, 1., 1., 1., 1.);
    floats = _mm256_add_ps(floats, floats);
    floats = _mm256_mul_ps(floats, floats);
    floats = _mm256_div_ps(floats, floats);
    println!("{:?}", floats);
}



let simd = unsafe {
    let s = _mm256_set_ps(5., 5., 5., 5., 5., 5., 5., 5.);

    let o = _mm256_set_ps(3., 3., 3., 3., 3., 3., 3., 3.);

    let res = _mm256_mul_ps(s, o);
    let mut dst = [0.; 8];
    _mm256_store_ps(dst.as_mut_ptr(), res);
    dst
};
println!("{:?}", simd);*/

//test_vector = simd.chain(remainder).collect();
//println!("{:?}", test_vector);
//}
