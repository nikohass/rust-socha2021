

# Software-Challenge 2021 Client
This is the client that participated in the [Software-Challenge Germany](https://software-challenge.de/) 2021 for the team "[Hermann-Tast-Sch.](https://www.hts-husum.de) Q2Phy". The game of the Software-Challenge 2021 is the two-player version of Blokus. It is an abstract strategy board game with perfect information that is played on a 20x20 board. The version of the game that is played in the Software-Challenge follows all the [official rules](https://service.mattel.com/instruction_sheets/BJV44-Eng.pdf) of Blokus except that the players can't choose their start piece. Instead, the start piece that all players have to use in the first round is selected randomly at the start of a game.
## Table of contents
1. [Internal representation of the game](#internal_representation)
	- [Bitboards](#bitboards)
	- [Shapes](#shapes)
	- [Moves](#moves)
	- [Move generation](#movegen)
2. [Player](#player)
	- [Minimax](#minimax)
		- [Hashing](#hashing)
		- [Transposition Table](#transposition_table)
	- [Monte Carlo tree search](#mcts)
		- [Heuristics](#heuristics)
		- [Simulation](#simulation)
	- [Neural networks](#neural_networks)
3. [Usage](#usage)
## Internal representation of the game<a name="internal_representation"></a>
### Bitboards<a name="bitboards"></a>
The bitboards need to have a size of at least 20x20 to store all fields. To simplify finding neighboring or diagonal fields, the board actually has a size of 21x20. Because there are no integers in Rust that have this many bits, each 512-bit bitboard is made up of four u128 integers. Bitboards that big are a lot slower than small bitboards, but they are definitely faster than an array-based board representation. Especially when it comes to determining all possible moves or calculating heuristics.
### Shapes<a name="shapes"></a>
There are 21 different piece types, but because most piece types can be mirrored and/or rotated, there are 91 different shapes that can be placed on the board. Shapes are stored as an integer that is used as an index to an array that contains all the shapes on a 128-bit bitboard.
### Moves<a name="moves"></a>
To avoid collisions with the move keyword in Rust, moves are called actions in the entire source code. Actions are stored as a 16-bit unsigned integer to save memory in MCTS and the transposition table. Each action stores a shape and a destination for the shape.
### Move generation<a name="movegen"></a>
The client uses two types of move generation. The main move generation function determines all possible moves and saves them in a list. To increase the speed of the random playouts in MCTS, a random move generation function is used. This function is a lot faster because it returns after finding a single legal move. In the performance test, this function can generate about 3,170,000 legal moves per second.
The move generation first determines the fields on which pieces can be placed and the corners that each piece has to touch.
```rust
// Fields that are occupied by the current color
let own_fields = self.board[color];
// All fields that are occupied by the other colors
let other_fields = self.get_occupied_fields() & !own_fields;
// Fields that newly placed pieces can occupy
let legal_fields = !(own_fields | other_fields | own_fields.neighbors()) & VALID_FIELDS;
// Calculate the corners of existing pieces at which new pieces can be placed
let p = if self.ply > 3 {
    own_fields.diagonal_neighbors() & legal_fields
} else {
    START_FIELDS & !other_fields
};
```

Then possible destinations for each shape can then be found using expressions like this:
```rust
// Finds all possible destinations for a Domino shape
(legal_fields & legal_fields >> 1) & (p | p >> 1)
```
The left part determines all destinations at which the piece can be placed without occupying a field that is already occupied or a field that is next to another piece of the same color. The right part of this expression makes sure that the shape can only be placed at the corner of an existing piece.
## Player<a name="player"></a>
The main algorithm of this client is Monte Carlo tree search with rapid action value estimation and heuristic search seeding. Minimax and convolutional neural networks were also implemented, but both play significantly worse than MCTS. Minimax plays very weakly due to the large branching factor of Blokus and my rather poor evaluation function. In general, I think convolutional neural networks for Blokus are a good idea because Blokus is very much about pattern recognition. But my implementation has to run on a single CPU core and is just too poorly optimized to even remotely keep up with MCTS.
### Minimax<a name="minimax"></a>
The minimax algorithm is a principal variation search algorithm with iterative deepening. It was surpassed by MCTS relatively quickly. That's why I stopped improving it halfway through.
#### Hashing<a name="hashing"></a>
Standard Zobrist hashing is not a good solution for Blokus. To save a key for each shape at every position for every color, it would require 91x400x4 keys. An alternative would be to store a key for fields instead of keys for shapes. This way only 400x4 keys would be needed, but it would be necessary to iterate over all fields that the shape occupies and lookup up to 5 keys to do or undo a single move. This degenerates performance by a lot. In the end, I decided to split the Zobrist key into shape/color and destination field/color keys. Blokus also has the advantage that transposition can only occur in the same turn. That's why the current turn can also be used for hashing.
I am not quite satisfied with this solution and I haven't tested the probability of key collisions, but I decided to leave it this way because the hash is only used in the transposition table for minimax, and the test results showed that it improved the playing strength of the algorithm anyways.
#### Transposition table<a name="transposition_table"></a>
The main purpose of the transposition table is actually to save information about nodes for iterative deepening and between searches. Transpositions in Blokus can only occur after a search depth of 8 plies, which is rarely reached by the minimax search.
### Monte Carlo tree search<a name="mcts"></a>
#### Heuristics<a name="heuristics"></a>
Monte Carlo tree search uses a heuristic function to pre initialize new nodes with a heuristic value. The heuristic function takes into consideration:
- The size of the piece
- The number of opponent fields that are blocked by a piece
- Whether the piece helps the color to 'leak' into areas that it couldn't reach before
- The distance of the piece to the center of the board
- The new corners that the piece would create at which new pieces can be attached

Using this heuristic, MCTS will focus on the more important parts of the tree and spend less time evaluating bad moves. The heuristic evaluation is built in a way, that as many calculations as possible are done beforehand so that estimating the value of each child node is relatively fast. The heuristics significantly reduces the search depth of MCTS, but it still improves the playing strength by a lot.
#### Simulation<a name="simulation"></a>
The MCTS algorithm uses random playouts to estimate the value of a node. After each playout, the result of the game is returned and the values of the actions are stored in the RAVE table.
### Neural networks<a name="neural_networks"></a>
I've tried a lot to make the neural network work. Different sizes and different numbers of layers and filters, different activation functions, and neural networks with only dense or only convolutional layers. I tried to use them as policy and value networks. The models were trained using TensorFlow and a [Python implementation](https://github.com/nikohass/python-socha2021) of Blokus. In a few encounters, a client played for our team that used a neural network to make decisions in the first few rounds because MCTS was not able to reliably plan far enough ahead. But after I improved MCTS further, the neural network was not needed anymore. In general, all the neural networks I trained had not enough layers, and my dataset was too small and one-sided to generalize the neural network. The best neural networks only play slightly better than the heuristic that is used in MCTS, but each feed-forward takes usually more than 100 milliseconds, which makes it inviable to use it in a 2-second tree search.
## Usage<a name="usage"></a>
To build the client, run `cargo build --release --bin client`. To play a game against it, you will need the [latest version of the Software-Challenge GUI for Blokus](https://github.com/CAU-Kiel-Tech-Inf/gui/releases/tag/21.4.0). For a performance test run `cargo run --bin perft --release` and to run the unit tests, use `cargo test --release`. To compile a client that uses a different algorithm than MCTS you only need to edit the imports in `client/src/main.rs`.
```rust
//use player::simple_client::SimpleClient as Algorithm; // Random player
//use player::mcts::heuristics::HeuristicPlayer as Algorithm; // MCTS heuristics
//use player::neural_network::cnn::NeuralNetwork as Algorithm; // Convolutional neural network
//use player::minimax::search::Searcher as Algorithm; // Minimax
use player::mcts::search::Mcts as Algorithm; // MCTS
```
## Inspired by<a name="inspiredby"></a>
 - https://github.com/imkgerC/rust-socha2020
 - https://github.com/enz/pentobi
