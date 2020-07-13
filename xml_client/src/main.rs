use game_sdk::actionlist::ActionList;
use game_sdk::gamestate::GameState;
use rand::{rngs::SmallRng, RngCore, SeedableRng};

fn main() {
    let mut rng = SmallRng::from_entropy();

    let mut gs = GameState::new();
    let mut al = ActionList::default();

    for _ in 0..36 {
        al.size = 0;
        println!("{}", gs);
        gs.get_possible_actions(&mut al);
        let rand = rng.next_u64() as usize % al.size;
        gs.do_action(al[rand]);
        println!("-> {}", al[rand].to_string());
    }
    println!("{}", gs);
}
