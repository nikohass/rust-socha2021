pub mod action;
pub mod actionlist;
pub mod bitboard;
pub mod constants;
pub mod gamestate;
pub mod hashing;
pub mod piece_type;

pub use action::Action;
pub use actionlist::{ActionList, ActionListStack};
pub use bitboard::Bitboard;
pub use constants::*;
pub use gamestate::GameState;
pub use hashing::*;
pub use piece_type::{PieceType, PIECE_TYPES, START_PIECE_TYPES};

pub trait Player {
    fn on_move_request(&mut self, state: &GameState) -> Action;

    fn on_reset(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::{Action, ActionList, ActionListStack, Bitboard, GameState};
    pub const TEST_FENS: [&str; 10] = [
        "8 25769807872 996921150343251776065009382701662208 0 0 0 0 0 2287396291528267661315 0 0 0 950739007981840676599504568320 98304 10633839178386343325247702478008352768 0 0 25961484447265452962400921471401968",
        "30 25769807872 996921150343254045022605263184592960 36346084818414599765537557453996062 0 0 0 649047475072245635402777447366721 31721753070385684709394273345208323 0 0 1453845140708519202399872246643459328 19190730219111893411153728625845993472 98436 10713670744443226362423265803308892214 0 0 25961484348662411652754519073628144",
        "10 32212258816 498460617261625002680807800407327744 0 0 0 0 37779057963781087920128 5316919589048100566575940396521095168 32768 5316919589044473787964173904878239744 42535316147526911584592249876222312448 0 0 0 1 170141426849423161187183435651397713921 24663410308945225816314412397354992",
        "17 32212258816 498460617261625002680807800407328000 124615253350608212906188473242746880 0 0 0 47223797684926261854208 5316919589048100566575940396521095168 32768 5316919589044473787964173904878239772 42539534889014613328747941906419286016 0 0 0 649038035772768361448997673500673 170141426849423161187183435651397713921 24663410289300180527000730730356720",
        "25 32212258944 571152783184632903763276948209403136 124617154827037460296372118561488896 0 0 0 47223797684927201378408 5321462849418288560393594718258724864 32768 5316919589058980911634609709481459932 42560304076448752639262063891736166400 0 0 26388283260963 5841342321947831702261143397466113 170141426849423161187183435651397713921 24663410134552943990435559312777200",
        "29 32212258944 571152783184632903763276948209403136 124617154828246386692447981075955712 0 0 0 47223797684927201378408 5321462849418288781754576379339866112 32768 5316919589368466437964859210817863900 42560304076448752639262063891736166400 0 0 26388283260963 5841342331619245176708303948349441 170141426849423161187183435651397713921 24663410133948462633879772968312816",
        "4 32212258816 0 0 0 0 0 0 4835705584303175180484608 32768 5316919589044473786811251850515316736 0 0 0 0 0 9223376434907578369 24663410310154151708022919493844976",
        "10 32212258816 498460617261625002680807800407327744 0 0 0 0 37779057963781087920128 5316919589048100566575940396521095168 32768 5316919589044473787964173904878239744 42535316147526911584592249876222312448 0 0 0 1 170141426849423161187183435651397713921 24663410308945225816314412397354992",
        "20 32212258816 498460617261625002680807800407328000 124615253350608212906188473242746880 0 0 0 47223797684927201378400 5316919589048100566575940396521095168 32768 5316919589044473787964173905079566556 42560304076448752639262063891736166400 0 0 26388283260931 649038035772768361448997673500673 170141426849423161187183435651397713921 24663410134557666392956022074236912",
        "25 32212258944 571152783184632903763276948209403136 124617154827037460296372118561488896 0 0 0 47223797684927201378408 5321462849418288560393594718258724864 32768 5316919589058980911634609709481459932 42560304076448752639262063891736166400 0 0 26388283260963 5841342321947831702261143397466113 170141426849423161187183435651397713921 24663410134552943990435559312777200",
    ];

    fn count_actions(
        state: &mut GameState,
        depth: usize,
        action_list_stack: &mut ActionListStack,
    ) -> u64 {
        action_list_stack[depth].clear();
        state.get_possible_actions(&mut action_list_stack[depth]);
        if depth == 0 || state.is_game_over() {
            return action_list_stack[depth].size as u64;
        }
        let mut nodes: u64 = 0;
        for i in 0..action_list_stack[depth].size {
            state.do_action(action_list_stack[depth][i]);
            nodes += count_actions(state, depth - 1, action_list_stack);
            state.undo_action(action_list_stack[depth][i]);
        }
        nodes
    }

    #[test]
    fn test_state() {
        let mut state = GameState::from_fen("10 229376 21270254224581560681009129656623824896 0 0 17179881472 1993842152134227742961992379458912256 0 0 0 0 0 162259412228914383610878529372163 0 0 0 2658457893046336551349755980500697088 230904757757165799616708592".to_string());
        let mut action_list_stack = ActionListStack::with_size(4);
        assert_eq!(
            count_actions(&mut state, 2, &mut action_list_stack),
            46643240
        );
        let results: [u64; 10] = [
            48433824, 6248584, 29639180, 53644449, 28047938, 5378634, 10513152, 29639180, 39037106,
            28047938,
        ];
        for (i, fen) in TEST_FENS.iter().enumerate().take(2) {
            let mut state = GameState::from_fen(fen.to_string());
            assert_eq!(
                results[i],
                count_actions(&mut state, 2, &mut action_list_stack)
            );
        }
    }

    #[test]
    fn test_action_serialization() {
        let mut action_list = ActionList::default();
        for fen in TEST_FENS.iter() {
            let state = GameState::from_fen(fen.to_string());
            state.get_possible_actions(&mut action_list);
            for index in 0..action_list.size / 10 {
                let action = action_list[index];
                assert_eq!(action, Action::deserialize(action.serialize()));
            }
        }
    }

    #[test]
    fn test_action_from_bitboard() {
        let mut action_list = ActionList::default();
        for fen in TEST_FENS.iter() {
            let state = GameState::from_fen(fen.to_string());
            state.get_possible_actions(&mut action_list);
            for index in 0..action_list.size / 10 {
                if let Action::Set(to, shape_index) = action_list[index] {
                    let action_board = Bitboard::with_piece(to, shape_index);
                    assert_eq!(action_list[index], Action::from_bitboard(action_board));
                }
            }
        }
    }

    #[test]
    fn test_skipped() {
        let mut state = GameState::new();
        let mut action_list = ActionList::default();
        for _ in 0..4 {
            state.get_possible_actions(&mut action_list);
            state.do_action(action_list[0]);
        }

        for _ in 0..8 {
            state.do_action(Action::Skip);
        }
        assert_eq!(state.skipped & 0b1111, 0b1111); // all players skipped twice
        for _ in 0..4 {
            state.undo_action(Action::Skip);
        }
        assert_eq!(state.skipped & 0b1111, 0b1111); // all players skipped once
        for _ in 0..2 {
            state.undo_action(Action::Skip);
        }
        assert_eq!(state.skipped & 0b1111, 0b0011); // Only BLUE and YELLOW skipped
    }

    #[test]
    fn test_fen() {
        for i in 0..10 {
            assert_eq!(
                TEST_FENS[i],
                GameState::from_fen(TEST_FENS[i].to_string()).to_fen()
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
                for piece in pieces.iter() {
                    if let Action::Set(to, shape_index) = piece {
                        board |= Bitboard::with_piece(*to, *shape_index);
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
