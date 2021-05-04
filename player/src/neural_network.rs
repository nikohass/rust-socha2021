use super::float_stuff::{relu, sigmoid};
use game_sdk::{Action, ActionList, Bitboard, GameState, Player};
use std::fmt::{Display, Formatter, Result};
use std::fs::File;
use std::io::Read;

type ActivationFunction = fn(f32) -> f32;

pub fn state_to_vector(state: &GameState, al: &mut ActionList) -> Vec<Vec<Vec<f32>>> {
    let mut vector = vec![vec![vec![0.; 5]; 20]; 20];
    state.get_possible_actions(al);
    let mut reachable_fields = Bitboard::empty();
    for i in 0..al.size {
        let action = al[i];
        if action.is_skip() {
            continue;
        }
        let destination = action.get_destination();
        let shape = action.get_shape() as usize;
        let piece = Bitboard::with_piece(destination, shape);
        reachable_fields |= piece;
    }
    while reachable_fields.not_zero() {
        let field_index = reachable_fields.trailing_zeros();
        reachable_fields.flip_bit(field_index);
        let x = field_index % 21;
        let y = (field_index - x) / 21;
        vector[x as usize][y as usize][4] = 1.;
    }
    let mut current_ply = state.ply as usize;
    for i in 0..4 {
        let mut board = state.board[i];
        while board.not_zero() {
            let field_index = board.trailing_zeros();
            board.flip_bit(field_index);
            let x = field_index % 21;
            let y = (field_index - x) / 21;
            vector[x as usize][y as usize][current_ply & 0b11] = 1.;
        }
        current_ply += 1;
    }
    vector
}

pub fn flatten(vector: Vec<Vec<Vec<f32>>>) -> Vec<f32> {
    vector.into_iter().flatten().into_iter().flatten().collect()
}

pub struct DenseLayer {
    input_size: usize,
    output_size: usize,
    weights: Vec<Vec<f32>>,
    biases: Vec<f32>,
    activation: ActivationFunction,
}

impl DenseLayer {
    pub fn with_shape(
        input_size: usize,
        output_size: usize,
        activation: ActivationFunction,
    ) -> Self {
        let weights = vec![vec![0.; output_size]; input_size];
        let biases = vec![0.; output_size];
        Self {
            input_size,
            output_size,
            weights,
            biases,
            activation,
        }
    }

    fn feed_forward(&self, input: &[f32]) -> Vec<f32> {
        let mut output: Vec<f32> = vec![0.; self.output_size];
        for (i, output_neuron) in output.iter_mut().enumerate() {
            for (j, input_neuron) in input.iter().enumerate() {
                *output_neuron += input_neuron * self.weights[j][i];
            }
            *output_neuron = (self.activation)(*output_neuron + self.biases[i]);
        }
        output
    }

    fn load_weights(&mut self, bytes: &[u8], mut index: usize) -> usize {
        for i in 0..self.weights.len() {
            for j in 0..self.weights[i].len() {
                self.weights[i][j] = f32::from_le_bytes([
                    bytes[index],
                    bytes[index + 1],
                    bytes[index + 2],
                    bytes[index + 3],
                ]);
                index += 4;
            }
        }
        for i in 0..self.biases.len() {
            self.biases[i] = f32::from_le_bytes([
                bytes[index],
                bytes[index + 1],
                bytes[index + 2],
                bytes[index + 3],
            ]);
            index += 4;
        }
        index
    }
}

impl Display for DenseLayer {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let shape = &format!("{:?}", (self.input_size, self.output_size));
        write!(
            f,
            "Dense      | {:19} | {}",
            shape,
            if self.activation as usize == sigmoid as usize {
                "Sigmoid"
            } else {
                "ReLU"
            }
        )
    }
}

pub struct ConvolutionalLayer {
    weights: Vec<Vec<Vec<Vec<f32>>>>,
    biases: Vec<f32>,
    kernel_size: usize,
    channels: usize,
    previous_layer_channels: usize,
}

impl ConvolutionalLayer {
    pub fn with_shape(kernel_size: usize, previous_layer_channels: usize, channels: usize) -> Self {
        let weights =
            vec![vec![vec![vec![0.; channels]; previous_layer_channels]; kernel_size]; kernel_size];
        let biases = vec![0.; channels];
        Self {
            weights,
            biases,
            kernel_size,
            channels,
            previous_layer_channels,
        }
    }

