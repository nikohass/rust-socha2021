use game_sdk::{Action, Bitboard, GameState, PieceType};
use std::collections::{HashMap, VecDeque};
use std::io::BufReader;
use std::net::TcpStream;
use xml::reader::{EventReader, XmlEvent};

#[derive(Debug)]
pub struct XmlNode {
    pub name: String,
    pub data: String,
    attribs: HashMap<String, Vec<String>>,
    childs: Vec<XmlNode>,
}

impl XmlNode {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            data: String::new(),
            attribs: HashMap::new(),
            childs: Vec::new(),
        }
    }

    pub fn read_from(xml_parser: &mut EventReader<BufReader<&TcpStream>>) -> Self {
        let mut node_stack: VecDeque<XmlNode> = VecDeque::new();
        let mut has_received_first = false;
        let mut final_node: Option<XmlNode> = None;

        loop {
            match xml_parser.next() {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) => {
                    let mut node = XmlNode::new();
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

    pub fn as_room(&self) -> String {
        let err = "Error while parsing XML node to Room";
        self.get_attribute("roomId").expect(err).to_string()
    }

    pub fn as_memento(&self, state: &mut GameState) {
        let err = "Error while parsing XML node to Memento";
        self.get_child("state").expect(err).update_state(state);
    }

    pub fn update_state(&self, state: &mut GameState) {
        // get the current ply
        let new_ply = self
            .get_attribute("turn")
            .expect("Error while reading turn")
            .parse::<u8>()
            .expect("Error while parsing turn");

        if new_ply == 0 {
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
            println!("    start piece: {}", state.start_piece_type.to_string());
            return;
        }

        if state.ply == new_ply {
            println!("    State did not change since last memento");
            return;
        }

        // get current board
        let mut new_board = [Bitboard::empty(); 4];
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
            new_board[board_index].flip_bit(x + y * 21);
        }

        // find the actions that lead to the new state and update the GameState
        loop {
            let last_board = state.board[state.get_current_color()];
            let changed_fields = new_board[state.get_current_color()] & !last_board;
            let action = Action::from_bitboard(changed_fields);
            println!(
                "{}: {}",
                match state.get_current_color() {
                    0 => "    BLUE".to_string(),
                    1 => "    YELLOW".to_string(),
                    2 => "    RED".to_string(),
                    _ => "    GREEN".to_string(),
                },
                action
            );
            state.do_action(action);
            if state.ply == new_ply {
                break;
            }
        }
    }

    pub fn get_children(&self) -> &Vec<XmlNode> {
        &self.childs
    }

    pub fn get_child(&self, name: &str) -> Option<&XmlNode> {
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
