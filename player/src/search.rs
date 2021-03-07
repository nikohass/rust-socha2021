use super::cache::{EvaluationCache, TranspositionTable, TranspositionTableEntry};
use super::evaluation::static_evaluation;
use game_sdk::{Action, ActionList, ActionListStack, GameState, Player};
use std::time::Instant;

pub const MAX_SEARCH_DEPTH: usize = 40;
pub const MAX_SCORE: i16 = std::i16::MAX;
pub const MATE_SCORE: i16 = 32_000;
pub const TT_SIZE: usize = 20_000_000;
pub const EVAL_CACHE_SIZE: usize = 1_000_000;

pub struct Searcher {
    pub nodes_searched: u64,
    pub root_ply: u8,
    pub stop: bool,
    pub action_list_stack: ActionListStack,
    pub principal_variation: ActionList,
    pub pv_table: ActionListStack,
    pub transposition_table: TranspositionTable,
    pub evaluation_cache: EvaluationCache,
    pub start_time: Instant,
    pub time_limit: u128,
}

impl Searcher {
    pub fn new(time_limit: u128) -> Searcher {
        Searcher {
            nodes_searched: 0,
            root_ply: 0,
            stop: false,
            action_list_stack: ActionListStack::with_size(MAX_SEARCH_DEPTH),
            principal_variation: ActionList::default(),
            pv_table: ActionListStack::with_size(MAX_SEARCH_DEPTH),
            transposition_table: TranspositionTable::with_size(TT_SIZE),
            evaluation_cache: EvaluationCache::with_size(EVAL_CACHE_SIZE),
            start_time: Instant::now(),
            time_limit,
        }
    }

    pub fn search_action(&mut self, state: &GameState) -> Action {
        println!("Searching action using PV-Search. Fen: {}", state.to_fen());
        println!("Depth    Time   Score     Nodes     Nodes/s PV");
        let mut state = state.clone();
        self.nodes_searched = 0;
        self.root_ply = state.ply;
        self.start_time = Instant::now();
        self.stop = false;
        self.principal_variation.clear();

        let mut score = -MAX_SCORE;
        let mut best_action = Action::Skip;
        let mut last_principal_variation_size: usize = 0;
        for depth in 1..=MAX_SEARCH_DEPTH {
            let depth_start_time = Instant::now();
            let current_score =
                principal_variation_search(self, &mut state, -MAX_SCORE, MAX_SCORE, 0, depth);
            let time = self.start_time.elapsed().as_millis();
            print!(
                "{:5} {:5}ms {:7} {:9} {:11.1} ",
                depth,
                time,
                current_score,
                self.nodes_searched,
                (self.nodes_searched as f64) / (time as f64) * 1000.
            );
            if self.stop {
                println!("(canceled)");
                break;
            }
            score = current_score;
            self.principal_variation = self.pv_table[0].clone();
            best_action = self.principal_variation[0];

            if self.principal_variation.size == last_principal_variation_size {
                println!("\nReached the end of the search tree.");
                if score >= MATE_SCORE {
                    println!("Mate in {} (+{})", depth - 1, score - MATE_SCORE);
                } else if score == 0 {
                    println!("Draw in {}", depth - 1);
                } else {
                    println!("Mated in {} ({})", depth - 1, score + MATE_SCORE);
                }
                break;
            }
            last_principal_variation_size = self.principal_variation.size;
            println!("{}", self.principal_variation);

            if depth_start_time.elapsed().as_millis() > (self.time_limit - time) / 2 {
                break;
            }
        }
        println!(
            "Search finished after {}ms. Score: {} Nodes: {} Nodes/s: {:.3} PV: {}",
            self.start_time.elapsed().as_millis(),
            score,
            self.nodes_searched,
            self.nodes_searched as f64 / self.start_time.elapsed().as_millis() as f64 * 1000.,
            self.principal_variation,
        );
        best_action
    }

    pub fn reset(&mut self) {
        self.transposition_table = TranspositionTable::with_size(TT_SIZE);
        self.evaluation_cache = EvaluationCache::with_size(EVAL_CACHE_SIZE);
        self.nodes_searched = 0;
        self.root_ply = 0;
        self.stop = false;
    }
}

impl Player for Searcher {
    fn on_move_request(&mut self, state: &GameState) -> Action {
        self.search_action(state)
    }

    fn on_reset(&mut self) {
        self.reset();
    }
}

pub fn principal_variation_search(
    searcher: &mut Searcher,
    state: &mut GameState,
    mut alpha: i16,
    mut beta: i16,
    current_depth: usize,
    depth_left: usize,
) -> i16 {
    searcher.nodes_searched += 1;
    searcher.pv_table[current_depth].clear();
    let is_pv_node = beta > 1 + alpha;
    let original_alpha = alpha;

    if searcher.nodes_searched % 4096 == 0 {
        searcher.stop = searcher.start_time.elapsed().as_millis() >= searcher.time_limit;
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
            if state.has_team_one_skipped() && state.game_result() < 0 {
                return -MATE_SCORE;
            }
        } else if state.has_team_two_skipped() && state.game_result() > 0 {
            return -MATE_SCORE;
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

    let tt_entry = searcher.transposition_table.lookup(state.hash);
    if !tt_entry.is_empty()
        && tt_entry.depth_left >= depth_left as u8
        && tt_entry.ply == state.ply
        && tt_entry.hash == state.hash
    {
        if !is_pv_node {
            if tt_entry.alpha && tt_entry.beta {
                return tt_entry.score;
            } else if tt_entry.alpha {
                alpha = tt_entry.score;
            } else if tt_entry.beta {
                beta = tt_entry.score;
            }
        }
        let tt_action = tt_entry.action;
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
            searcher.pv_table[current_depth].clear();
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
            ply: state.ply,
            depth_left: depth_left as u8,
            alpha: best_score <= original_alpha,
            beta: alpha >= beta,
            hash: state.hash,
        },
    );

    alpha
}
