use game_sdk::actionlist::ActionList;
use game_sdk::gamestate::GameState;
use rand::{rngs::SmallRng, RngCore, SeedableRng};

fn main() {
    let mut rng = SmallRng::from_entropy();
    let mut gs = GameState::new();

    for _ in 0..16 {
        let mut al = ActionList::default();
        gs.get_possible_actions(&mut al);
        let rand = rng.next_u64() as usize % al.size;
        for i in 0..al.size {
            println!("{}", al[i].to_string());
        }
        println!("=> {}", al[rand].to_string());
        gs.do_action(al[rand]);

        //gs.pieces_left = [[true; 2]; 21];
        println!("{}\n", gs);

        //println!("Result: {}", gs.game_result());
    }
}
