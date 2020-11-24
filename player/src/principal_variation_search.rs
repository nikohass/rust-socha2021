use super::cache::CacheEntry;
use super::evaluation::evaluate;
use super::search::{SearchParameters, MAX_SCORE};
use game_sdk::GameState;

pub fn principal_variation_search(
    params: &mut SearchParameters,
    state: &mut GameState,
    mut alpha: i16,
    mut beta: i16,
    current_depth: usize,
    depth_left: usize,
) -> i16 {
    params.nodes_searched += 1;
    let is_pv_node = beta > 1 + alpha;
    params.pv_table[current_depth].size = 0;
    let original_alpha = alpha;

    if params.nodes_searched % 4096 == 0 {
        params.stop = params.start_time.elapsed().as_millis() > params.time;
    }
    if depth_left == 0 || params.stop || state.is_game_over() {
        return evaluate(state);
    }
    state.get_possible_actions(&mut params.action_list_stack[depth_left]);

    let mut ordering_index: usize = 0;
    if params.principal_variation.size > current_depth {
        let pv_action = params.principal_variation[current_depth];
        for i in 0..params.action_list_stack[depth_left].size {
            if pv_action == params.action_list_stack[depth_left][i] {
                params.action_list_stack[depth_left].swap(ordering_index, i);
                ordering_index += 1;
                break;
            }
        }
    }

    let transposition_table_entry = params.transposition_table.lookup(state.hash);
    if !transposition_table_entry.is_empty()
        && transposition_table_entry.depth_left >= depth_left as u8
        && transposition_table_entry.depth == current_depth as u8
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
        for i in ordering_index..params.action_list_stack[depth_left].size {
            if tt_action == params.action_list_stack[depth_left][i] {
                params.action_list_stack[depth_left].swap(ordering_index, i);
                //ordering_index += 1;
                break;
            }
        }
    }

    let mut best_score = -MAX_SCORE;
    let mut best_action_index: usize = 0;
    for index in 0..params.action_list_stack[depth_left].size {
        let action = params.action_list_stack[depth_left][index];
        state.do_action(action);

        let score = if index == 0 {
            -principal_variation_search(
                params,
                state,
                -beta,
                -alpha,
                current_depth + 1,
                depth_left - 1,
            )
        } else {
            let mut score = -principal_variation_search(
                params,
                state,
                -alpha - 1,
                -alpha,
                current_depth + 1,
                depth_left - 1,
            );
            if score > alpha {
                score = -principal_variation_search(
                    params,
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
            params.pv_table[current_depth].size = 0;
            params.pv_table[current_depth].push(action);
            if is_pv_node {
                for i in 0..params.pv_table[current_depth + 1].size {
                    let action = params.pv_table[current_depth + 1][i];
                    params.pv_table[current_depth].push(action);
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

    params.transposition_table.insert(
        state.hash,
        CacheEntry {
            action: params.action_list_stack[depth_left][best_action_index],
            score: best_score,
            depth: current_depth as u8,
            depth_left: depth_left as u8,
            alpha: best_score <= original_alpha,
            beta: alpha >= beta,
        },
    );

    alpha
}
