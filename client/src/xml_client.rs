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
                            println!("Recieved memento: ");
                            node.as_memento(&mut self.state);
                            println!("    fen: {}", self.state.to_fen());
                            println!("    ply: {}", self.state.ply);
                        }
                        "welcomeMessage" => {
                            println!("Recieved welcome message");
                        }
                        "sc.framework.plugins.protocol.MoveRequest" => {
                            if self.state.ply > 1 {
                                println!(
                                    "Recieved move request (Opponent took ca. {}ms to respond)",
                                    self.opponent_time.elapsed().as_millis()
                                );
                            } else {
                                println!("Recieved move request");
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
                        "result" => {
                            println!("Recieved result");
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
                                    "RULE_VIOLATION" =>
                                        "The game ended because of a rule violation.".to_string(),
                                    "SOFT_TIMEOUT" =>
                                        "The game ended because a player caused a soft timeout."
                                            .to_string(),
                                    _ => "The game ended because of a hard timeout.".to_string(),
                                }
                            );
                            println!("Result: {}", self.state.game_result());
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

    fn write_to(stream: &TcpStream, data: &str) {
        let _ = BufWriter::new(stream).write(data.as_bytes());
    }
}
