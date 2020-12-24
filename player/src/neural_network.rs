use game_sdk::*;
use std::fs::File;
use std::io::Read;

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
    pub fn with_size(input_size: usize, output_size: usize, sigmoid: bool) -> Layer {
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

    /*
    pub fn random_weights(&mut self) {
        let mut rng = SmallRng::from_entropy();
        for i in 0..self.output_size {
            for j in 0..self.input_size {
                self.weights[j][i] = (rng.next_u64() as f32 / (std::u64::MAX as f32)) * 2. - 1.;
            }
            self.biases[i] = (rng.next_u64() as f32 / (std::u64::MAX as f32)) * 2. - 1.;
        }
    }*/

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
    pub fn new() -> NeuralNetwork {
        let layers: Vec<Layer> = vec![
            Layer::with_size(1608, 1608, false),
            Layer::with_size(1608, 1024, false),
            Layer::with_size(1024, 512, false),
            Layer::with_size(512, 400, true),
        ];
        NeuralNetwork { layers }
    }

    pub fn load_weights(&mut self, weights_file: &str) {
        println!("loading weights from {}...", weights_file);
        let file = File::open(weights_file);
        let mut bytes = Vec::new();
        match file {
            Ok(mut file) => file.read_to_end(&mut bytes).unwrap(),
            Err(error) => {
                println!("Unable to load weights file {}", error);
                return;
            }
        };
        println!("bytes: {}", bytes.len());
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
        println!("parameters: {}", byte_index / 4);
        if bytes.len() != byte_index {
            println!("WARNING: The length of the weights file does not match the number of network parameters.");
        }
        println!("Network parameters have been loaded.");
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

    pub fn choose_action(&self, state: &GameState) -> (Action, f32) {
        let mut input_vector = state_to_vector(&state);
        let output = self.feed_forward(&mut input_vector);
        let mut action_list = ActionList::default();
        state.get_possible_actions(&mut action_list);

        let mut highest_confidence: f32 = 0.;
        let mut best_action: usize = 0;
        for index in 0..action_list.size {
            let mut confidence: f32 = 0.;
            if let Action::Set(to, _, shape_index) = action_list[index] {
                let mut action_board = Bitboard::with_piece(to, shape_index);
                while action_board.not_zero() {
                    let bit_index = action_board.trailing_zeros();
                    action_board.flip_bit(bit_index);
                    let x = bit_index % 21;
                    let y = (bit_index - x) / 21;
                    confidence += output[(x + y * 20) as usize];
                }
            }
            if confidence > highest_confidence {
                highest_confidence = confidence;
                best_action = index;
            }
        }
        (action_list[best_action], highest_confidence)
    }
}

fn state_to_vector(state: &GameState) -> Vec<f32> {
    let mut vector: Vec<f32> = vec![0.; 1608];
    let mut index: usize = 0;
    let mut color = state.current_player;
    for _ in 0..4 {
        for i in 0..440 {
            if VALID_FIELDS.check_bit(i) {
                if state.board[color as usize].check_bit(i) {
                    vector[index] = 1.;
                }
                index += 1;
            }
        }
        color = color.next();
    }
    let mut bit: u8 = 1;
    for entry in vector.iter_mut().take(1909).skip(1601) {
        if state.ply & bit == bit {
            *entry = 1.;
        }
        bit <<= 1;
    }
    vector
}
