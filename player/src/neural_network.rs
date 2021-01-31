use game_sdk::*;
use std::fs::File;
use std::io::Read;

const INPUT_DIMS: usize = 1692;
const OUTPUT_DIMS: usize = 400;

#[inline(always)]
fn sigmoid(x: f32) -> f32 {
    1. / (1. + (-x).exp())
}

#[inline(always)]
fn relu(x: f32) -> f32 {
    f32::max(0., x)
}

pub struct Layer {
    input_size: usize,
    output_size: usize,
    weights: Vec<Vec<f32>>,
    biases: Vec<f32>,
    sigmoid: bool,
}

impl Layer {
    pub fn with_shape(input_size: usize, output_size: usize, sigmoid: bool) -> Layer {
        let weights = vec![vec![0.; output_size]; input_size];
        let biases = vec![0.; output_size];
        Layer {
            input_size,
            output_size,
            weights,
            biases,
            sigmoid,
        }
    }

    pub fn feed_forward_relu(&self, input: &[f32]) -> Vec<f32> {
        let mut output: Vec<f32> = vec![0.; self.output_size];
        for (i, output_neuron) in output.iter_mut().enumerate() {
            for (j, input_neuron) in input.iter().enumerate() {
                *output_neuron += input_neuron * self.weights[j][i];
            }
            *output_neuron = relu(*output_neuron + self.biases[i]);
        }
        output
    }

    pub fn feed_forward_sigmoid(&self, input: &[f32]) -> Vec<f32> {
        let mut output: Vec<f32> = vec![0.; self.output_size];
        for (i, output_neuron) in output.iter_mut().enumerate() {
            for (j, input_neuron) in input.iter().enumerate() {
                *output_neuron += input_neuron * self.weights[j][i];
            }
            *output_neuron = sigmoid(*output_neuron + self.biases[i]);
        }
        output
    }
}

pub struct NeuralNetwork {
    layers: Vec<Layer>,
}

impl NeuralNetwork {
    pub fn policy_network() -> NeuralNetwork {
        let layers: Vec<Layer> = vec![
            Layer::with_shape(INPUT_DIMS, INPUT_DIMS, false),
            Layer::with_shape(INPUT_DIMS, 1024, false),
            Layer::with_shape(1024, 1024, false),
            Layer::with_shape(1024, 1024, false),
            Layer::with_shape(1024, 1024, false),
            Layer::with_shape(1024, 1024, false),
            Layer::with_shape(1024, OUTPUT_DIMS, true),
        ];
        NeuralNetwork { layers }
    }

    pub fn load_weights(&mut self, weights_file: &str) -> bool {
        println!("loading weights from \"{}\"...", weights_file);
        let file = File::open(weights_file);
        let mut bytes = Vec::new();
        match file {
            Ok(mut file) => file.read_to_end(&mut bytes).unwrap(),
            Err(error) => {
                println!("Unable to load weights file {}", error);
                return false;
            }
        };
        println!("bytes: {} parameters: {}", bytes.len(), bytes.len() / 4);
        let mut byte_index: usize = 0;

        println!("Layer index | input neurons | output neurons | parameters");
        for layer_index in 0..self.layers.len() {
            print!(
                "{:11} | {:13} | {:14} |",
                layer_index,
                self.layers[layer_index].input_size,
                self.layers[layer_index].output_size
            );
            for i in 0..self.layers[layer_index].weights.len() {
                for j in 0..self.layers[layer_index].weights[i].len() {
                    self.layers[layer_index].weights[i][j] = f32::from_le_bytes([
                        bytes[byte_index],
                        bytes[byte_index + 1],
                        bytes[byte_index + 2],
                        bytes[byte_index + 3],
                    ]);
                    byte_index += 4;
                }
            }
            println!(
                "{:10}",
                self.layers[layer_index].biases.len()
                    + self.layers[layer_index].input_size * self.layers[layer_index].output_size
            );
            for i in 0..self.layers[layer_index].biases.len() {
                self.layers[layer_index].biases[i] = f32::from_le_bytes([
                    bytes[byte_index],
                    bytes[byte_index + 1],
                    bytes[byte_index + 2],
                    bytes[byte_index + 3],
                ]);
                byte_index += 4;
            }
        }
        println!("Network parameters have been loaded.");
        if bytes.len() != byte_index {
            println!("WARNING: The length of the weights file does not match the number of network parameters.");
        }
        bytes.len() == byte_index
    }

    pub fn feed_forward(&self, input: &mut Vec<f32>) -> Vec<f32> {
        let mut output: Vec<f32>;
        for layer in self.layers.iter() {
            output = if layer.sigmoid {
                layer.feed_forward_sigmoid(&input)
            } else {
                layer.feed_forward_relu(&input)
            };
            *input = output;
        }
        input.to_vec()
    }

    pub fn pick_action(&self, state: &GameState) -> (Action, f32) {
        let mut state = state.clone();
        let rotation = Rotation::from_state(&state);
        rotation.rotate_state(&mut state);
        let mut action_list = ActionList::default();
        state.get_possible_actions(&mut action_list);
        if action_list[0] == Action::Skip {
            return (Action::Skip, std::f32::INFINITY);
        }
        let mut input_vector = state_to_vector(&state);
        let output_vector = self.feed_forward(&mut input_vector);

        let mut highest_confidence: f32 = 0.;
        let mut best_action: usize = 0;
        for index in 0..action_list.size {
            let mut confidence: f32 = 0.;
            if let Action::Set(to, shape_index) = action_list[index] {
                let mut action_board = Bitboard::with_piece(to, shape_index);
                while action_board.not_zero() {
                    let bit_index = action_board.trailing_zeros();
                    action_board.flip_bit(bit_index);
                    let x = bit_index % 21;
                    let y = (bit_index - x) / 21;
                    confidence += output_vector[(x + y * 20) as usize];
                }
            }
            if confidence > highest_confidence {
                highest_confidence = confidence;
                best_action = index;
            }
        }

        (
            rotation.rotate_action(action_list[best_action]),
            highest_confidence,
        )
    }

