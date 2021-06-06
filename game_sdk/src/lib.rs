pub mod action;
pub mod actionlist;
pub mod bitboard;
pub mod gamestate;
pub mod hashing;
pub mod piece_type;

pub use action::Action;
pub use actionlist::{ActionList, ActionListStack};
pub use bitboard::{Bitboard, START_FIELDS, VALID_FIELDS};
pub use gamestate::GameState;
pub use piece_type::{PieceType, PIECE_TYPES, START_PIECE_TYPES};

pub trait Player {
    fn on_move_request(&mut self, state: &GameState) -> Action;

    fn on_reset(&mut self) {}

    fn set_time_limit(&mut self, _time: u128) {}
}

#[cfg(test)]
mod tests {
    use super::{Action, ActionList, Bitboard, GameState};
    pub const TEST_FENS: [&str; 4] = [
        "9488 1813758321899637372028928 98304 31901482040045200628318736031602966529 162259508943118303423338611999184 10384593717069655257060992658440192 0 0 14680065 170141507979487117894522954291043368963 17179881472 996921076066887197892070253015345152 1952305837197645587728919239017365504 0 0 0 68719509504 9304611499219250726980198399157469184",
        "14096 6654190920398850590723072 98304 31901482040045200628318736031602966529 20282409835765575363979011887727056 93461620752214586704661989910642688 0 0 42535316147536582995760855127085285377 170141507984438882183735147901579427843 17179881472 996921076067189429491089201464125440 1952305854528819124263596185110970368 0 0 0 73014483968 9470764998692365211093174290282477568",
        "17168 6732109985381697757862914 884736 31901482040045200655988913714818449409 20282409835765575363979011887727056 93461620752214586704661989910642688 0 0 42535316147536582995760855127085285377 170141548549277432327859950371488137219 17179881472 996921076067190019787743985368344704 1952305854528819124263596185110970368 0 0 0 2535303278298107582477523524608 9470764998692365211093174290282477568",
        "18194 6732109985390493852982274 884736 31901482040045200655988913714818449409 20282409835765575363979011887727056 93461620752214586704661989910642688 0 131072 42535316147536582995760855127085285377 170141548549277432327859950371488137219 17179881472 996921076067190019787743985469008000 1952305854528819124263596185110970368 0 0 0 2535303278298107582477523524608 9470764998692365211093174290282477568",
    ];

    #[test]
    fn test_action_serialization() {
        let mut al = ActionList::default();
        for fen in TEST_FENS.iter() {
            let state = GameState::from_fen(fen.to_string());
            state.get_possible_actions(&mut al);
            for index in 0..al.size / 10 {
                let action = al[index];
                assert_eq!(action, Action::deserialize(action.serialize()));
            }
        }
    }

    #[test]
    fn test_action_from_bitboard() {
        let mut al = ActionList::default();
        for fen in TEST_FENS.iter() {
            let state = GameState::from_fen(fen.to_string());
            state.get_possible_actions(&mut al);
            for index in 0..al.size / 10 {
                if al[index].is_set() {
                    let destination = al[index].get_destination();
                    let shape = al[index].get_shape() as usize;
                    let action = Action::from_bitboard(Bitboard::with_piece(destination, shape));
                    assert_eq!(al[index], action);
                }
            }
        }
    }

    #[test]
    fn test_skipped() {
        let mut state = GameState::default();
        let mut al = ActionList::default();
        for _ in 0..4 {
            state.get_possible_actions(&mut al);
            state.do_action(al[0]);
        }

        for _ in 0..8 {
            state.do_action(Action::SKIP);
        }
        assert_eq!(state.skipped & 0b1111, 0b1111);
        for _ in 0..4 {
            state.undo_action(Action::SKIP);
        }
        assert_eq!(state.skipped & 0b1111, 0b1111);
        for _ in 0..2 {
            state.undo_action(Action::SKIP);
        }
        assert_eq!(state.skipped & 0b1111, 0b0011);
    }

    #[test]
    fn test_fen() {
        for fen in TEST_FENS.iter() {
            assert_eq!(
                fen.to_string(),
                GameState::from_fen(fen.to_string()).to_fen()
            );
        }
    }

    #[test]
    fn test_bitboard_get_pieces() {
        for fen in TEST_FENS.iter() {
            let state = GameState::from_fen(fen.to_string());
            for color in 0..4 {
                let pieces = state.board[color].get_pieces();
                let mut board = Bitboard::empty();
                for action in pieces.iter() {
                    if action.is_set() {
                        board |= Bitboard::with_piece(
                            action.get_destination(),
                            action.get_shape() as usize,
                        );
                    }
                }
                assert_eq!(board, state.board[color]);
            }
        }
    }

    #[test]
    fn test_check_integrity() {
        for fen in TEST_FENS.iter() {
            let state = GameState::from_fen(fen.to_string());
            assert!(state.check_integrity());
        }
    }
}
