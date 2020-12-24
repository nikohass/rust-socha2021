use super::cache::TranspositionTableEntry;
use super::evaluation::static_evaluation;
use super::search::{Searcher, MATE_SCORE, MAX_SCORE};
use game_sdk::GameState;

pub fn principal_variation_search(
    searcher: &mut Searcher,
    state: &mut GameState,
    mut alpha: i16,
    mut beta: i16,
    current_depth: usize,
    depth_left: usize,
) -> i16 {
    searcher.nodes_searched += 1;
    searcher.pv_table[current_depth].size = 0;
    let is_pv_node = beta > 1 + alpha;
    let original_alpha = alpha;

    if searcher.nodes_searched % 4096 == 0 {
        searcher.stop = searcher.start_time.elapsed().as_millis() > searcher.time_limit;
    }

    if depth_left == 0 || searcher.stop || state.is_game_over() {
        let evaluation_cache_entry = searcher.evaluation_cache.lookup(state.hash);
        if evaluation_cache_entry.hash == state.hash
            && evaluation_cache_entry.score != std::i16::MIN
        {
            return evaluation_cache_entry.score;
        } else {
            let score = static_evaluation(state);
            searcher.evaluation_cache.insert(state.hash, score);
            return score;
        }
    }

    if !is_pv_node && current_depth > 2 {
        if state.ply & 0b1 == 0 {
            if state.skipped & 0b101 == 0b101 && state.game_result() < 0 {
                return MATE_SCORE; // team one will lose
            }
        } else if state.skipped & 0b1010 == 0b1010 && state.game_result() > 0 {
            return MATE_SCORE; // team two will lose
        }
    }

    state.get_possible_actions(&mut searcher.action_list_stack[depth_left]);

    let mut ordering_index: usize = 0;
    if searcher.principal_variation.size > current_depth {
        let pv_action = searcher.principal_variation[current_depth];
        for i in 0..searcher.action_list_stack[depth_left].size {
            if pv_action == searcher.action_list_stack[depth_left][i] {
                searcher.action_list_stack[depth_left].swap(ordering_index, i);
                ordering_index += 1;
                break;
            }
        }
    }

    let transposition_table_entry = searcher.transposition_table.lookup(state.hash);
    if !transposition_table_entry.is_empty()
        && transposition_table_entry.depth_left >= depth_left as u8
        && transposition_table_entry.depth == current_depth as u8
        && transposition_table_entry.hash == state.hash
    {
        if !is_pv_node {
            if transposition_table_entry.alpha && transposition_table_entry.beta {
                return transposition_table_entry.score;
            } else if transposition_table_entry.alpha {
                alpha = transposition_table_entry.score;
            } else if transposition_table_entry.beta {
                beta = transposition_table_entry.score;
            }
        }
        let tt_action = transposition_table_entry.action;
        for i in ordering_index..searcher.action_list_stack[depth_left].size {
            if tt_action == searcher.action_list_stack[depth_left][i] {
                searcher.action_list_stack[depth_left].swap(ordering_index, i);
                //ordering_index += 1;
                break;
            }
        }
    }

    let mut best_score = -MAX_SCORE;
    let mut best_action_index: usize = 0;
    for index in 0..searcher.action_list_stack[depth_left].size {
        let action = searcher.action_list_stack[depth_left][index];
        state.do_action(action);

        let score = if index == 0 {
            -principal_variation_search(
                searcher,
                state,
                -beta,
                -alpha,
                current_depth + 1,
                depth_left - 1,
            )
        } else {
            let mut score = -principal_variation_search(
                searcher,
                state,
                -alpha - 1,
                -alpha,
                current_depth + 1,
                depth_left - 1,
            );
            if score > alpha {
                score = -principal_variation_search(
                    searcher,
                    state,
                    -beta,
                    -alpha,
                    current_depth + 1,
                    depth_left - 1,
                );
            }
            score
        };
        state.undo_action(action);

        if score > best_score {
            best_action_index = index;
            best_score = score;
            searcher.pv_table[current_depth].size = 0;
            searcher.pv_table[current_depth].push(action);
            if is_pv_node {
                for i in 0..searcher.pv_table[current_depth + 1].size {
                    let action = searcher.pv_table[current_depth + 1][i];
                    searcher.pv_table[current_depth].push(action);
                }
            }
            if score > alpha {
                alpha = score;
            }
        }
        if alpha >= beta {
            break;
        }
    }

    searcher.transposition_table.insert(
        state.hash,
        TranspositionTableEntry {
            action: searcher.action_list_stack[depth_left][best_action_index],
            score: best_score,
            depth: current_depth as u8,
            depth_left: depth_left as u8,
            alpha: best_score <= original_alpha,
            beta: alpha >= beta,
            hash: state.hash,
        },
    );

    alpha
}
