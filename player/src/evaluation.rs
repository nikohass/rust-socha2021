use super::search::MATE_SCORE;
use game_sdk::{Bitboard, GameState};

pub struct EvaluationParameters {
    valuable_fields: Bitboard,
    occupied_field_factor: f32,
    placement_field_factor: f32,
    blocked_factor: f32,
    valuable_field_factor: f32,
    proximity_factor: f32,
    double_placement_field_factor: f32,
    monomino_placed_last_factor: f32,
}

const DEFAULT_PARAMS: EvaluationParameters = EvaluationParameters {
    valuable_fields: Bitboard(
        4096,
        10966141185668210596808960154823295996,
        81778714517053366965726746058431544,
        1000815853617977686608561488208592896,
    ),
    /*
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  1  .
    .  .  1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  1  .  .
    .  .  .  1  .  .  .  .  .  .  .  .  .  .  .  .  1  .  .  .
    .  .  .  .  1  .  .  .  .  .  .  .  .  .  .  1  .  .  .  .
    .  .  .  .  .  1  1  .  .  .  .  .  .  1  1  .  .  .  .  .
    .  .  .  .  .  1  1  1  .  .  .  .  1  1  1  .  .  .  .  .
    .  .  .  .  .  .  1  1  1  1  1  1  1  1  .  .  .  .  .  .
    .  .  .  .  .  .  .  1  1  1  1  1  1  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  1  1  1  1  1  1  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  1  1  1  1  1  1  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  1  1  1  1  1  1  .  .  .  .  .  .  .
    .  .  .  .  .  .  1  1  1  1  1  1  1  1  .  .  .  .  .  .
    .  .  .  .  .  1  1  1  .  .  .  .  1  1  1  .  .  .  .  .
    .  .  .  .  .  1  1  .  .  .  .  .  .  1  1  .  .  .  .  .
    .  .  .  .  1  .  .  .  .  .  .  .  .  .  .  1  .  .  .  .
    .  .  .  1  .  .  .  .  .  .  .  .  .  .  .  .  1  .  .  .
    .  .  1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  1  .  .
    .  1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  1  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
     */
    occupied_field_factor: 25.,
    placement_field_factor: 11.,
    blocked_factor: 2.5,
    valuable_field_factor: 7.5,
    proximity_factor: 15.,
    double_placement_field_factor: -50.,
    monomino_placed_last_factor: 70.,
};

pub fn static_evaluation(state: &GameState) -> i16 {
    let team = state.get_team();
    if state.is_game_over() {
        let result = state.game_result();
        return -if result > 0 {
            MATE_SCORE + result
        } else {
            -MATE_SCORE + result
        } * team;
    }
    let one_fields = state.board[0] | state.board[2];
    let two_fields = state.board[1] | state.board[3];
    let all_occupied_fields = one_fields | two_fields;

    let field_difference = one_fields.count_ones() as f32 - two_fields.count_ones() as f32;

    let placement_field_difference;
    let double_placement_field_difference;
    {
        let blue_placement_fields = state.board[0].diagonal_neighbours()
            & !(all_occupied_fields | state.board[0].neighbours());
        let yellow_placement_fields = state.board[1].diagonal_neighbours()
            & !(all_occupied_fields | state.board[1].neighbours());
        let red_placement_fields = state.board[2].diagonal_neighbours()
            & !(all_occupied_fields | state.board[2].neighbours());
        let green_placement_fields = state.board[3].diagonal_neighbours()
            & !(all_occupied_fields | state.board[3].neighbours());

        let one_placement_fields = blue_placement_fields | red_placement_fields;
        let two_placement_fields = yellow_placement_fields | green_placement_fields;

        placement_field_difference =
            one_placement_fields.count_ones() as f32 - two_placement_fields.count_ones() as f32;

        double_placement_field_difference = (blue_placement_fields & red_placement_fields)
            .count_ones() as f32
            - (yellow_placement_fields & green_placement_fields).count_ones() as f32;
    }

    let blocked_placement_field_difference = ((state.board[0].diagonal_neighbours()
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

    let valuable_field_difference = (one_fields & DEFAULT_PARAMS.valuable_fields).count_ones()
        as f32
        - (two_fields & DEFAULT_PARAMS.valuable_fields).count_ones() as f32;

    let proximity_difference = ((all_occupied_fields & state.board[0].neighbours())
        | (all_occupied_fields & state.board[2].neighbours()))
    .count_ones() as f32
        - ((all_occupied_fields & state.board[1].neighbours())
            | (all_occupied_fields & state.board[3].neighbours()))
        .count_ones() as f32;

    let m_last = state.skipped as u8 & state.monomino_placed_last;
    let monomino_placed_last_difference =
        (m_last & 0b101).count_ones() as f32 - (m_last & 0b1010).count_ones() as f32;

    let score = field_difference * DEFAULT_PARAMS.occupied_field_factor
        + placement_field_difference * DEFAULT_PARAMS.placement_field_factor
        + blocked_placement_field_difference * DEFAULT_PARAMS.blocked_factor
        + valuable_field_difference * DEFAULT_PARAMS.valuable_field_factor
        + proximity_difference * DEFAULT_PARAMS.proximity_factor
        + double_placement_field_difference * DEFAULT_PARAMS.double_placement_field_factor
        + monomino_placed_last_difference * DEFAULT_PARAMS.monomino_placed_last_factor;

    score.round() as i16 * -team
}
