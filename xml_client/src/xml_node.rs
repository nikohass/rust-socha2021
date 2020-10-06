use game_sdk::bitboard::Bitboard;
use game_sdk::color::Color;
use game_sdk::gamestate::GameState;
use game_sdk::piece_type::PieceType;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::BufReader;
use std::net::TcpStream;
use std::vec::Vec;
use xml::reader::{EventReader, XmlEvent};

#[derive(Debug)]
pub struct XMLNode {
    pub name: String,
    pub data: String,
    attribs: HashMap<String, Vec<String>>,
    childs: Vec<XMLNode>,
}

impl XMLNode {
    pub fn new() -> XMLNode {
        XMLNode {
            name: String::new(),
            data: String::new(),
            attribs: HashMap::new(),
            childs: Vec::new(),
        }
    }
    pub fn read_from(xml_parser: &mut EventReader<BufReader<&TcpStream>>) -> XMLNode {
        let mut node_stack: VecDeque<XMLNode> = VecDeque::new();
        let mut has_received_first = false;
        let mut final_node: Option<XMLNode> = None;

        loop {
            match xml_parser.next() {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) => {
                    let mut node = XMLNode::new();
                    node.name = name.local_name;
                    for attribute in attributes {
                        let attrib_name = attribute.name.local_name;
                        if !node.attribs.contains_key(&attrib_name) {
                            node.attribs.insert(attrib_name.to_string(), Vec::new());
                        }
                        node.attribs
                            .get_mut(&attrib_name)
                            .unwrap()
                            .push(attribute.value.to_string());
                    }
                    node_stack.push_back(node);
                    has_received_first = true;
                }
                Ok(XmlEvent::EndElement { .. }) => {
                    if node_stack.len() > 2 {
                        let child = node_stack.pop_back().expect("Unexpectedly found empty XML node stack while trying to pop off new child element");
                        let mut node = node_stack.pop_back().expect("Unexpectedly found empty XML node stack while trying to hook up new child element");
                        node.childs.push(child);
                        node_stack.push_back(node);
                    } else if has_received_first {
                        final_node = Some(node_stack.pop_back().expect(
                            "Unexpectedly found empty XML node stack while trying to return node",
                        ));
                    }
                }
                Ok(XmlEvent::Characters(content)) => {
                    node_stack.back_mut().expect("Unexpectedly found empty XML node stack while trying to add characters").data += content.as_str();
                }
                Err(_) => {
                    break;
                }
                _ => {}
            }
            if final_node.is_some() {
                break;
            }
        }
        final_node.unwrap()
    }

    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        self.attribs.get(name).map(|a| &a[0])
    }

    pub fn as_welcome_message(&self) -> usize {
        let err = "Error while parsing XML node to WelcomeMessage";
        match self.get_attribute("color").expect(err).as_str() {
            "one" => 0,
            "two" => 1,
            _ => panic!(err),
        }
    }

    pub fn as_room(&self) -> String {
        let err = "Error while parsing XML node to Room";
        self.get_attribute("roomId").expect(err).to_string()
    }

    pub fn as_memento(&self, old_state: &mut GameState) {
        let err = "Error while parsing XML node to Memento";
        self.get_child("state").expect(err).update_state(old_state);
    }

    pub fn get_child(&self, name: &str) -> Option<&XMLNode> {
        for child in &self.childs {
            if child.name.as_str() == name {
                return Some(&child);
            }
        }
        None
    }

    pub fn update_state(&self, state: &mut GameState) {
        let new_ply = self
            .get_attribute("turn")
            .expect("Error while reading turn")
            .parse::<u8>()
            .expect("Error while parsing turn");

        state.ply = new_ply;

        if new_ply == 0 {
            state.start_piece_type = match self
                .get_attribute("startPiece")
                .expect("Error while reading start piece")
                .as_ref()
            {
                "PENTO_L" => PieceType::LPentomino,
                "PENTO_T" => PieceType::TPentomino,
                "PENTO_V" => PieceType::VPentomino,
                "PENTO_S" => PieceType::NPentomino,
                "PENTO_Z" => PieceType::ZPentomino,
                "PENTO_I" => PieceType::IPentomino,
                "PENTO_P" => PieceType::PPentomino,
                "PENTO_W" => PieceType::WPentomino,
                "PENTO_U" => PieceType::UPentomino,
                "PENTO_R" => PieceType::FPentomino,
                "PENTO_Y" => PieceType::YPentomino,
                _ => panic!("Unknown start piece"),
            };
            println!("Start piece type is {}", state.start_piece_type.to_string());
        }

        let mut pieces_left = [[false; 4]; 21];
        let vec = &self
            .get_child("undeployedPieceShapes")
            .expect("Error while reading undeployedPieceShapes")
            .get_children();
        for c in 0..4 {
            let children = vec[c].get_children();
            for child in children.iter().take(4) {
                for piece_type in child.get_children().iter() {
                    let index = match piece_type.data.as_ref() {
                        "MONO" => 0,
                        "DOMINO" => 1,
                        "TRIO_I" => 2,
                        "TRIO_L" => 3,
                        "TETRO_I" => 4,
                        "TETRO_L" => 5,
                        "TETRO_T" => 6,
                        "TETRO_O" => 7,
                        "TETRO_Z" => 8,
                        "PENTO_R" => 9,
                        "PENTO_I" => 10,
                        "PENTO_L" => 11,
                        "PENTO_S" => 12,
                        "PENTO_P" => 13,
                        "PENTO_T" => 14,
                        "PENTO_U" => 15,
                        "PENTO_V" => 16,
                        "PENTO_W" => 17,
                        "PENTO_X" => 18,
                        "PENTO_Y" => 19,
                        "PENTO_Z" => 20,
                        _ => panic!("Invalid piece name"),
                    };
                    pieces_left[index][c] = true;
                }
            }
        }
        state.pieces_left = pieces_left;

        state.board = [Bitboard::new(); 4]; // clear boards
        for (x, node) in self
            .get_child("board")
            .expect("Error while reading board")
            .get_child("gameField")
            .expect("Error while reading gameField")
            .get_children()
            .iter()
            .enumerate()
        {
            for (y, field) in node.get_children().iter().enumerate() {
                let to = y as u16 + (x as u16) * 21;
                match field.data.as_ref() {
                    "BLUE" => state.board[0].flip_bit(to),
                    "YELLOW" => state.board[1].flip_bit(to),
                    "RED" => state.board[2].flip_bit(to),
                    "GREEN" => state.board[3].flip_bit(to),
                    _ => {}
                }
            }
        }
        /*
        for field in vec.iter() {
            println!("field.data = {}", field.data);

            let x = field
                .get_attribute("x")
                .expect("error")
                .parse::<u16>()
                .expect(err);
            let y = field
                .get_attribute("y")
                .expect("error")
                .parse::<u16>()
                .expect(err);
            let to = x + y * 21;

            let board_index = match field.get_attribute("content").expect("error").as_ref() {
                "BLUE" => 0,
                "YELLOW" => 1,
                "RED" => 2,
                _ => 3,
            };
            state.board[board_index].flip_bit(to);
        }*/
        let current_player_index = self // currentColorIndex does not behave like described in the xml documentation
            .get_attribute("currentColorIndex")
            .expect("Error while reading currentColorIndex")
            .parse::<usize>()
            .expect("Error while parsing currentColorIndex");

        let mut active_players = Vec::new();
        for i in 0..4 {
            //if state.skipped & 1 << i == 0 {
            active_players.push(match i {
                0 => Color::BLUE,
                1 => Color::YELLOW,
                2 => Color::RED,
                _ => Color::GREEN,
            });
            //}
        }

        state.current_player = active_players[current_player_index];

        println!(
            "Updated state: turn {}, player {} ",
            state.ply,
            state.current_player.to_string(),
        );

        if !state.check_integrity() {
            println!("Integrity check failed!");
        }
    }

    pub fn get_children(&self) -> &Vec<XMLNode> {
        &self.childs
    }
}
