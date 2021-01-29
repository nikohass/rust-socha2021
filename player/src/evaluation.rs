use super::search::MATE_SCORE;
use game_sdk::{Bitboard, GameState};

pub struct EvaluationParameters {
    //pub piece_values: [f32; 21],
    pub valuable_fields: Bitboard,
    pub occupied_fields_factor: f32,
    pub placement_fields_factor: f32,
    pub blocked_factor: f32,
    pub valuable_fields_factor: f32,
    pub proximity_factor: f32,
}

const DEFAULT_PARAMS: EvaluationParameters = EvaluationParameters {
    valuable_fields: Bitboard::from(
        4096,
        10966141185668210596808960154823295996,
        81778714517053366965726746058431544,
        1000815853617977686608561488208592896,
    ),
    /*
    . . . . . . . . . . . . . . . . . . . .
    . 1 . . . . . . . . . . . . . . . . 1 .
    . . 1 . . . . . . . . . . . . . . 1 . .
    . . . 1 . . . . . . . . . . . . 1 . . .
    . . . . 1 . . . . . . . . . . 1 . . . .
    . . . . . 1 1 . . . . . . 1 1 . . . . .
    . . . . . 1 1 1 . . . . 1 1 1 . . . . .
    . . . . . . 1 1 1 1 1 1 1 1 . . . . . .
    . . . . . . . 1 1 1 1 1 1 . . . . . . .
    . . . . . . . 1 1 1 1 1 1 . . . . . . .
    . . . . . . . 1 1 1 1 1 1 . . . . . . .
    . . . . . . . 1 1 1 1 1 1 . . . . . . .
    . . . . . . 1 1 1 1 1 1 1 1 . . . . . .
    . . . . . 1 1 1 . . . . 1 1 1 . . . . .
    . . . . . 1 1 . . . . . . 1 1 . . . . .
    . . . . 1 . . . . . . . . . . 1 . . . .
    . . . 1 . . . . . . . . . . . . 1 . . .
    . . 1 . . . . . . . . . . . . . . 1 . .
    . 1 . . . . . . . . . . . . . . . . 1 .
    . . . . . . . . . . . . . . . . . . . .
    */
    occupied_fields_factor: 25.,
    placement_fields_factor: 11.,
    blocked_factor: 2.5,
    valuable_fields_factor: 7.5,
    proximity_factor: 15.,
};

pub fn static_evaluation(state: &GameState) -> i16 {
    let team = state.current_color.team_i16();
    if state.is_game_over() {
        return MATE_SCORE - state.game_result() * team;
    }
    let one_fields = state.board[0] | state.board[2];
    let two_fields = state.board[1] | state.board[3];
    let all_occupied_fields = one_fields | two_fields;

    let field_difference = one_fields.count_ones() as f32 - two_fields.count_ones() as f32;

    let placement_fields_difference = (((state.board[0].diagonal_neighbours()
        & !(all_occupied_fields | state.board[0].neighbours()))
        | (state.board[2].diagonal_neighbours()
            & !(all_occupied_fields | state.board[2].neighbours())))
    .count_ones()) as f32
        - ((state.board[1].diagonal_neighbours()
            & !(all_occupied_fields | state.board[1].neighbours()))
            | (state.board[3].diagonal_neighbours()
                & !(all_occupied_fields | state.board[3].neighbours())))
        .count_ones() as f32;

    let blocked_placement_fields_difference = ((state.board[0].diagonal_neighbours()
        & !(state.board[0].neighbours())
        & all_occupied_fields)
        | (state.board[2].diagonal_neighbours()
            & !(state.board[2].neighbours())
            & all_occupied_fields))
        .count_ones() as f32
        - ((state.board[1].diagonal_neighbours()
            & !(state.board[1].neighbours())
            & all_occupied_fields)
            | (state.board[3].diagonal_neighbours()
                & !(state.board[3].neighbours())
                & all_occupied_fields))
            .count_ones() as f32;

    let proximity_difference = ((all_occupied_fields & state.board[0].neighbours())
        | (all_occupied_fields & state.board[2].neighbours()))
    .count_ones() as f32
        - ((all_occupied_fields & state.board[1].neighbours())
            | (all_occupied_fields & state.board[3].neighbours()))
        .count_ones() as f32;

    let valuable_fields_difference = (one_fields & DEFAULT_PARAMS.valuable_fields).count_ones()
        as f32
        - (two_fields & DEFAULT_PARAMS.valuable_fields).count_ones() as f32;

    let score = field_difference * DEFAULT_PARAMS.occupied_fields_factor
        + placement_fields_difference * DEFAULT_PARAMS.placement_fields_factor
        + blocked_placement_fields_difference * DEFAULT_PARAMS.blocked_factor
        + valuable_fields_difference * DEFAULT_PARAMS.valuable_fields_factor
        + proximity_difference * DEFAULT_PARAMS.proximity_factor;

    score.round() as i16 * -team
}
