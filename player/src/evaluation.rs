use game_sdk::bitboard::Bitboard;
use game_sdk::constants::VALID_FIELDS;
use game_sdk::gamestate::GameState;

pub fn evaluate(state: &GameState) -> i16 {
    let team_0_fields = state.board[0] | state.board[1];
    let team_1_fields = state.board[2] | state.board[3];
    let value = (evaluate_team(state, 0, team_0_fields, team_1_fields)
        - evaluate_team(state, 1, team_1_fields, team_0_fields)) as i16;

    if state.ply % 4 < 2 {
        return value;
    } else {
        return -value;
    }
}

pub fn evaluate_team(
    _state: &GameState,
    _team: usize,
    own_fields: Bitboard,
    other_fields: Bitboard,
) -> f64 {
    let must_fields =
        own_fields.diagonal_neighbours() & VALID_FIELDS & !(other_fields | own_fields);
    (own_fields.count_ones() as f64) * 5. + (must_fields.count_ones() as f64) * 1.
}
