use argparse::{ArgumentParser, Store};
use game_sdk::{Action, ActionList, GameState};
use std::io::{BufRead, BufReader, Write};
mod openings;
//use openings::random_opening;
//use rand::{rngs::SmallRng, SeedableRng};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};
use std::time::Instant;

pub struct Client {
    pub input: ChildStdin,
    pub output: ChildStdout,
    pub path: String,
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
}

impl Client {
    pub fn from_path(path: String, time: u64) -> Client {
        let mut process = Command::new(path.clone())
            .args(&["--time", &time.to_string()])
            .args(&["--testclient", "true"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|_| panic!("Can't start the client: {}", path));
        let input = process.stdin.take().unwrap();
        let output = process.stdout.take().unwrap();

        Client {
            input,
            output,
            path,
            wins: 0,
            draws: 0,
            losses: 0,
        }
    }
}

pub fn print_stats(client1: &Client, client2: &Client) {
    let mut line = String::new();
    let games_played = client1.wins + client1.draws + client1.losses;
    line.push_str(&format!(
        "{:6} {:27} {:6} {:6} {:6} {:27}",
        games_played, client1.path, client1.wins, client1.draws, client1.losses, client2.path
    ));
    println!("{}", line);
}

fn wait_for_action(client: &mut Client) -> Action {
    let mut stdin = BufReader::new(&mut client.output);
    let mut new_line = String::new();
    let start_time = Instant::now();
    loop {
        stdin.read_line(&mut new_line).expect("Can't read line");
        if !new_line.is_empty() && new_line.contains("action: ") {
            new_line = (&new_line[8..]).to_string();
            break;
        }
        new_line = String::new();
        if start_time.elapsed().as_secs() > 20 {
            panic!("Test client not responding");
        }
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

fn run_game(
    state: &mut GameState,
    client1: &mut Client,
    client2: &mut Client,
    automated_test: bool,
) -> i16 {
    let mut action_list = ActionList::default();
    while !state.is_game_over() {
        let team_one = state.ply % 2 == 0;
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
        if !automated_test {
            println!("{}", state);
        }
    }
    let result = state.game_result();
    match result {
        0 => {
            client1.draws += 1;
            client2.draws += 1;
        }
        r if r > 0 => {
            client1.wins += 1;
            client2.losses += 1;
        }
        _ => {
            client1.losses += 1;
            client2.wins += 1;
        }
    }
    result
}

fn main() {
    let mut client1_path = String::new();
    let mut client2_path = String::new();
    let mut games: u32 = 1_000_000;
    let mut automated_test = false;
    let mut time: u64 = 1900;

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
        parser
            .refer(&mut time)
            .add_option(&["-t", "--time"], Store, "Time in milliseconds");
        parser.parse_args_or_exit();
    }

    if !automated_test {
        println!("client1 path: {}", client1_path);
        println!("client2 path: {}", client2_path);
        println!("games: {}", games);
        println!("time / action: {}", time);
    }

    let mut client1 = Client::from_path(client1_path, time);
    let mut client2 = Client::from_path(client2_path, time);
    //let mut rng = SmallRng::from_entropy();
    let mut state = GameState::new(); //random_opening(&mut rng);
    std::thread::sleep(std::time::Duration::from_millis(1000));
    for i in 0..games {
        let result = if i % 2 == 0 {
            run_game(
                &mut state.clone(),
                &mut client1,
                &mut client2,
                automated_test,
            )
        } else {
            -run_game(
                &mut state.clone(),
                &mut client2,
                &mut client1,
                automated_test,
            )
        };
        if !automated_test {
            print_stats(&client1, &client2);
        } else {
            println!("{}", result);
        }
        if i % 2 == 1 {
            state = GameState::new(); //random_opening(&mut rng);
        }
    }
    if automated_test {
        println!("bye");
    }
}
