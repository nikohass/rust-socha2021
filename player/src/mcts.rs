use super::float_stuff::{ln, sqrt};
use super::search::format_principal_variation;
use game_sdk::{Action, ActionList, GameState, Player};
use rand::{rngs::SmallRng, SeedableRng};
use std::time::Instant;

const C: f32 = 0.0;
const C_BASE: f32 = 7000.;
const C_FACTOR: f32 = 35.5;
const VISITS_BEFORE_EXPANSION: usize = 30;

pub fn rollout(state: &GameState, rng: &mut SmallRng) -> f32 {
    let mut result = 0;
    let mut state = state.clone();
    while !state.is_game_over() {
        let random_action = state.get_random_possible_action(rng, state.ply < 16, 30);
        state.do_action(random_action);
        result = state.game_result();
        if (state.skipped & 0b101 == 0b101 && result < 0)
            || (state.skipped & 0b1010 == 0b1010 && result > 0)
        {
            break;
        }
    }
    match result.cmp(&0) {
        std::cmp::Ordering::Greater => 1.,
        std::cmp::Ordering::Less => 0.,
        std::cmp::Ordering::Equal => 0.5,
    }
}

pub struct Node {
    children: Vec<Node>,
    action: Action,
    n: f32,
    q: f32,
}

impl Node {
    pub fn empty() -> Self {
        Self {
            children: Vec::new(),
            action: Action::Skip,
            n: 0.,
            q: 0.,
        }
    }

    pub fn get_value(&self) -> f32 {
        if self.n > 0. {
            self.q / self.n
        } else {
            std::f32::NEG_INFINITY
        }
    }

    pub fn best_child(&self) -> &Node {
        let mut best_child: usize = 0;
        let mut best_value = std::f32::NEG_INFINITY;
        for (i, child_node) in self.children.iter().enumerate() {
            let child_value = child_node.get_value();
            if child_value > best_value {
                best_value = child_value;
                best_child = i;
            }
        }
        &self.children[best_child]
    }

    pub fn best_action(&self) -> Action {
        if self.children.is_empty() {
            Action::Skip
        } else {
            self.best_child().action
        }
    }

    fn get_uct_value(&self, parent_n: f32, c: f32) -> f32 {
        if self.n > 0. {
            (self.q / self.n) + c * sqrt(ln(parent_n) / self.n)
        } else {
            std::f32::INFINITY
        }
    }

    fn child_with_max_uct_value(&mut self) -> &mut Node {
        let c_adjusted = C + C_FACTOR * ln((1. + self.n + C_BASE) / C_BASE);
        let mut best_child = 0;
        let mut best_value = std::f32::NEG_INFINITY;
        for (i, child) in self.children.iter().enumerate() {
            let value = child.get_uct_value(self.n, c_adjusted);
            if value > best_value {
                best_value = value;
                best_child = i;
            }
        }
        &mut self.children[best_child]
    }

    fn backpropagate(&mut self, q: f32) {
        self.n += 1.;
        self.q += q;
    }

    fn expand(&mut self, state: &mut GameState, action_list: &mut ActionList) {
        state.get_possible_actions(action_list);
        self.children = Vec::with_capacity(action_list.size);
        for i in 0..action_list.size {
            self.children.push(Node {
                children: Vec::new(),
                action: action_list[i],
                n: 0.,
                q: 0.,
            });
        }
    }

    pub fn iteration(
        &mut self,
        action_list: &mut ActionList,
        state: &mut GameState,
        rng: &mut SmallRng,
    ) -> f32 {
        let delta;
        if self.children.is_empty() {
            if !state.is_game_over() {
                if self.n as usize % VISITS_BEFORE_EXPANSION == 1 {
                    self.expand(state, action_list);
                }
                delta = -rollout(&state, rng) * state.current_color.team_f32();
            } else if self.n == 0. {
                let result = state.game_result() * state.current_color.team_i16();
                self.q = match result {
                    r if r > 0 => 1.,
                    r if r < 0 => 0.,
                    _ => 0.5,
                };
                self.n = 1.;
                delta = self.q / self.n;
            } else {
                delta = self.q / self.n;
            }
            self.backpropagate(delta);
            return 1. - delta;
        }
        let next_child = self.child_with_max_uct_value();
        state.do_action(next_child.action);
        delta = next_child.iteration(action_list, state, rng);
        self.backpropagate(delta);
        1. - delta
    }

