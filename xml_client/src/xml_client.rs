use game_sdk::GameState;
use std::io::{prelude::Write, BufReader, BufWriter};
use std::net::TcpStream;
extern crate xml;
use self::xml::reader::*;
use super::xml_node::XMLNode;
use player::search::search_action;

pub struct XMLClient {
    my_team: Option<usize>,
    state: GameState,
    host: String,
    port: String,
    reservation: String,
    room_id: Option<String>,
    active_players: [bool; 4],
}

impl XMLClient {
    pub fn new(host: String, port: String, reservation: String) -> XMLClient {
        XMLClient {
            my_team: None,
            state: GameState::default(),
            host,
            port,
            reservation,
            room_id: None,
            active_players: [true; 4],
        }
    }

    pub fn run(&mut self) {
        println!("Connecting to {}:{}...", self.host, self.port);
        let stream = TcpStream::connect(&format!("{}:{}", self.host, self.port))
            .expect("Could not connect to server");
        println!("Connected");
        XMLClient::write_to(&stream, "<protocol>");

        let join_xml: String;
        match self.reservation.as_str() {
            "" => join_xml = "<join gameType=\"swc_2021_blokus\"/>".to_string(),
            _ => join_xml = format!("<joinPrepared reservationCode=\"{}\" />", self.reservation),
        }

        println!("Sending join message: {}", join_xml);
        XMLClient::write_to(&stream, join_xml.as_str());

        self.handle_stream(&stream);
    }

    fn handle_stream(&mut self, stream: &TcpStream) {
        let mut parser = EventReader::new(BufReader::new(stream));

        loop {
            let mut node = XMLNode::read_from(&mut parser);

            match node.name.as_str() {
                "data" => {
                    let invalid = &"".to_string();
                    let data_class = node.get_attribute("class").unwrap_or(invalid).to_string();
                    match data_class.as_str() {
                        "memento" => {
                            println!("Recieved memento");
                            node.as_memento(&mut self.state, &mut self.active_players);
                        }
                        "welcomeMessage" => {
                            println!("Recieved welcome message");
                            self.handle_welcome_message_node(&mut node)
                        }
                        "sc.framework.plugins.protocol.MoveRequest" => {
                            println!("Recieved move request");
                            let action = search_action(&self.state);
                            let xml_move = action.to_xml(self.state.current_player);
                            println!("Sending: {}", action);
                            XMLClient::write_to(
                                stream,
                                &format!(
                                    "<room roomId=\"{}\">\n{}\n</room>",
                                    self.room_id.as_ref().expect("error"),
                                    xml_move
                                ),
                            );
                        }
                        "result" => {
                            println!("got result");
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

    fn handle_welcome_message_node(&mut self, node: &mut XMLNode) {
        let team = node.as_welcome_message();
        self.my_team = Some(team);
    }

    fn write_to(stream: &TcpStream, data: &str) {
        let _ = BufWriter::new(stream).write(data.as_bytes());
    }
}
