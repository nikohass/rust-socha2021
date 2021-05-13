use argparse::{ArgumentParser, Store};
use game_sdk::{Action, ActionList, GameState, Player};
use std::io::{BufRead, BufReader, Write};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};
use std::time::Instant;

pub struct Client {
    pub path: String,
    pub stdin: ChildStdin,
    pub stdout: ChildStdout,
    pub time: u64,
}

impl Client {
    pub fn from_path(path: String, time: u64) -> Self {
        let mut process = Command::new(path.clone())
            .args(&["--time", &time.to_string()])
            .args(&["--test", "true"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|_| panic!("Can't start the client: {}", path));
        Self {
            path,
            stdin: process.stdin.take().unwrap(),
            stdout: process.stdout.take().unwrap(),
            time,
        }
    }
}

impl Player for Client {
    fn on_move_request(&mut self, state: &GameState) -> Action {
        let mut fen = state.to_fen();
        fen.push('\n');
        self.stdin.write_all(fen.as_bytes()).unwrap();
        let start_time = Instant::now();
        let mut read = BufReader::new(&mut self.stdout);
        let mut line = String::new();
        loop {
            read.read_line(&mut line).unwrap();
            if !line.is_empty() && line.contains("action: ") {
                line = (&line[8..]).to_string();
                break;
            }
            if !line.is_empty() {
                line.pop();
                println!("info: {}", line);
            }
            line.truncate(0);
            let elapsed = start_time.elapsed().as_millis();
            if elapsed > self.time as u128 + 2500 {
                println!("warning: Client {} hard-timeout: {}ms", self.path, elapsed);
            }
        }
        let elapsed = start_time.elapsed().as_millis();
        if elapsed as u64 > 1990 {
            println!("warning: Client {} soft-timeout: {}ms", self.path, elapsed);
        }
        line.pop();
        Action::deserialize(line)
    }

    fn on_reset(&mut self) {}
}

pub fn play_game(client_one: &mut Client, client_two: &mut Client, first: u8) {
    let mut state = GameState::random();
    let mut al = ActionList::default();
    while !state.is_game_over() {
        state.get_possible_actions(&mut al);
        if al[0].is_skip() {
            state.do_action(al[0]);
            continue;
        }
        let action = if state.ply % 2 == first {
            client_one.on_move_request(&state)
        } else {
            client_two.on_move_request(&state)
        };
        state.do_action(action);
    }
    let result = state.game_result() as i64;
    let mut scores: [u32; 4] = [0, 0, 0, 0];
    for (color, score) in scores.iter_mut().enumerate() {
        *score = state.board[color].count_ones();
        if *score == 89 {
            *score += 15;
            if state.monomino_placed_last & 0b1 << color != 0 {
                *score += 5;
            }
        }
    }
    println!(
        "result: {} {} {} {}",
        first,
        result,
        scores[0] + scores[2],
        scores[1] + scores[3],
    );
}

fn main() {
    let mut client_one_path = String::new();
    let mut client_two_path = String::new();
    let mut games: u64 = 1000;
    let mut time: u64 = 1600;

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
        parser
            .refer(&mut time)
            .add_option(&["-t", "--time"], Store, "Time/Action in milliseconds");
        parser.parse_args_or_exit();
    }

    println!("info: client_one_path: {}", client_one_path);
    println!("info: client_two_path: {}", client_two_path);
    println!("info: games: {}", games);
    println!("info: time: {}", time);

    let mut client_one = Client::from_path(client_one_path, time);
    let mut client_two = Client::from_path(client_two_path, time);
    std::thread::sleep(std::time::Duration::from_millis(1000));
    loop {
        play_game(&mut client_one, &mut client_two, 0);
        play_game(&mut client_one, &mut client_two, 1);
    }
}
