use game_sdk::{GameState, Player};
extern crate xml;
use super::xml_node::XMLNode;
use std::io::{prelude::Write, BufReader, BufWriter};
use std::net::TcpStream;
use std::time::Instant;
use xml::reader::*;

pub struct XMLClient {
    state: GameState,
    host: String,
    port: String,
    reservation: String,
    room_id: Option<String>,
    player: Box<dyn Player>,
    opponent_time: Instant,
}

impl XMLClient {
    pub fn new(
        host: String,
        port: String,
        reservation: String,
        player: Box<dyn Player>,
    ) -> XMLClient {
        XMLClient {
            state: GameState::default(),
            host,
            port,
            reservation,
            room_id: None,
            player,
            opponent_time: Instant::now(),
        }
    }

    pub fn run(&mut self) {
        print!("Connecting to {}:{}... ", self.host, self.port);
        let stream = TcpStream::connect(&format!("{}:{}", self.host, self.port))
            .expect("Could not connect to server");
        println!("Connected");
        XMLClient::write_to(&stream, "<protocol>");

        let join_xml = match self.reservation.as_str() {
            "" => "<join gameType=\"swc_2021_blokus\"/>".to_string(),
            _ => format!("<joinPrepared reservationCode=\"{}\" />", self.reservation),
        };
        print!("Sending join message ");
        XMLClient::write_to(&stream, join_xml.as_str());

        self.handle_stream(&stream);
    }

    fn handle_stream(&mut self, stream: &TcpStream) {
        let mut parser = EventReader::new(BufReader::new(stream));

        loop {
            let node = XMLNode::read_from(&mut parser);
            match node.name.as_str() {
                "data" => {
                    let invalid = &"".to_string();
                    let data_class = node.get_attribute("class").unwrap_or(invalid).to_string();
                    match data_class.as_str() {
                        "memento" => {
                            println!("Received memento: ");
                            node.as_memento(&mut self.state);
                            println!("    fen: {}", self.state.to_fen());
                            println!("    ply: {}", self.state.ply);
                        }
                        "welcomeMessage" => {
                            println!("Received welcome message");
                        }
                        "sc.framework.plugins.protocol.MoveRequest" => {
                            self.handle_move_request(stream);
                        }
                        "result" => {
                            self.handle_result(node);
                            return;
                        }
                        s => {
                            println!("{} {}", s.to_string(), node.data.to_string());
                        }
                    }
                }
                "joined" => {
                    self.room_id = Some(node.as_room());
                    println!("Joined {}", node.as_room());
                }
                "sc.protocol.responses.CloseConnection" => {
                    println!("Connection closed");
                    break;
                }
                _ => {}
            }
        }
    }

    fn handle_move_request(&mut self, stream: &TcpStream) {
        if self.state.ply > 1 {
            println!(
                "Received move request (Opponent responded after approximately {}ms)",
                self.opponent_time.elapsed().as_millis()
            );
        } else {
            println!("Received move request");
        }
        let action = self.player.on_move_request(&self.state);
        let xml_move = action.to_xml(self.state.get_current_color());
        println!("Sending: {}", action);
        XMLClient::write_to(
            stream,
            &format!(
                "<room roomId=\"{}\">\n{}\n</room>",
                self.room_id.as_ref().expect("Error while reading room id"),
                xml_move
            ),
        );
        self.opponent_time = Instant::now();
    }

    pub fn handle_result(&self, node: XMLNode) {
        println!("Received result");
        let score = node.get_child("score").expect("Unable to read score");
        println!(
            "{}",
            match score
                .get_attribute("cause")
                .expect("Error while reading cause")
                .as_str()
            {
                "REGULAR" => "The game ended regular.".to_string(),
                "LEFT" => "The game ended because a player left.".to_string(),
                "RULE_VIOLATION" => "The game ended because of a rule violation.".to_string(),
                "SOFT_TIMEOUT" =>
                    "The game ended because a player caused a soft timeout.".to_string(),
                _ => "The game ended because of a hard timeout.".to_string(),
            }
        );
        let mut team_one_score: i16 = 0;
        let mut team_two_score: i16 = 0;
        println!("Color   | Fields | Monomino last | All placed | Score");
        for color in 0..4 {
            let fields = self.state.board[color].count_ones() as i16;
            let all_placed = fields == 89;
            let m_last = self.state.monomino_placed_last & 0b1 << color != 0;
            let mut score = fields;
            if all_placed {
                score += 15;
                if m_last {
                    score += 5;
                }
            }
            println!(
                "{} | {:6} | {:13} | {:10} | {:5}",
                match color {
                    0 => "BLUE   ",
                    1 => "YELLOW ",
                    2 => "RED    ",
                    _ => "GREEN  ",
                },
                fields,
                m_last,
                all_placed,
                score
            );
            if color % 2 == 0 {
                team_one_score += score;
            } else {
                team_two_score += score;
            }
        }
        println!("One score: {}", team_one_score);
        println!("Two score: {}", team_two_score);
        let result = team_one_score - team_two_score;
        print!("Result: {} => ", self.state.game_result());
        match result {
            r if r > 0 => println!("Winner: Team One (BLUE, RED)"),
            r if r < 0 => println!("Winner: Team Two (YELLOW, GREEN)"),
            _ => println!("Draw"),
        }
    }

    fn write_to(stream: &TcpStream, data: &str) {
        let _ = BufWriter::new(stream).write(data.as_bytes());
    }
}
