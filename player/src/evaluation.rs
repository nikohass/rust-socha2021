use super::search::MATE_SCORE;
use game_sdk::{Bitboard, GameState, VALID_FIELDS};

pub struct EvaluationParameters {
    pub piece_values: [f64; 21],
    pub valuable_fields: Bitboard,
}

const DEFAULT_PARAMS: EvaluationParameters = EvaluationParameters {
    piece_values: [
        -200., -100., 3., 3., 4., 4., 4., 4., 4., 5., 5., 5., 5., 5., 5., 5., 5., 5., 5., 5., 5.,
    ],
    valuable_fields: Bitboard::from(
        34359812096,
        21932282371336421193617920309646591992,
        163557429034106733931453492116863088,
        2001631707235955373217122976418234370,
    ),
};

pub fn evaluate(state: &GameState) -> i16 {
    if state.is_game_over() {
        MATE_SCORE - state.game_result() * ((state.current_player as i16) % 2 * 2 - 1)
    } else {
        let team0_fields = state.board[0] | state.board[2];
        let team1_fields = state.board[1] | state.board[3];
        let value = (evaluate_team(state, 0, team0_fields, team1_fields, &DEFAULT_PARAMS)
            - evaluate_team(state, 1, team1_fields, team0_fields, &DEFAULT_PARAMS))
            as i16;
        value * -((state.current_player as i16) % 2 * 2 - 1)
    }
}

pub fn evaluate_team(
    state: &GameState,
    team: usize,
    own_fields: Bitboard,
    other_fields: Bitboard,
    params: &EvaluationParameters,
) -> f64 {
    let placement_fields =
        own_fields.diagonal_neighbours() & VALID_FIELDS & !(other_fields | own_fields);

    let mut score: f64 =
        (own_fields.count_ones() as f64) * 25. + (placement_fields.count_ones() as f64) * 5.;
    let opponents_neighbour_fields = other_fields.neighbours() & VALID_FIELDS & own_fields;
    score += opponents_neighbour_fields.count_ones() as f64 * 2.5;
    score += 12.5 * ((placement_fields & params.valuable_fields).count_ones()) as f64;
    for (index, piece_value) in params.piece_values.iter().enumerate() {
        if state.pieces_left[index][team] {
            score -= piece_value * 2.5;
        }
        if state.pieces_left[index][team + 2] {
            score -= piece_value * 2.5;
        }
    }
    score
}
