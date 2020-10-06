use super::cache::CacheEntry;
use super::evaluation::evaluate;
use super::search::{SearchParams, MATE_SCORE};
use game_sdk::action::Action;
use game_sdk::gamestate::GameState;

pub fn principal_variation_search(
    params: &mut SearchParams,
    state: &mut GameState,
    mut alpha: i16,
    mut beta: i16,
    depth: usize,
) -> i16 {
    params.nodes_searched += 1;
    let root = state.ply == params.root_ply;
    let original_alpha = alpha;

    if depth == 0 || params.stop || state.is_game_over() {
        return evaluate(state);
    }
    params.stop = params.start_time.elapsed().as_millis() > 900;

    let mut tt_action: Option<Action> = None;
    {
        let tt_entry = params.transposition_table.lookup(state.hash);
        if let Some(tt_entry) = tt_entry {
            if tt_entry.depth >= depth as u8 {
                if tt_entry.alpha {
                    alpha = i16::max(alpha, tt_entry.score);
                } else if tt_entry.beta {
                    beta = i16::min(beta, tt_entry.score);
                } // else {
                  //    return tt_entry.score;
                  //}
            }
            tt_action = Some(tt_entry.action);
        }
    }

    params.action_list_stack[depth].size = 0;
    state.get_possible_actions(&mut params.action_list_stack[depth]);
    if let Some(tt_action) = tt_action {
        for i in 0..params.action_list_stack[depth].size {
            if tt_action == params.action_list_stack[depth][i] {
                params.action_list_stack[depth].swap(0, i);
                break;
            }
        }
    }
    let mut best_score = -MATE_SCORE;
    let mut best_action: usize = 0;

    for index in 0..params.action_list_stack[depth].size {
        let action = params.action_list_stack[depth][index];
        state.do_action(action);
        let score = if index == 0 {
            -principal_variation_search(params, state, -beta, -alpha, depth - 1)
        } else {
            let mut score =
                -principal_variation_search(params, state, -alpha - 1, -alpha, depth - 1);
            if score > alpha {
                score = -principal_variation_search(params, state, -beta, -alpha, depth - 1);
            }
            score
        };
        state.undo_action(action);

        if score > best_score {
            best_action = index;
            best_score = score;
            if root && score > params.best_score {
                params.best_action = action;
                params.best_score = score;
            }
            if score > alpha {
                alpha = score;
            }
        }
        if alpha >= beta {
            break;
        }
    }

    let entry = CacheEntry {
        score: best_score,
        action: params.action_list_stack[depth][best_action],
        depth: depth as u8,
        alpha: best_score <= original_alpha,
        beta: alpha >= beta,
    };
    params.transposition_table.insert(state.hash, entry);

    alpha
}
