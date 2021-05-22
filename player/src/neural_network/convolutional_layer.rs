use super::float_stuff::relu;
use std::fmt::{Display, Formatter, Result};

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

    pub fn load_weights(&mut self, bytes: &[u8], mut index: usize) -> usize {
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
