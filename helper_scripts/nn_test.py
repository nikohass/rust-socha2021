import numpy as np
import tensorflow as tf
from tensorflow.keras.models import Sequential
from tensorflow.keras.layers import Dense

def relu(x):
    return np.maximum(0, x)

def sigmoid(x):
    return 1 / (1 + np.exp(-x))

class NpNeuralNetwork:
    def __init__(self, weights):
        self.layers = [NpLayer(weights[i * 2], weights[i * 2 + 1]) for i in range(len(weights) // 2)]

    @staticmethod
    def from_tf_model(model):
        nn = NpNeuralNetwork(model.get_weights())
        for layer_index, layer in enumerate(nn.layers):
            activation_function_name = model.get_layer(index=layer_index).activation.__name__
            if activation_function_name == "relu":
                layer.activation_function = relu
            elif activation_function_name == "sigmoid":
                layer.activation_function = sigmoid
            else:
                raise "Unknown activation function"
        return nn

    def feed_forward(self, value):
        for layer in self.layers:
            value = layer.feed_forward(value)
        return value

    def __str__(self):
        st = ""
        for layer in self.layers:
            st += str(layer)
        return st

    def __repr__(self):
        return str(self)

class NpLayer:
    def __init__(self, weights, biases, activation_function=relu):
        self.weights = weights
        self.biases = biases
        self.activation_function = activation_function

    def feed_forward(self, inp):
        return self.activation_function(np.dot(inp, self.weights) + self.biases)

    def __str__(self):
        return f"{self.activation_function.__name__} {self.weights.shape}\n"

    def __repr__(self):
        return str(self)

class TfNeuralNetwork:
    def __init__(self):
        self.model = Sequential(
            [
                Dense(10, activation="relu"),
                Dense(500, activation="relu"),
                Dense(10, activation="relu"),
                Dense(10, activation="relu"),
                Dense(2, activation="sigmoid"),
            ]
        )
        self.model.compile(
            loss='mean_squared_error',
            optimizer=tf.keras.optimizers.Adam(1e-4),
            metrics=['accuracy'],
        )
        self.model.build((1, 10))

    def feed_forward(self, x):
        return self.model.predict(np.array([x]))[0]

if __name__ == "__main__":
    test_X = np.random.random(100).reshape(-1, 10)

    tf_nn = TfNeuralNetwork()
    np_nn = NpNeuralNetwork.from_tf_model(tf_nn.model)

    print(tf_nn.model.summary())
    print(np_nn)

    for i, x in enumerate(test_X):
        tf_prediction = tf_nn.feed_forward(x)
        np_prediction = np_nn.feed_forward(x)
        print(np.sum(tf_prediction - np_prediction))
