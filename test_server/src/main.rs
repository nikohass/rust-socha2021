use argparse::{ArgumentParser, Store};
use game_sdk::action::Action;
use game_sdk::actionlist::ActionList;
use game_sdk::gamestate::GameState;
use std::io::{BufRead, BufReader, Write};
mod client;
use client::Client;

fn wait_for_action(client: &mut Client) -> Action {
    let mut bufreader = BufReader::new(&mut client.output);
    let mut new_line = String::new();
    loop {
        bufreader.read_line(&mut new_line).expect("Can't read line");

        if new_line.len() != 0 {
            if new_line.contains("info: ") {
                println!("{}", new_line);
            } else {
                break;
            }
        }
        new_line = String::new();
    }
    new_line.pop(); // remove \n
    Action::deserialize(new_line)
}

fn request_action(state: &GameState, client: &mut Client) {
    let mut fen = state.to_fen();
    fen.push_str("\n");
    client
        .input
        .write_all(fen.as_bytes())
        .expect("Can't write to stdin");
}

fn run_game(state: &mut GameState, client1: &mut Client, client2: &mut Client) {
    let mut action_list = ActionList::default();

    while !state.is_game_over() {
        let team_one = state.ply % 2 == 0 || state.ply % 2 == 1;

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
            panic!("Invalid action");
        }
        state.do_action(action);
    }
    let result = state.game_result();
    if result == 0 {
        client1.draws_when_team1 += 1;
        client2.draws_when_team2 += 1;
    } else if result > 0 {
        client1.wins_when_team1 += 1;
        client2.losses_when_team2 += 1;
    } else {
        client1.losses_when_team1 += 1;
        client2.wins_when_team2 += 1;
    }
}

fn main() {
    let mut client1_path = String::new();
    let mut client2_path = String::new();
    let mut games: u32 = 100;

    {
        let mut parser = ArgumentParser::new();
        parser
            .refer(&mut client1_path)
            .add_option(&["-o", "--one"], Store, "client path");
        parser
            .refer(&mut client2_path)
            .add_option(&["-t", "--two"], Store, "client path");
        parser
            .refer(&mut games)
            .add_option(&["-g", "--games"], Store, "number of games");
        parser.parse_args_or_exit();
    }

    println!("client1_path: {}", client1_path);
    println!("client2_path: {}", client2_path);
    println!("games: {}", games);

    print!("Game  ║Client              ║Wins  ║Draws ║Losses       ");
    println!("Game  ║Client              ║Wins  ║Draws ║Losses");
    print!("══════╬════════════════════╬══════╬══════╬═══════      ");
    println!("══════╬════════════════════╬══════╬══════╬═══════");

    let mut client1 = Client::from_path(client1_path);
    let mut client2 = Client::from_path(client2_path);

    for i in 0..games {
        let mut state = GameState::new();

        if i % 2 == 0 {
            run_game(&mut state, &mut client1, &mut client2);
        } else {
            run_game(&mut state, &mut client2, &mut client1);
        }

        client1.print_stats();
        print!("       ");
        client2.print_stats();
        print!("\n");
    }
}
