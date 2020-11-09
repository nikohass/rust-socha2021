use game_sdk::{Bitboard, Color, GameState, PieceType};
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

    pub fn as_memento(&self, state: &mut GameState) {
        let err = "Error while parsing XML node to Memento";
        self.get_child("state").expect(err).update_state(state);
    }

    pub fn update_state(&self, state: &mut GameState) {
        // update board
        {
            let mut new_board = [Bitboard::new(); 4];
            let vec = &self
                .get_child("board")
                .expect("Error while reading board")
                .get_children();
            for field in vec.iter() {
                let x = field
                    .get_attribute("x")
                    .expect("Error while reading x")
                    .parse::<u16>()
                    .expect("Error while parsing x");
                let y = field
                    .get_attribute("y")
                    .expect("Error while reading y")
                    .parse::<u16>()
                    .expect("Error while parsing y");
                let to = x + y * 21;

                let board_index = match field
                    .get_attribute("content")
                    .expect("Error while reading field content")
                    .as_ref()
                {
                    "BLUE" => 0,
                    "YELLOW" => 1,
                    "RED" => 2,
                    _ => 3,
                };
                new_board[board_index].flip_bit(to);
            }
            state.board = new_board;
        }

        // update pieces left
        {
            let mut pieces_left = [[false; 4]; 21];
            for (c, child_name) in ["blueShapes", "yellowShapes", "redShapes", "greenShapes"]
                .iter()
                .enumerate()
                .take(4)
            {
                let child = &self
                    .get_child(child_name)
                    .expect("Error while reading undeployedPieceShapes");
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
            state.pieces_left = pieces_left;
        }

        // update current player
        {
            let ordered_colors = &self
                .get_child("orderedColors")
                .expect("Error while reading orderedColors")
                .get_children();
            let current_player_index = self
                .get_attribute("currentColorIndex")
                .expect("Error while reading currentColorIndex")
                .parse::<usize>()
                .expect("Error while parsing currentColorIndex");

            let mut active_player_vec = Vec::new();
            for color in ordered_colors.iter() {
                active_player_vec.push(match color.data.as_str() {
                    "BLUE" => Color::BLUE,
                    "YELLOW" => Color::YELLOW,
                    "RED" => Color::RED,
                    _ => Color::GREEN,
                });
            }
            if current_player_index < active_player_vec.len() {
                state.current_player = active_player_vec[current_player_index];
            }
        }

        // update ply
        {
            let round = self
                .get_attribute("round")
                .expect("Error while reading round")
                .parse::<u8>()
                .expect("Error while parsing turn")
                - 1;

            state.ply = round * 4 + state.current_player as u8;
        }

        if state.ply == 0 {
            // update start piece type
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

        println!(
            "Updated state: ply {}, player {} ",
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

    pub fn get_child(&self, name: &str) -> Option<&XMLNode> {
        for child in &self.childs {
            if child.name.as_str() == name {
                return Some(&child);
            }
        }
        None
    }

    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        self.attribs.get(name).map(|a| &a[0])
    }
}