    pub fn feed_forward(&self, input: Vec<Vec<Vec<f32>>>) -> Vec<Vec<Vec<f32>>> {
        let input_shape = (input.len(), input[0].len());
        let mut output = vec![vec![vec![0.; self.channels]; input_shape.0]; input_shape.1];
        let offset = self.kernel_size / 2;
        for channel_index in 0..self.channels {
            for (x, out) in output.iter_mut().enumerate().take(input_shape.0) {
                for (y, o) in out.iter_mut().enumerate().take(input_shape.1) {
                    let mut value = 0.;
                    for kx in 0..self.kernel_size {
                        let mut x_ = kx + x;
                        if x_ >= input_shape.0 + offset || x_ < offset {
                            continue;
                        } else {
                            x_ -= offset;
                        }
                        for ky in 0..self.kernel_size {
                            let mut y_ = ky + y;
                            if y_ >= input_shape.0 + offset || y_ < offset {
                                continue;
                            } else {
                                y_ -= offset;
                            }
                            for prev_channel_index in 0..self.previous_layer_channels {
                                value += input[x_][y_][prev_channel_index]
                                    * self.weights[kx][ky][prev_channel_index][channel_index];
                            }
                        }
                        o[channel_index] = relu(value + self.biases[channel_index]);
                    }
                }
            }
        }
        output
    }

    fn load_weights(&mut self, bytes: &[u8], mut index: usize) -> usize {
        for i in 0..self.weights.len() {
            for j in 0..self.weights[i].len() {
                for k in 0..self.weights[i][j].len() {
                    for l in 0..self.weights[i][j][k].len() {
                        self.weights[i][j][k][l] = f32::from_le_bytes([
                            bytes[index],
                            bytes[index + 1],
                            bytes[index + 2],
                            bytes[index + 3],
                        ]);
                        index += 4;
                    }
                }
            }
        }
        for i in 0..self.biases.len() {
            self.biases[i] = f32::from_le_bytes([
                bytes[index],
                bytes[index + 1],
                bytes[index + 2],
                bytes[index + 3],
            ]);
            index += 4;
        }
        index
    }
}

impl Display for ConvolutionalLayer {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let shape = &format!(
            "{:?}",
            (
                self.weights.len(),
                self.weights[0].len(),
                self.weights[0][0].len(),
                self.weights[0][0][0].len()
            )
        );
        write!(f, "Conv2D     | {:19} | ReLU", shape)
    }
}

pub struct NeuralNetwork {
    convolutional_layers: Vec<ConvolutionalLayer>,
    dense_layers: Vec<DenseLayer>,
}

impl NeuralNetwork {
    pub fn new(weights_file: &str) -> Option<Self> {
        let mut nn = Self::default();
        nn.add_convolutional_layer(ConvolutionalLayer::with_shape(7, 5, 128));
        nn.add_convolutional_layer(ConvolutionalLayer::with_shape(5, 128, 32));
        nn.add_convolutional_layer(ConvolutionalLayer::with_shape(3, 32, 32));
        nn.add_convolutional_layer(ConvolutionalLayer::with_shape(3, 32, 32));
        nn.add_convolutional_layer(ConvolutionalLayer::with_shape(3, 32, 1));
        nn.add_dense_layer(DenseLayer::with_shape(400, 400, relu));
        nn.add_dense_layer(DenseLayer::with_shape(400, 400, sigmoid));

        println!("{}", nn);
        if nn.load_weights(weights_file) {
            Some(nn)
        } else {
            None
        }
    }

    pub fn feed_forward(&self, input: Vec<Vec<Vec<f32>>>) -> Vec<f32> {
        let mut previous_layer_output = input;
        for layer in self.convolutional_layers.iter() {
            previous_layer_output = layer.feed_forward(previous_layer_output);
        }
        let mut previous_layer_output = flatten(previous_layer_output);
        for layer in self.dense_layers.iter() {
            previous_layer_output = layer.feed_forward(&previous_layer_output);
        }
        previous_layer_output
    }

    pub fn add_convolutional_layer(&mut self, layer: ConvolutionalLayer) {
        self.convolutional_layers.push(layer);
    }

