use game_sdk::{Bitboard, Color, GameState, VALID_FIELDS};

pub fn evaluate(state: &GameState) -> i16 {
    let team_0_fields = state.board[0] | state.board[2];
    let team_1_fields = state.board[1] | state.board[3];
    let value = (evaluate_team(state, 0, team_0_fields, team_1_fields)
        - evaluate_team(state, 1, team_1_fields, team_0_fields)) as i16;

    match state.current_player {
        Color::RED => value,
        Color::BLUE => value,
        Color::GREEN => -value,
        Color::YELLOW => -value,
    }
}

pub fn evaluate_team(
    _state: &GameState,
    _team: usize,
    own_fields: Bitboard,
    other_fields: Bitboard,
) -> f64 {
    let placement_fields =
        own_fields.diagonal_neighbours() & VALID_FIELDS & !(other_fields | own_fields);
    (own_fields.count_ones() as f64) * 5. + (placement_fields.count_ones() as f64) * 1.
}
