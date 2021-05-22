pub use super::float_stuff;
pub mod cnn;
pub mod convolutional_layer;
pub mod dense_layer;
pub type ActivationFunction = fn(f32) -> f32;