    pub fn add_dense_layer(&mut self, layer: DenseLayer) {
        self.dense_layers.push(layer);
    }

    pub fn load_weights(&mut self, weights_file: &str) -> bool {
        print!("Loading weights from \"{}\"... ", weights_file);
        let file = File::open(weights_file);
        let mut index: usize = 0;
        let mut bytes = Vec::new();
        match file {
            Ok(mut file) => file.read_to_end(&mut bytes).unwrap(),
            Err(error) => {
                println!("Unable to load weights file: {}", error);
                return false;
            }
        };
        print!("({} bytes, {} parameters) ", bytes.len(), bytes.len() / 4);
        for layer in self.convolutional_layers.iter_mut() {
            index = layer.load_weights(&bytes, index);
        }
        for layer in self.dense_layers.iter_mut() {
            index = layer.load_weights(&bytes, index);
        }
        if index != bytes.len() {
            println!("warning: The length of the weights file does not match the number of network parameters.");
            false
        } else {
            println!("Weights loaded successfully");
            true
        }
    }
}

impl Display for NeuralNetwork {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut string = "Layer Type | Shape               | Activation\n".to_string();
        for layer in self.convolutional_layers.iter() {
            string.push_str(&layer.to_string());
            string.push('\n');
        }
        string.push_str("Flatten    |                     |\n");
        for layer in self.dense_layers.iter() {
            string.push_str(&layer.to_string());
            string.push('\n');
        }
        string.pop();
        write!(f, "{}", string)
    }
}

impl Default for NeuralNetwork {
    fn default() -> Self {
        Self {
            convolutional_layers: Vec::new(),
            dense_layers: Vec::new(),
        }
    }
}

impl Player for NeuralNetwork {
    fn on_move_request(&mut self, state: &GameState) -> Action {
        let mut state = state.clone();
        let r = BoardRotation::rotate_state(&mut state);
        let mut al = ActionList::default();
        let input = state_to_vector(&state, &mut al);
        let output = self.feed_forward(input);
        let mut best_value = std::f32::NEG_INFINITY;
        let mut best_action = al[0];
        for i in 0..al.size {
            let action = al[i];
            if action.is_skip() {
                continue;
            }
            let mut value: f32 = 0.;
            let destination = action.get_destination();
            let shape = action.get_shape() as usize;
            let mut piece = Bitboard::with_piece(destination, shape);
            piece = r.rotate_bitboard(piece);
            while piece.not_zero() {
                let field_index = piece.trailing_zeros();
                piece.flip_bit(field_index);
                let x = field_index % 21;
                let y = (field_index - x) / 21;
                value += output[(x + y * 20) as usize];
            }
            if value > best_value {
                best_value = value;
                best_action = action;
            }
        }
        r.rotate_action(best_action)
    }
}

pub struct BoardRotation {
    top_left_corner: u8,
}

impl BoardRotation {
    pub fn rotate_state(state: &mut GameState) -> BoardRotation {
        let board = state.board[state.get_current_color() as usize];
        let top_left_corner = if board.check_bit(0) {
            0
        } else if board.check_bit(19) {
            1
        } else if board.check_bit(399) {
            2
        } else {
            3
        };
        let board_rotation = Self { top_left_corner };
        for board in state.board.iter_mut() {
            *board = board_rotation.rotate_bitboard(*board);
        }
        board_rotation
    }

    pub fn rotate_bitboard(&self, board: Bitboard) -> Bitboard {
        match self.top_left_corner {
            1 => board.mirror(),
            2 => board.flip(),
            3 => board.rotate_left().rotate_left(),
            _ => board,
        }
    }

    pub fn rotate_bitboard_back(&self, board: Bitboard) -> Bitboard {
        match self.top_left_corner {
            1 => board.mirror(),
            2 => board.flip(),
            3 => board.rotate_left().rotate_left(),
            _ => board,
        }
    }

    pub fn rotate_action(&self, action: Action) -> Action {
        if action.is_set() {
            let to = action.get_destination();
            let shape = action.get_shape() as usize;
            Action::from_bitboard(self.rotate_bitboard_back(Bitboard::with_piece(to, shape)))
        } else {
            action
        }
    }
}
