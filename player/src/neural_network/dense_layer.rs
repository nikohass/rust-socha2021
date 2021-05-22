use super::float_stuff::sigmoid;
use super::ActivationFunction;
use std::fmt::{Display, Formatter, Result};

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

    pub fn feed_forward(&self, input: &[f32]) -> Vec<f32> {
        let mut output: Vec<f32> = vec![0.; self.output_size];
        for (i, output_neuron) in output.iter_mut().enumerate() {
            for (j, input_neuron) in input.iter().enumerate() {
                *output_neuron += input_neuron * self.weights[j][i];
            }
            *output_neuron = (self.activation)(*output_neuron + self.biases[i]);
        }
        output
    }

    pub fn load_weights(&mut self, bytes: &[u8], mut index: usize) -> usize {
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
