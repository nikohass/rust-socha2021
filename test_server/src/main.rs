use argparse::{ArgumentParser, Store};
use game_sdk::{Action, ActionList, GameState};
use rand::{rngs::SmallRng, RngCore, SeedableRng};
use std::fmt::{Display, Formatter, Result};
use std::io::{BufRead, BufReader, Write};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};
use std::time::Instant;

pub struct TestResult {
    client_one_wins: u64,
    draws: u64,
    client_two_wins: u64,
    games_played: u64,
    sum_results: i64,
}

impl TestResult {
    pub fn add_game_result(&mut self, result: i16) {
        match result.cmp(&0) {
            std::cmp::Ordering::Greater => self.client_one_wins += 1,
            std::cmp::Ordering::Less => self.client_two_wins += 1,
            std::cmp::Ordering::Equal => self.draws += 1,
        };
        self.sum_results += result as i64;
        self.games_played += 1;
    }
}

impl Default for TestResult {
    fn default() -> Self {
        Self {
            client_one_wins: 0,
            draws: 0,
            client_two_wins: 0,
            games_played: 0,
            sum_results: 0,
        }
    }
}

impl Display for TestResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.client_one_wins,
            self.draws,
            self.client_two_wins,
            self.games_played,
            self.sum_results
        )
    }
}

pub struct Client {
    pub input: ChildStdin,
    pub output: ChildStdout,
    pub path: String,
    pub time: u64,
}

impl Client {
    pub fn from_path(path: String, time: u64) -> Self {
        let mut process = Command::new(path.clone())
            .args(&["--time", &time.to_string()])
            .args(&["--testclient", "true"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|_| panic!("Can't start the client: {}", path));
        Self {
            input: process.stdin.take().unwrap(),
            output: process.stdout.take().unwrap(),
            path,
            time,
        }
    }

    pub fn wait_for_action(&mut self, verbose: bool) -> Action {
        let mut stdin = BufReader::new(&mut self.output);
        let mut new_line = String::new();
        let start_time = Instant::now();
        if verbose {
            println!("Waiting for Action from {}", self.path);
        }
        loop {
            stdin.read_line(&mut new_line).expect("Can't read line");
            if !new_line.is_empty() && new_line.contains("action: ") {
                new_line = (&new_line[8..]).to_string();
                break;
            }
            if !new_line.is_empty() && verbose {
                new_line.pop();
                println!("{}", new_line);
            }
            new_line = String::new();
            if start_time.elapsed().as_millis() > self.time as u128 + 500 {
                panic!("Client not responding");
            }
        }
        new_line.pop();
        Action::deserialize(new_line)
    }

    pub fn request_action(&mut self, state: &GameState) {
        let mut fen = state.to_fen();
        fen.push('\n');
        self.input
            .write_all(fen.as_bytes())
            .expect("Can't write to stdin");
    }

    pub fn get_action(&mut self, state: &GameState, verbose: bool) -> Action {
        self.request_action(state);
        self.wait_for_action(verbose)
    }
}

fn play_game(
    state: &mut GameState,
    client_team_one: &mut Client,
    client_team_two: &mut Client,
    verbose: bool,
    r: f64,
) -> i16 {
    let mut rng = SmallRng::from_entropy();
    let mut action_list = ActionList::default();
    while !state.is_game_over() {
        state.get_possible_actions(&mut action_list);

        if action_list[0] == Action::Skip {
            state.do_action(Action::Skip);
            continue;
        }

        // do a random action sometimes to avoid playing the same game over and over again
        if rng.next_u64() < ((std::u64::MAX as f64) * r) as u64 {
            state.do_action(action_list[rng.next_u64() as usize % action_list.size]);
            continue;
        }

        // request an action from the current client
        let action = if state.ply & 0b1 == 0 {
            client_team_one.get_action(&state, verbose)
        } else {
            client_team_two.get_action(&state, verbose)
        };
        assert!(state.validate_action(&action));
        state.do_action(action);
        if verbose {
            println!("{}\n{}", state, state.to_fen());
        }
    }
    state.game_result()
}

fn main() {
    let mut client_one_path = String::new();
    let mut client_two_path = String::new();
    let mut games: u32 = 100;
    let mut verbose = false;
    let mut time: u64 = 1980;
    let mut r: f64 = 0.05;

    {
        let mut parser = ArgumentParser::new();
        parser
            .refer(&mut client_one_path)
            .add_option(&["-1", "--one"], Store, "Client 1 path");
        parser
            .refer(&mut client_two_path)
            .add_option(&["-2", "--two"], Store, "Client 2 path");
        parser
            .refer(&mut games)
            .add_option(&["-g", "--games"], Store, "Number of games");
        parser.refer(&mut verbose).add_option(
            &["-v", "--verbose"],
            Store,
            "Print the stdout of the Clients and the current GameState",
        );
        parser
            .refer(&mut time)
            .add_option(&["-t", "--time"], Store, "Time/Action in milliseconds");
        parser.refer(&mut r).add_option(
            &["-r", "--random"],
            Store,
            "Probability of a random action",
        );
        parser.parse_args_or_exit();
    }

    if verbose {
        println!("client_one_path: {}", client_one_path);
        println!("client_two_path: {}", client_two_path);
        println!("games: {}", games);
        println!("time: {}", time);
    }

    let mut client_one = Client::from_path(client_one_path, time);
    let mut client_two = Client::from_path(client_two_path, time);
    let mut test_result = TestResult::default();
    let state = GameState::new();
    std::thread::sleep(std::time::Duration::from_millis(1000)); // give the clients some time to initialize

    for _ in 0..games / 2 {
        test_result.add_game_result(play_game(
            &mut state.clone(),
            &mut client_one,
            &mut client_two,
            verbose,
            r,
        ));
        println!("{}", test_result);
        test_result.add_game_result(-play_game(
            &mut state.clone(),
            &mut client_two,
            &mut client_one,
            verbose,
            r,
        ));
        println!("{}", test_result);
    }
}