    pub fn principal_variation(&self, state: &mut GameState, action_list: &mut ActionList) {
        if self.children.is_empty() {
            return;
        }
        let child = self.best_child();
        if child.n == 0. {
            return;
        }
        let action = child.action;
        action_list.push(action);
        state.do_action(action);
        child.principal_variation(state, action_list);
    }
}

pub struct MCTS {
    root_node: Node,
    root_state: GameState,
    time_limit: i64,
}

impl MCTS {
    pub fn new(time_limit: u128) -> Self {
        Self {
            root_node: Node::empty(),
            root_state: GameState::new(),
            time_limit: time_limit as i64,
        }
    }

    fn set_root(&mut self, state: &GameState) {
        self.root_state = state.clone();
        self.root_node = Node::empty();
    }

    fn search_nodes(&mut self, n: usize, rng: &mut SmallRng) {
        let mut action_list = ActionList::default();
        for _ in 0..n {
            self.root_node
                .iteration(&mut action_list, &mut self.root_state.clone(), rng);
        }
    }

    fn print_stats(&self, principal_variation: &mut ActionList, time_left: i64) {
        if self.root_node.children.is_empty() {
            println!("    Time  Value PV");
            return;
        }
        principal_variation.clear();
        self.root_node
            .principal_variation(&mut self.root_state.clone(), principal_variation);

        println!(
            "{:6}ms {:6.2} {}",
            time_left,
            self.root_node.get_value(),
            format_principal_variation(&principal_variation)
        );
    }

    pub fn search_action(&mut self, state: &GameState) -> Action {
        println!("Searching action using MCTS");
        let start_time = Instant::now();
        self.set_root(&state);
        let mut rng = SmallRng::from_entropy();
        let mut principal_variation = ActionList::default();
        let mut iterations_per_ms = 0.1;
        let mut searched: usize = 0;
        let mut action_list = ActionList::default();
        self.root_state.get_possible_actions(&mut action_list);
        if action_list[0] == Action::Skip {
            return Action::Skip;
        }

        loop {
            let time_left = self.time_limit - start_time.elapsed().as_millis() as i64;
            self.print_stats(&mut principal_variation, time_left);
            if time_left < 80 {
                break;
            }
            let to_search = ((time_left as f64 / 2.) * iterations_per_ms)
                .max(1.)
                .min(500_000.) as usize;
            self.search_nodes(to_search, &mut rng);
            searched += to_search;
            iterations_per_ms = searched as f64 / start_time.elapsed().as_millis() as f64;
        }
        self.print_stats(&mut principal_variation, 0);
        let action = self.root_node.best_action();
        for i in 0..action_list.size {
            if action_list[i] == action {
                return action;
            }
        }
        println!("{}", format_principal_variation(&action_list));
        println!("{}", action.visualize());
        panic!("MCTS selected {}. This actions is invalid.", action);
    }
}

impl Player for MCTS {
    fn on_move_request(&mut self, state: &GameState) -> Action {
        self.search_action(state)
    }
}

