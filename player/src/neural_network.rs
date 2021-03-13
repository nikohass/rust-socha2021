use super::float_stuff::{relu, sigmoid};
use game_sdk::{Action, ActionList, Bitboard, GameState, Player};
use std::fmt::{Display, Formatter, Result};
use std::fs::File;
use std::io::Read;

pub fn state_to_vector(state: &GameState) -> Vec<Vec<Vec<f32>>> {
    let mut vector = vec![vec![vec![0.; 4]; 20]; 20];
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

pub fn flatten(vector: &[Vec<Vec<f32>>]) -> Vec<f32> {
    let size = vector.len() * vector[0].len() * vector[1].len();
    let mut ret: Vec<f32> = Vec::with_capacity(size);
    for i in vector.iter() {
        for j in i.iter() {
            for k in j.iter() {
                ret.push(*k);
            }
        }
    }
    ret
}

pub struct DenseLayer {
    input_size: usize,
    output_size: usize,
    weights: Vec<Vec<f32>>,
    biases: Vec<f32>,
    sigmoid: bool,
}

impl DenseLayer {
    pub fn with_shape(input_size: usize, output_size: usize, sigmoid: bool) -> Self {
        let weights = vec![vec![0.; output_size]; input_size];
        let biases = vec![0.; output_size];
        Self {
            input_size,
            output_size,
            weights,
            biases,
            sigmoid,
        }
    }

    pub fn feed_forward(&self, input: &[f32]) -> Vec<f32> {
        if self.sigmoid {
            self.feed_forward_sigmoid(input)
        } else {
            self.feed_forward_relu(input)
        }
    }

    fn feed_forward_relu(&self, input: &[f32]) -> Vec<f32> {
        let mut output: Vec<f32> = vec![0.; self.output_size];
        for (i, output_neuron) in output.iter_mut().enumerate() {
            for (j, input_neuron) in input.iter().enumerate() {
                *output_neuron += input_neuron * self.weights[j][i];
            }
            *output_neuron = relu(*output_neuron + self.biases[i]);
        }
        output
    }

    fn feed_forward_sigmoid(&self, input: &[f32]) -> Vec<f32> {
        let mut output: Vec<f32> = vec![0.; self.output_size];
        for (i, output_neuron) in output.iter_mut().enumerate() {
            for (j, input_neuron) in input.iter().enumerate() {
                *output_neuron += input_neuron * self.weights[j][i];
            }
            *output_neuron = sigmoid(*output_neuron + self.biases[i]);
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
            if self.sigmoid { "sigmoid" } else { "relu" }
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
    pub fn with_shape(kernel_size: usize, channels: usize, previous_layer_channels: usize) -> Self {
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
        write!(f, "Conv2D     | {:19} | relu", shape)
    }
}

pub struct NeuralNetwork {
    convolutional_layers: Vec<ConvolutionalLayer>,
    dense_layers: Vec<DenseLayer>,
}

impl NeuralNetwork {
    pub fn new(weights_file: &str) -> Self {
        let mut nn = Self::default();
        nn.add_convolutional_layer(ConvolutionalLayer::with_shape(7, 128, 4));
        nn.add_convolutional_layer(ConvolutionalLayer::with_shape(7, 128, 128));
        nn.add_convolutional_layer(ConvolutionalLayer::with_shape(5, 128, 128));
        nn.add_convolutional_layer(ConvolutionalLayer::with_shape(5, 64, 128));
        nn.add_convolutional_layer(ConvolutionalLayer::with_shape(5, 64, 64));
        nn.add_convolutional_layer(ConvolutionalLayer::with_shape(5, 64, 64));
        nn.add_convolutional_layer(ConvolutionalLayer::with_shape(3, 32, 64));
        nn.add_convolutional_layer(ConvolutionalLayer::with_shape(3, 4, 32));

        nn.add_dense_layer(DenseLayer::with_shape(1600, 800, false));
        nn.add_dense_layer(DenseLayer::with_shape(800, 400, true));

        nn.load_weights(weights_file);
        nn
    }

    pub fn feed_forward(&self, input: Vec<Vec<Vec<f32>>>) -> Vec<f32> {
        let mut previous_layer_output = input;
        for layer in self.convolutional_layers.iter() {
            previous_layer_output = layer.feed_forward(previous_layer_output);
        }
        let mut previous_layer_output = flatten(&previous_layer_output);
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
        println!("loading weights from \"{}\"...", weights_file);
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
        println!("Bytes: {} Parameters: {}", bytes.len(), bytes.len() / 4);
        for layer in self.convolutional_layers.iter_mut() {
            index = layer.load_weights(&bytes, index);
        }
        for layer in self.dense_layers.iter_mut() {
            index = layer.load_weights(&bytes, index);
        }
        if index != bytes.len() {
            println!("WARNING: The length of the weights file does not match the number of network parameters.");
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
        for _ in 0..50 {
            string.push('=');
        }
        string.push('\n');
        for layer in self.convolutional_layers.iter() {
            string.push_str(&layer.to_string());
            string.push('\n');
        }
        string.push_str("Flatten    |                     |\n");
        for layer in self.dense_layers.iter() {
            string.push_str(&layer.to_string());
            string.push('\n');
        }

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
        let rotation = Rotation::from_state(&state);
        rotation.rotate_state(&mut state);
        let mut al = ActionList::default();
        state.get_possible_actions(&mut al);
        if al[0] == Action::Skip {
            return Action::Skip;
        }
        let input_vector = state_to_vector(&state);
        let output_vector = self.feed_forward(input_vector);

        let mut highest_confidence: f32 = 0.;
        let mut best_action: usize = 0;
        for index in 0..al.size {
            let mut confidence: f32 = 0.;
            if let Action::Set(to, shape) = al[index] {
                let mut action_board = Bitboard::with_piece(to, shape);
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
        println!("Confidence: {}", highest_confidence);
        rotation.rotate_action(al[best_action])
    }
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
            Action::Set(to, shape) => {
                Action::from_bitboard(self.rotate_bitboard_back(Bitboard::with_piece(to, shape)))
            }
            Action::Skip => action,
        }
    }
}
