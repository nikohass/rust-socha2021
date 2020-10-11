use argparse::{ArgumentParser, Store};
use game_sdk::{Action, ActionList, GameState};
use std::io::{BufRead, BufReader, Write};
mod client;
use client::{print_stats, Client};
mod openings;
use openings::random_opening;
use rand::{rngs::SmallRng, SeedableRng};

fn wait_for_action(client: &mut Client) -> Action {
    let mut bufreader = BufReader::new(&mut client.output);
    let mut new_line = String::new();
    loop {
        bufreader.read_line(&mut new_line).expect("Can't read line");

        if !new_line.is_empty() && new_line.contains("action: ") {
            new_line = (&new_line[8..]).to_string();
            break;
        }
        new_line = String::new();
    }
    new_line.pop(); // remove \n
    Action::deserialize(new_line)
}

fn request_action(state: &GameState, client: &mut Client) {
    let mut fen = state.to_fen();
    fen.push('\n');
    client
        .input
        .write_all(fen.as_bytes())
        .expect("Can't write to stdin");
}

fn run_game(state: &mut GameState, client1: &mut Client, client2: &mut Client) -> i16 {
    let mut action_list = ActionList::default();

    while !state.is_game_over() {
        let team_one = state.ply % 2 == 0;

        action_list.size = 0;
        state.get_possible_actions(&mut action_list);

        let action = if team_one {
            request_action(&state, client1);
            wait_for_action(client1)
        } else {
            request_action(&state, client2);
            wait_for_action(client2)
        };

        let mut valid = false;
        for i in 0..action_list.size {
            if action == action_list[i] {
                valid = true;
                break;
            }
        }
        if !valid {
            println!("Invalid action\n{} {}", state, action.to_string());
            for i in 0..action_list.size {
                println!("{}", action_list[i].to_string());
            }
            state.do_action(action);
            println!("{}", state);
            panic!("Invalid action");
        }
        state.do_action(action);
    }
    let result = state.game_result();
    match result {
        0 => {
            client1.draws_when_team1 += 1;
            client2.draws_when_team2 += 1;
        }
        r if r > 0 => {
            client1.wins_when_team1 += 1;
            client2.losses_when_team2 += 1;
        }
        _ => {
            client1.losses_when_team1 += 1;
            client2.wins_when_team2 += 1;
        }
    }
    result
}

fn main() {
    let mut client1_path = String::new();
    let mut client2_path = String::new();
    let mut games: u32 = 1_000_000;
    let mut automated_test = false;

    {
        let mut parser = ArgumentParser::new();
        parser
            .refer(&mut client1_path)
            .add_option(&["-o", "--one"], Store, "client1 path");
        parser
            .refer(&mut client2_path)
            .add_option(&["-t", "--two"], Store, "client2 path");
        parser
            .refer(&mut games)
            .add_option(&["-g", "--games"], Store, "number of games");
        parser.refer(&mut automated_test).add_option(
            &["-a", "--automated_test"],
            Store,
            "automated test",
        );
        parser.parse_args_or_exit();
    }
    if !automated_test {
        println!("client1 path: {}", client1_path);
        println!("client2 path: {}", client2_path);
        println!("games: {}", games);
    }

    let mut client1 = Client::from_path(client1_path);
    let mut client2 = Client::from_path(client2_path);

    let mut rng = SmallRng::from_entropy();
    let mut state = random_opening(&mut rng);
    for i in 0..games {
        let result = if i % 2 == 0 {
            run_game(&mut state.clone(), &mut client1, &mut client2)
        } else {
            -run_game(&mut state.clone(), &mut client2, &mut client1)
        };
        if !automated_test {
            print_stats(&client1, &client2);
        } else {
            println!("{}", result);
        }
        if i % 2 == 1 {
            state = random_opening(&mut rng);
        }
    }
    if automated_test {
        println!("bye");
    }
}