    pub fn sort_actions(&self, state: &GameState, action_list: &mut ActionList) -> Vec<f32> {
        state.get_possible_actions(action_list);
        let mut input_vector = state_to_vector(&state);
        let output_vector = self.feed_forward(&mut input_vector);
        let mut confidence_vec: Vec<f32> = vec![std::f32::NEG_INFINITY; action_list.size];

        for index in 0..action_list.size {
            let mut confidence: f32 = 0.;
            if let Action::Set(to, shape_index) = action_list[index] {
                let mut action_board = Bitboard::with_piece(to, shape_index);
                while action_board.not_zero() {
                    let bit_index = action_board.trailing_zeros();
                    action_board.flip_bit(bit_index);
                    let x = bit_index % 21;
                    let y = (bit_index - x) / 21;
                    confidence += output_vector[(x + y * 20) as usize];
                }
            }
            confidence_vec[index] = confidence;
        }

        for i in 0..action_list.size {
            let mut max_value = std::f32::NEG_INFINITY;
            let mut next_best_action_index = 0;
            for (j, conf) in confidence_vec
                .iter()
                .enumerate()
                .take(action_list.size)
                .skip(i)
            {
                if *conf > max_value {
                    max_value = confidence_vec[j];
                    next_best_action_index = j;
                }
            }

            action_list.swap(i, next_best_action_index);
            confidence_vec.swap(next_best_action_index, i);
        }
        confidence_vec
    }

    pub fn append_principal_variation(
        &self,
        principal_variation: &mut ActionList,
        state: &GameState,
    ) -> (Action, f32) {
        let mut state = state.clone();
        for i in 0..principal_variation.size {
            state.do_action(principal_variation[i]);
        }
        let (action, confidence) = self.pick_action(&state);
        principal_variation.push(action);
        (action, confidence)
    }
}

pub fn state_to_vector(state: &GameState) -> Vec<f32> {
    let mut vector: Vec<f32> = vec![0.; INPUT_DIMS];
    let mut index: usize = 0;
    let mut color = state.current_color;
    for _ in 0..4 {
        for x in 0..20 {
            for y in 0..20 {
                if state.board[color as usize].check_bit(x + y * 21) {
                    vector[index] = 1.;
                }
                index += 1;
            }
        }
        color = color.next();
    }

    color = state.current_color;
    for _ in 0..4 {
        for i in 0..21 {
            if state.pieces_left[i][color as usize] {
                vector[index] = 1.;
            }
            index += 1;
        }
    }

    let mut bit: u8 = 1;
    for entry in vector.iter_mut().take(index + 8).skip(index) {
        if state.ply & bit == bit {
            *entry = 1.;
        }
        bit <<= 1;
    }
    vector
}

pub fn bitboard_to_vector(board: Bitboard) -> Vec<f32> {
    let mut vector: Vec<f32> = vec![0.; 400];
    for x in 0..20 {
        for y in 0..20 {
            if board.check_bit(x + y * 21) {
                vector[(y + x * 20) as usize] = 1.;
            }
        }
    }
    vector
}

pub struct Rotation {
    pub mirror: bool,
    pub top_left_corner: u8,
}

impl Rotation {
    pub fn new(mirror: bool, top_left_corner: u8) -> Rotation {
        Rotation {
            mirror,
            top_left_corner,
        }
    }

    pub fn from_state(state: &GameState) -> Rotation {
        let board = state.board[state.current_color as usize];
        let top_left_corner = if board.check_bit(0) {
            0
        } else if board.check_bit(19) {
            1
        } else if board.check_bit(399) {
            2
        } else {
            3
        };
        let mut mirror = false;
        for (a, b) in [(1, 21), (2, 42), (23, 43), (24, 64)].iter() {
            let c = board.check_bit(*a);
            let d = board.check_bit(*b);
            if c && !d {
                mirror = true;
                break;
            }
            if !c && d {
                break;
            }
        }
        Rotation {
            mirror,
            top_left_corner,
        }
    }

    pub fn rotate_bitboard(&self, board: Bitboard) -> Bitboard {
        let board = match self.top_left_corner {
            1 => board.mirror(),
            2 => board.flip(),
            3 => board.rotate_left().rotate_left(),
            _ => board,
        };
        if self.mirror {
            board.mirror_diagonal()
        } else {
            board
        }
    }

    pub fn rotate_bitboard_back(&self, board: Bitboard) -> Bitboard {
        let board = if self.mirror {
            board.mirror_diagonal()
        } else {
            board
        };
        match self.top_left_corner {
            1 => board.mirror(),
            2 => board.flip(),
            3 => board.rotate_left().rotate_left(),
            _ => board,
        }
    }

    pub fn rotate_state(&self, state: &mut GameState) {
        for board in state.board.iter_mut() {
            *board = self.rotate_bitboard(*board);
        }
    }

    pub fn rotate_action(&self, action: Action) -> Action {
        match action {
            Action::Set(to, shape_index) => Action::from_bitboard(
                self.rotate_bitboard_back(Bitboard::with_piece(to, shape_index)),
            ),
            Action::Skip => action,
        }
    }
}
