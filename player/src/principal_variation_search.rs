use super::evaluation::evaluate;
use super::search::{SearchParams, MATE_SCORE};
use game_sdk::GameState;

pub fn principal_variation_search(
    params: &mut SearchParams,
    state: &mut GameState,
    mut alpha: i16,
    beta: i16,
    current_depth: usize,
    depth_left: usize,
) -> i16 {
    params.nodes_searched += 1;
    let is_pv_node = beta > 1 + alpha;
    params.pv_table[current_depth].size = 0;

    params.stop = params.start_time.elapsed().as_millis() > 200;
    if depth_left == 0 || params.stop || state.is_game_over() {
        return evaluate(state);
    }
    state.get_possible_actions(&mut params.action_list_stack[depth_left]);

    {
        if params.principal_variation.size > current_depth {
            let pv_action = params.principal_variation[current_depth];
            for i in 0..params.action_list_stack[depth_left].size {
                if pv_action == params.action_list_stack[depth_left][i] {
                    params.action_list_stack[depth_left].swap(0, i);
                    break;
                }
            }
        }
    }

    let mut best_score = -MATE_SCORE;
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
    alpha
}