/*
const INVALID: u32 = std::u32::MAX;
const C: f32 = 1.5;
const C_BASE: f32 = 7000.;
const C_FACTOR: f32 = 0.0;

#[derive(Clone)]
pub struct Node {
    pub parent_index: u32,      // 4
    pub first_child_index: u32, // 8
    pub sibling_index: u32,     // 12
    pub n: f32,                 // 16
    pub q: f32,                 // 20
    pub action: Action,         // 36
}

impl Node {
    pub fn empty() -> Node {
        Node {
            parent_index: INVALID,
            first_child_index: INVALID,
            sibling_index: INVALID,
            n: 0.,
            q: 0.,
            action: Action::Skip,
        }
    }

    pub fn iteration(
        &mut self,
        state: &mut GameState,
        action_list: &mut ActionList,
        rng: &mut SmallRng,
        searcher: &mut MCTS,
        my_index: u32,
    ) -> f32 {
        let delta;
        let c_adjusted = C + C_FACTOR * ((1. + self.n + C_BASE) / C_BASE).ln();
        if self.first_child_index == INVALID {
            if !state.is_game_over() {
                state.get_possible_actions(action_list);
                let mut last_added_index = 0;
                for i in 0..action_list.size {
                    searcher.add_node(Node {
                        parent_index: my_index,
                        first_child_index: INVALID,
                        sibling_index: INVALID,
                        n: 0.,
                        q: 0.,
                        action: action_list[i],
                    });
                    if i == 0 {
                        self.first_child_index = searcher.write_index as u32;
                    } else if i != action_list.size - 1 {
                        searcher.nodes[last_added_index].sibling_index =
                            searcher.write_index as u32;
                    }
                    last_added_index = searcher.write_index;
                }
                delta = rollout(&state, rng) * ((state.current_color as i16 & 0b1) * 2 - 1) as f32;
            } else if self.n == 0. {
                let result = -state.game_result() * ((state.current_color as i16 & 0b1) * 2 - 1);
                self.q = match result {
                    r if r > 0 => 1.,
                    r if r < 0 => 0.,
                    _ => 0.5,
                };
                self.n = 1.;
                delta = self.q / self.n;
            } else {
                delta = self.q / self.n;
            }
            self.backpropagate(delta);
            return 1. - delta;
        }
        let mut best_child = 0;
        let mut best_value = std::f32::NEG_INFINITY;
        let mut child_index = self.first_child_index;
        loop {
            let child = &searcher.nodes[child_index as usize];
            let value = child.get_uct_value(self.n, c_adjusted);
            if value >= best_value {
                best_value = value;
                best_child = child_index;
            }
            child_index = child.sibling_index;
            if child_index == INVALID {
                break;
            }
        }
        let mut child = searcher.nodes[best_child as usize].clone();

        state.do_action(child.action);
        if state.is_game_over() {
            return self.get_value();
        }
        delta = child.iteration(state, action_list, rng, searcher, best_child);
        searcher.nodes[best_child as usize] = child;
        self.backpropagate(delta);
        1. - delta
    }

    pub fn backpropagate(&mut self, q: f32) {
        self.q += q;
        self.n += 1.;
    }

    pub fn get_value(&self) -> f32 {
        if self.n > 0. {
            self.q / self.n
        } else {
            std::f32::NEG_INFINITY
        }
    }

    pub fn get_uct_value(&self, parent_n: f32, c: f32) -> f32 {
        if self.n > 0. {
            (self.q / self.n) + c * (parent_n.ln() / self.n).sqrt()
        } else {
            std::f32::INFINITY
        }
    }

    pub fn best_action(&self, searcher: &MCTS) -> (f32, Action) {
        if self.first_child_index == INVALID {
            panic!("No action in terminal node");
        }

        let mut best_child = 0;
        let mut best_value = std::f32::NEG_INFINITY;
        let mut child_index = self.first_child_index;
        loop {
            let child = &searcher.nodes[child_index as usize];
            let value = child.get_value();
            if value > best_value {
                best_child = child_index;
                best_value = value;
            }
            child_index = child.sibling_index;
            if child_index == INVALID {
                break;
            }
        }
        (best_value, searcher.nodes[best_child as usize].action)
    }

    pub fn best_child(&self, searcher: &MCTS) -> u32 {
        if self.first_child_index == INVALID {
            return INVALID;
        }

        let mut best_child = 0;
        let mut best_value = std::f32::NEG_INFINITY;
        let mut child_index = self.first_child_index;
        loop {
            let child = &searcher.nodes[child_index as usize];
            let value = child.get_value();
            if value > best_value {
                best_child = child_index;
                best_value = value;
            }
            child_index = child.sibling_index;
            if child_index == INVALID {
                break;
            }
        }
        best_child
    }
}

pub struct MCTS {
    pub iterations_per_ms: f64,
    pub nodes: Vec<Node>,
    pub size: usize,
    pub write_index: usize,
    pub time: u64,
    pub initial_state: GameState,
}

impl MCTS {
    pub fn new(time: u128, size_in_mb: usize) -> MCTS {
        let size = size_in_mb * std::mem::size_of::<Node>() * 1024;
        MCTS {
            iterations_per_ms: 1.,
            nodes: vec![Node::empty(); size],
            size: size,
            write_index: 1,
            time: time as u64,
            initial_state: GameState::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        if self.size - self.write_index < 10 {
            return;
        }
        self.nodes[self.write_index] = node;
        self.write_index += 1;
    }

    pub fn search_nodes(&mut self, state: &GameState, n: usize, rng: &mut SmallRng) {
        let mut action_list = ActionList::default();
        for _ in 0..n {
            let mut node = self.nodes[0].clone();
            node.iteration(&mut state.clone(), &mut action_list, rng, self, 0);
            self.nodes[0] = node;
        }
    }

    pub fn build_principal_variation(&mut self, principal_variation: &mut ActionList) {
        principal_variation.clear();
        let mut current_node = 0;
        loop {
            let best_child = self.nodes[current_node].best_child(&self);
            if best_child == INVALID || principal_variation.size > 20 {
                break;
            }
            principal_variation.push(self.nodes[best_child as usize].action);
            if best_child == 0 {
                break;
            }
            current_node = best_child as usize;
        }
    }

    pub fn search_action(&mut self, state: &GameState) -> Action {
        let start_time = Instant::now();
        println!("Searching action using MCTS");
        let mut rng = SmallRng::from_entropy();
        self.initial_state = state.clone();
        self.nodes[0] = Node::empty();
        self.write_index = 0;
        self.iterations_per_ms = 1.;

        let mut principal_variation = ActionList::default();
        let mut action_list = ActionList::default();
        state.get_possible_actions(&mut action_list);
        if action_list[0] == Action::Skip {
            return Action::Skip;
        }

        let mut samples = 0;
        let mut memory: f64 = 0.;
        let mut last_printed: f64 = std::f64::INFINITY;
        loop {
            let time_left = self.time as f64 - start_time.elapsed().as_millis() as f64;
            if time_left < 20. {
                break;
            }
            let iterations = ((time_left / 2.) * self.iterations_per_ms)
                .max(1.)
                .min(if memory < 50. { 10_000. } else { 1500. })
                as usize;
            samples += iterations;
            self.search_nodes(state, iterations, &mut rng);
            self.iterations_per_ms = samples as f64 / start_time.elapsed().as_millis() as f64;
            self.build_principal_variation(&mut principal_variation);
            memory = ((self.write_index as f64) / self.size as f64) * 100.;
            if memory > 90. {
                println!("Not enough memory to continue searching.");
                break;
            }
            if last_printed - time_left > (self.time as f64) / 15. {
                println!(
                    "{:6.2}% {} {} {}",
                    memory,
                    samples,
                    iterations,
                    format_principal_variation(&principal_variation)
                );
                last_printed = time_left;
            }
        }

        self.build_principal_variation(&mut principal_variation);
        println!("{}", format_principal_variation(&principal_variation));
        let action = principal_variation[0];
        println!("Nodes: {} Action: {}", self.write_index, action);
        for i in 0..action_list.size {
            if action == action_list[i] {
                return action;
            }
        }
        println!("Invalid action");
        action_list[0]
    }

    pub fn reset(&self) {}
}
*/
