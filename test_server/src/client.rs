use std::process::{ChildStdin, ChildStdout, Command, Stdio};

pub struct Client {
    pub input: ChildStdin,
    pub output: ChildStdout,
    pub path: String,
    pub wins_when_team1: u32,
    pub draws_when_team1: u32,
    pub losses_when_team1: u32,
    pub wins_when_team2: u32,
    pub draws_when_team2: u32,
    pub losses_when_team2: u32,
}

impl Client {
    pub fn from_path(path: String) -> Client {
        let mut process = Command::new(path.clone())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|_| panic!("Can't start the client: {}", path));
        let input = process.stdin.take().unwrap();
        let output = process.stdout.take().unwrap();

        Client {
            input: input,
            output: output,
            path: path,
            wins_when_team1: 0,
            draws_when_team1: 0,
            losses_when_team1: 0,
            wins_when_team2: 0,
            draws_when_team2: 0,
            losses_when_team2: 0,
        }
    }

    pub fn print_stats(&self) {
        let wins = self.wins_when_team1 + self.wins_when_team2;
        let draws = self.draws_when_team1 + self.draws_when_team2;
        let losses = self.losses_when_team1 + self.losses_when_team2;
        let games_played = wins + draws + losses;

        let mut line = String::new();
        line.push_str(&format!("{:6}", games_played));
        line.push_str(&format!("║{:20}║", self.path));
        line.push_str(&format!("{:6}║", wins));
        line.push_str(&format!("{:6}║", draws));
        line.push_str(&format!("{:6}", losses));
        print!("{}", line);
    }
}
