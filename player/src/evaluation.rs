use super::search::MATE_SCORE;
use game_sdk::{Bitboard, GameState};

pub struct EvaluationParameters {
    pub piece_values: [f64; 21],
    pub valuable_fields: Bitboard,
    pub own_fields_factor: f64,
    pub placement_fields_factor: f64,
    pub opponents_neighbour_fields_factor: f64,
    pub valuable_fields_factor: f64,
    pub proximity_factor: f64,
    pub current_player_factor: [f64; 4],
}

impl EvaluationParameters {
    pub const fn new(
        array: [f64; 9],
        piece_values: [f64; 21],
        valuable_fields: Bitboard,
    ) -> EvaluationParameters {
        EvaluationParameters {
            piece_values,
            valuable_fields,
            own_fields_factor: array[0],
            placement_fields_factor: array[1],
            opponents_neighbour_fields_factor: array[2],
            valuable_fields_factor: array[3],
            proximity_factor: array[4],
            current_player_factor: [array[5], array[6], array[7], array[8]],
        }
    }
}

const DEFAULT_PARAMS: EvaluationParameters = EvaluationParameters::new(
    [25., 5., 2.5, 12.5, 30., -6., -4., -3., 0.],
    [
        -500., -250., 7.5, 7.5, 10., 10., 10., 10., 10., 12.5, 12.5, 12.5, 12.5, 12.5, 12.5, 12.5,
        12.5, 12.5, 12.5, 12.5, 12.5,
    ],
    Bitboard::from(
        17179906048,
        10966141185668210596808960154823295996,
        81778714517053366965726746058431544,
        1000815853617977686608561488209117185,
    ),
);

pub fn evaluate(state: &GameState) -> i16 {
    if state.is_game_over() {
        MATE_SCORE - state.game_result() * ((state.current_player as i16) % 2 * 2 - 1)
    } else {
        let team0_fields = state.board[0] | state.board[2];
        let team1_fields = state.board[1] | state.board[3];
        let value = (evaluate_team(state, 0, team0_fields, team1_fields, &DEFAULT_PARAMS)
            - evaluate_team(state, 1, team1_fields, team0_fields, &DEFAULT_PARAMS)
            + DEFAULT_PARAMS.current_player_factor[state.current_player as usize])
            as i16;
        value * -((state.current_player as i16) % 2 * 2 - 1)
    }
}

pub fn evaluate_with_params(state: &GameState, params: EvaluationParameters) -> i16 {
    if state.is_game_over() {
        MATE_SCORE - state.game_result() * ((state.current_player as i16) % 2 * 2 - 1)
    } else {
        let team0_fields = state.board[0] | state.board[2];
        let team1_fields = state.board[1] | state.board[3];
        let value = (evaluate_team(state, 0, team0_fields, team1_fields, &params)
            - evaluate_team(state, 1, team1_fields, team0_fields, &params)
            + params.current_player_factor[state.current_player as usize])
            as i16;
        value * -((state.current_player as i16) % 2 * 2 - 1)
    }
}

pub fn evaluate_team(
    state: &GameState,
    color1: usize,
    own_fields: Bitboard,
    other_fields: Bitboard,
    params: &EvaluationParameters,
) -> f64 {
    let color2 = color1 + 2;
    let placement_fields = own_fields.diagonal_neighbours() & !(other_fields | own_fields);
    let opponents_neighbour_fields = other_fields.neighbours() & own_fields;

    let mut score: f64 = (own_fields.count_ones() as f64) * params.own_fields_factor
        + (placement_fields.count_ones() as f64) * params.placement_fields_factor
        + opponents_neighbour_fields.count_ones() as f64 * params.opponents_neighbour_fields_factor
        + ((placement_fields & params.valuable_fields).count_ones()) as f64
            * params.valuable_fields_factor;

    for (index, piece_value) in params.piece_values.iter().enumerate() {
        if state.pieces_left[index][color1] {
            score -= piece_value;
        }
        if state.pieces_left[index][color2] {
            score -= piece_value;
        }
    }

    let all_occupied_fields = own_fields | other_fields;
    score += (((all_occupied_fields & state.board[color1].neighbours())
        | (all_occupied_fields & state.board[color2].neighbours()))
    .count_ones() as f64)
        * params.proximity_factor;

    score
}
