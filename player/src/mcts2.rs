use super::float_stuff::{ln, sqrt};
//use super::neural_network::{state_to_vector, BoardRotation, NeuralNetwork};
use super::heuristics::Heuristic;
use super::mcts::RaveTable;
use super::playout::playout;
use game_sdk::{Action, ActionList, GameState, Player};
use rand::{rngs::SmallRng, SeedableRng};
//use std::fs::File;
//use std::io::prelude::*;
use std::time::Instant;

const C: f32 = 0.0;
const C_BASE: f32 = 200.0;
const C_FACTOR: f32 = std::f32::consts::SQRT_2;
const VISITS_BEFORE_EXPANSION: usize = 80;
const B_SQUARED: f32 = 0.7;
const FPU_R: f32 = 0.1;

pub struct Group {
    pub members: Vec<usize>,
    pub n: f32,
    pub q: f32,
}

impl Group {
    pub fn all(n: usize) -> Self {
        Self {
            members: (0..n).collect(),
            n: 0.,
            q: 0.,
        }
    }

    pub fn new() -> Self {
        Self {
            members: Vec::new(),
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

    pub fn member_with_max_uct_value(
        &self,
        parent: &Node,
        color: usize,
        rave_table: &RaveTable,
        is_root: bool,
    ) -> usize {
        let c_adjusted = C + C_FACTOR * ln((1. + parent.n + C_BASE) / C_BASE);
        let fpu_base = (parent.n - parent.q) / parent.n - FPU_R;
        let mut best_child = 0;
        let mut best_value = std::f32::NEG_INFINITY;
        for member in self.members.iter() {
            let value = parent.children[*member]
                .get_uct_value(parent.n, c_adjusted, color, rave_table, fpu_base, is_root);
            if value > best_value {
                best_value = value;
                best_child = *member;
            }
        }
        best_child
    }

    pub fn get_uct_value(&self, parent_n: f32, c: f32) -> f32 {
        if self.n > 0. {
            self.q / self.n + c * sqrt(ln(parent_n) / self.n)
        } else {
            std::f32::INFINITY
        }
    }
}

pub struct Node {
    pub children: Vec<Node>,
    pub action: Action,
    pub n: f32,
    pub q: f32,
    pub child_groups: Vec<Group>,
    pub member_of_parents_group: Vec<usize>,
}

impl Node {
    pub fn empty() -> Self {
        Self {
            children: Vec::new(),
            action: Action::SKIP,
            n: 0.,
            q: 0.,
            child_groups: Vec::new(),
            member_of_parents_group: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn get_value(&self) -> f32 {
        if self.n > 0. {
            self.q / self.n
        } else {
            std::f32::NEG_INFINITY
        }
    }

    pub fn get_uct_value(
        &self,
        parent_n: f32,
        c: f32,
        color: usize,
        rave_table: &RaveTable,
        fpu_base: f32,
        is_root: bool,
    ) -> f32 {
        if is_root {
            return if self.n > 0. {
                self.q / self.n + c * sqrt(ln(parent_n) / self.n)
            } else {
                std::f32::INFINITY
            };
        }
        let (rave_q, rave_n) = rave_table.get_values(self.action, color);
        let beta = (rave_n / (rave_n + self.n + 4. * B_SQUARED * rave_n * self.n)).min(1.);
        if self.n > 0. {
            (1. - beta) * self.q / self.n
                + beta * rave_q / rave_n
                + self.q / self.n
                + c * sqrt(ln(parent_n) / self.n)
        } else {
            beta * rave_q / rave_n + (1. - beta) * fpu_base + c * sqrt(ln(parent_n))
        }
    }

    pub fn group_with_max_uct_value(&mut self) -> usize {
        let c_adjusted = C + C_FACTOR * 0.1 * ln((1. + self.n + C_BASE) / C_BASE);
        let mut max_value = std::f32::NEG_INFINITY;
        let mut best_group = 0;
        for (group_index, group) in self.child_groups.iter().enumerate() {
            let value = group.get_uct_value(self.n, c_adjusted);
            if value > max_value {
                max_value = value;
                best_group = group_index;
            }
        }
        //&mut self.child_groups[best_group]
        best_group
    }

    pub fn select_child(&mut self, color: usize, rave_table: &RaveTable, is_root: bool) -> usize {
        // select group
        let group = self.group_with_max_uct_value();
        // select member of the group
        self.child_groups[group].member_with_max_uct_value(self, color, rave_table, is_root)
    }

    #[inline(always)]
    fn backpropagate(&mut self, q: f32, child: Option<usize>) {
        self.n += 1.;
        self.q += q;
        if let Some(child) = child {
            for group in self.children[child].member_of_parents_group.iter() {
                self.child_groups[*group].q += q;
                self.child_groups[*group].n += 1.;
            }
        }
    }

    fn expand(&mut self, state: &GameState, al: &mut ActionList) {
        state.get_possible_actions(al);
        let h = Heuristic::for_state(state);
        let n_groups = h.get_num_groups();
        self.children = Vec::with_capacity(al.size);
        self.child_groups = Vec::with_capacity(n_groups);
        for _ in 0..n_groups.max(1) {
            self.child_groups.push(Group::new());
        }
        for i in 0..al.size {
            let action = al[i];
            let mut member_of = h.group(action);
            if action.is_skip() {
                member_of.push(0);
                break;
            }
            for m in member_of.iter() {
                self.child_groups[*m].members.push(i);
            }
            self.children.push(Node {
                children: Vec::new(),
                action,
                n: 0.,
                q: 0.,
                child_groups: Vec::new(),
                member_of_parents_group: member_of,
            });
        }
        for group in self.child_groups.iter_mut() {
            if group.members.is_empty() {
                //panic!("Empty group");
                group.n = 10_000.;
                group.q = 0.;
            }
        }
    }

    pub fn iteration(
        &mut self,
        al: &mut ActionList,
        state: &mut GameState,
        rng: &mut SmallRng,
        rave_table: &mut RaveTable,
        depth: u8,
        is_root: bool,
    ) -> f32 {
        let delta;
        if self.children.is_empty() {
            if !state.is_game_over() {
                if self.n as usize % VISITS_BEFORE_EXPANSION == 1 {
                    self.expand(state, al);
                }
                let result = playout(&mut state.clone(), rng, rave_table);
                delta = if state.ply % 2 == 0 {
                    1. - result
                } else {
                    result
                };
            } else if self.n == 0. {
                let result = state.game_result();
                self.q = match result * state.get_team() {
                    r if r > 0 => 0.999 + (result.abs() as f32) / 100_000.,
                    r if r < 0 => 0.001 - (result.abs() as f32) / 100_000.,
                    _ => 0.5,
                };
                self.n = 1.;
                delta = self.q / self.n;
            } else {
                delta = self.q / self.n;
            }
            self.backpropagate(delta, None);
            return 1. - delta;
        }
        //let next_child =#
        let next_child_index =
            self.select_child(state.get_current_color() as usize, &rave_table, is_root);
        let next_child = &mut self.children[next_child_index];
        state.do_action(next_child.action);
        delta = next_child.iteration(al, state, rng, rave_table, depth + 1, false);
        self.backpropagate(delta, Some(next_child_index));
        1. - delta
    }

    pub fn pv(&mut self, state: &mut GameState, al: &mut ActionList) {
        if self.children.is_empty() {
            return;
        }
        let child = self.best_child();
        let action = child.action;
        al.push(action);
        state.do_action(action);
        child.pv(state, al);
    }

    pub fn best_child(&mut self) -> &mut Node {
        let mut max_value = std::f32::NEG_INFINITY;
        let mut best_group = 0;
        for (group_index, group) in self.child_groups.iter().enumerate() {
            let value = 1. - group.get_value();
            if value > max_value {
                max_value = value;
                best_group = group_index;
            }
        }
        let mut best_child = 0;
        let mut best_child_value = std::f32::NEG_INFINITY;
        for member in self.child_groups[best_group].members.iter() {
            let value = self.children[*member].get_value();
            if value > best_child_value {
                best_child_value = value;
                best_child = *member
            }
        }
        &mut self.children[best_child]
        //&mut self.child_groups[best_group]
        /*
            let value = 1. - self.get_value();
            let mut best_child: usize = 0;
            let mut best_value = std::f32::NEG_INFINITY;
            for (i, child) in self.children.iter().enumerate() {
                let mut child_value = child.get_value();
                if value > 0.99 && child.action.is_set() && child.action.get_shape() == 0 {
                    child_value -= 0.05;
                }
                if child_value > best_value {
                    best_value = child_value;
                    best_child = i;
                }
            }
            &mut self.children[best_child]
        */
    }

    pub fn best_action(&mut self) -> Action {
        if self.children.is_empty() {
            Action::SKIP
        } else {
            self.best_child().action
        }
    }

    pub fn count_children(&self) -> usize {
        let mut children = 0;
        for child in self.children.iter() {
            if !child.children.is_empty() {
                children += 1;
                children += child.count_children();
            }
        }
        children
    }
}

pub struct Mcts {
    root_node: Node,
    root_state: GameState,
    time_limit: Option<i64>,
    iteration_limit: Option<usize>,
    pub rave_table: RaveTable,
}

impl Mcts {
    pub fn set_iteration_limit(&mut self, iteration_limit: Option<usize>) {
        self.iteration_limit = iteration_limit;
    }

    pub fn set_time_limit(&mut self, time_limit: Option<i64>) {
        self.time_limit = time_limit;
    }

    pub fn get_action_value_pairs(&self) -> Vec<(Action, f32)> {
        let mut ret: Vec<(Action, f32)> = Vec::with_capacity(1300);
        for child in self.root_node.children.iter() {
            ret.push((child.action, child.get_value()));
        }
        ret
    }

    pub fn get_value(&self) -> f32 {
        1. - self.root_node.get_value()
    }

    pub fn get_root_node(&mut self) -> &mut Node {
        &mut self.root_node
    }

    fn set_root(&mut self, state: &GameState) {
        loop {
            let last_board = self.root_state.board[self.root_state.get_current_color() as usize];
            let changed_fields =
                state.board[self.root_state.get_current_color() as usize] & !last_board;
            let action = Action::from_bitboard(changed_fields);
            let mut found = false;
            for (i, child) in self.root_node.children.iter().enumerate() {
                if child.action == action {
                    self.root_state.do_action(action);
                    self.root_node = self.root_node.children.remove(i);
                    found = true;
                    break;
                }
            }
            if self.root_state.ply == state.ply {
                break;
            }
            if !found {
                self.root_state = state.clone();
                self.root_node = Node::empty();
                break;
            }
        }
        self.root_state = state.clone();
    }

    fn do_iterations(&mut self, n: usize, rng: &mut SmallRng) {
        let mut al = ActionList::default();
        for _ in 0..n {
            self.root_node.iteration(
                &mut al,
                &mut self.root_state.clone(),
                rng,
                &mut self.rave_table,
                0,
                true,
            );
        }
    }

    pub fn search_action(&mut self, state: &GameState) -> Action {
        println!("Searching action using MCTS. Fen: {}", state.to_fen());
        let start_time = Instant::now();
        self.set_root(&state);
        let mut rng = SmallRng::from_entropy();
        let mut pv = ActionList::default();
        let mut iterations_per_ms = 5.;
        let mut iterations: usize = 0;

        println!("    Left Depth Iterations Value PV");
        let search_start_time = Instant::now();
        loop {
            pv.clear();
            self.root_node.pv(&mut self.root_state.clone(), &mut pv);

            let (next_iterations, stop) = if let Some(time_limit) = self.time_limit {
                let time_left = time_limit - start_time.elapsed().as_millis() as i64;
                println!(
                    "{:6}ms {:5} {:10} {:4.0}% {}",
                    time_left,
                    pv.size,
                    iterations,
                    (1. - self.root_node.get_value()).min(1.0) * 100.,
                    pv
                );
                let next_iterations =
                    ((time_left as f64 / 6.).min(5000.) * iterations_per_ms).max(1.) as usize;
                (next_iterations, time_left < 30)
            } else if let Some(iteration_limit) = self.iteration_limit {
                if iterations >= iteration_limit {
                    (0, true)
                } else {
                    let iterations_left = iteration_limit - iterations;
                    println!(
                        "{:6}it {:5} {:10} {:4.0}% {}",
                        iterations_left,
                        pv.size,
                        iterations,
                        (1. - self.root_node.get_value()).min(1.0) * 100.,
                        pv
                    );
                    let next_iterations = iterations_left as usize / 2;
                    (next_iterations, next_iterations < 100)
                }
            } else {
                panic!("Mcts has neither a time limit nor a node limit");
            };
            if stop {
                break;
            }
            self.do_iterations(next_iterations, &mut rng);
            iterations += next_iterations;
            let elapsed = search_start_time.elapsed().as_micros() as f64;
            if elapsed > 0. {
                iterations_per_ms = iterations as f64 / elapsed * 1000.;
            }
        }

        /*for child in self.root_node.children.iter() {
            for i in child.member_of_parents_group.iter() {
                println!("{:6} {:6} {:6} {:5}", self.root_node.child_groups[*i].get_value(), child.n, child.get_value(), child.action);
            }
            println!("  ");
        }*/
        for group in self.root_node.child_groups.iter() {
            println!("\n{}", group.get_value());
            for member in group.members.iter() {
                let child = &self.root_node.children[*member];
                println!("{:6} {:6} {:5}", child.n, child.get_value(), child.action);
            }
        }
        println!(
            "Search finished after {}ms. Value: {:.0}% PV-Depth: {} Iterations: {} Iterations/s: {:.2} PV: {}",
            start_time.elapsed().as_millis(),
            (1. - self.root_node.get_value()).min(1.0) * 100.,
            pv.size,
            iterations,
            iterations_per_ms * 1000.,
            pv,
        );
        //self.tree_statistics();
        self.root_node.best_action()
    }

    /*pub fn tree_statistics(&self) {
        let nodes = self.root_node.count_children();
        println!("{}", nodes);
    }*/
}

impl Player for Mcts {
    fn on_move_request(&mut self, state: &GameState) -> Action {
        self.search_action(state)
    }

    fn on_reset(&mut self) {
        self.root_node = Node::empty();
        self.rave_table = RaveTable::default();
    }
}

impl Default for Mcts {
    fn default() -> Self {
        Self {
            root_node: Node::empty(),
            root_state: GameState::default(),
            time_limit: Some(1960),
            iteration_limit: None,
            rave_table: RaveTable::default(),
        }
    }
}

/*use super::float_stuff::{ln, sqrt};
use super::playout::playout;
use game_sdk::{Action, ActionList, GameState};
use rand::{rngs::SmallRng, RngCore, SeedableRng};

use super::mcts::RaveTable;

const C: f32 = 0.0;
const C_BASE: f32 = 200.0;
const C_FACTOR: f32 = std::f32::consts::SQRT_2;

pub struct Node {
    n: f32,
    q: f32,
    action: Action,
    children: Vec<Node>,
    child_groups: Vec<Group>,
    member_of_parents_group: Vec<usize>,
}

impl Node {
    pub fn empty() -> Self {
        Self {
            n: 0.,
            q: 0.,
            action: Action::SKIP,
            children: Vec::new(),
            child_groups: Vec::new(),
            member_of_parents_group: Vec::new(),
        }
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
            self.q / self.n + c * sqrt(ln(parent_n) / self.n)
        } else {
            std::f32::INFINITY
        }
    }

    pub fn group_with_max_uct_value(&self) -> usize {
        let c_adjusted = C + C_FACTOR * ln((1. + self.n + C_BASE) / C_BASE);
        let mut best_group = 0;
        let mut best_value = std::f32::NEG_INFINITY;
        for (index, group) in self.child_groups.iter().enumerate() {
            let group_value = group.get_uct_value(self.n, c_adjusted);
            if group_value > best_value {
                best_value = group_value;
                best_group = index;
            }
        }
        best_group
    }

    pub fn select_next_child(&self) -> &mut Node {
        let group = self.group_with_max_uct_value();
        &mut self.children[self.child_groups[group].member_with_max_uct_value(&self)]
    }

    pub fn backpropagate(&mut self, value: f32, parent: &mut Node) {
        self.n += 1.;
        self.q += value;
        for group_index in self.member_of_parents_group.iter() {
            parent.child_groups[*group_index].backpropagate(value);
        }
    }

    pub fn expand(&mut self, state: &mut GameState, al: &mut ActionList) {
        state.get_possible_actions(al);
        self.children = Vec::with_capacity(al.size);
        for i in 0..al.size {
            let action = al[i];
            self.children.push(Node {
                n: 0.,
                q: 0.,
                action,
                children: Vec::new(),
                child_groups: vec![Group::all(al.size)],
                member_of_parents_group: vec![0; 1],
            });
        }
    }

    pub fn iteration(
        &mut self,
        al: &mut ActionList,
        state: &mut GameState,
        rng: &mut SmallRng,
        rave_table: RaveTable,
    ) -> f32 {
        let delta;
        if self.children.is_empty() {
            if !state.is_game_over() {
                self.expand(state, al);
                let result = playout(&mut state.clone(), rng, rave_table);
                delta = if state.ply % 2 == 0 {
                    1. - result
                } else {
                    result
                };
            } else if self.n == 0. {
                let result = state.game_result();
                self.q = match result * state.get_team() {
                    r if r > 0 => 0.999 + (result.abs() as f32) / 100_000.,
                    r if r < 0 => 0.001 - (result.abs() as f32) / 100_000.,
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
        let next_child = self.children[self.select_next_child()];
        state.do_action(next_child.action);
        delta = next_child.iteration(al, state, rng, rave_table);
        1. - delta
    }
}

pub struct Group {
    n: f32,
    q: f32,
    members: Vec<usize>,
}

impl Group {
    pub fn all(n: usize) -> Self {
        Self {
            n: 0.,
            q: 0.,
            members: (0..n).collect(),
        }
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
            self.q / self.n + c * sqrt(ln(parent_n) / self.n)
        } else {
            std::f32::INFINITY
        }
    }

    pub fn backpropagate(&mut self, value: f32) {
        self.n += 1.;
        self.q += value;
    }

    pub fn member_with_max_uct_value(&self, parent: &Node) -> usize {
        let c_adjusted = C + C_FACTOR * ln((1. + self.n + C_BASE) / C_BASE);
        let mut best_value = std::f32::NEG_INFINITY;
        let mut best_member = 0;
        for member_index in self.members.iter() {
            let value = parent.children[*member_index].get_uct_value(parent.n, c_adjusted);
            if value > best_value {
                best_value = value;
                best_member = *member_index;
            }
        }
        best_member
    }
}

pub struct Tree {
    root_node: Node,
    root_state: GameState,
}

impl Tree {
    pub fn new(state: &GameState) -> Self {
        Self {
            root_node: Node::empty(),
            root_state: state.clone(),
        }
    }
}
*/
/*
#[derive(Clone)]
pub struct Group {
    pub n: f32,
    pub q: f32,
    pub nodes: Vec<usize>,
    pub parent: usize,
}

impl Group {
    pub fn empty() -> Self {
        Self {
            n: 0.,
            q: 0.,
            nodes: Vec::new(),
            parent: 0,
        }
    }

    pub fn get_uct_value(&self, parent_n: f32, c: f32) -> f32 {
        if self.n > 0. {
            self.q / self.n + c * sqrt(ln(parent_n) / self.n)
        } else {
            std::f32::INFINITY
        }
    }

    pub fn random_node(&self, rng: &mut SmallRng) -> usize {
        self.nodes[rng.next_u64() as usize % self.nodes.len()]
    }

    pub fn backpropagate(&mut self, q: f32) {
        self.n += 1.0;
        self.q += q;
    }
}

#[derive(Clone)]
pub struct Node {
    n: f32,
    q: f32,
    action: Action,
    children: Vec<usize>,
    parents: Vec<usize>,
    groups: Vec<usize>,
}

impl Node {
    pub fn empty() -> Self {
        Self {
            n: 0.,
            q: 0.,
            action: Action::SKIP,
            children: Vec::new(),
            parents: Vec::new(),
            groups: Vec::new(),
        }
    }

    pub fn get_value(&self) -> f32 {
        if self.n > 0. {
            self.q / self.n
        } else {
            std::f32::NEG_INFINITY
        }
    }

    pub fn group_with_max_uct_value(&self, tree: &mut Tree, parent_n: f32) -> usize {
        let c_adjusted = C + C_FACTOR * ln((1. + self.n + C_BASE) / C_BASE);
        let mut best_group: usize = 0;
        let mut max_uct_value = std::f32::NEG_INFINITY;
        for group in self.groups.iter() {
            let group_uct_value = tree.groups[*group].get_uct_value(self.n, c_adjusted);
            if group_uct_value > max_uct_value {
                max_uct_value = group_uct_value;
                best_group = *group;
            }
        }
        best_group
    }

    pub fn expand(&mut self, tree: &mut Tree, state: &mut GameState, al: &mut ActionList) {
        state.get_possible_actions(al);
        for index in 0..al.size {
            let action = al[index];
            let i = tree.nodes.len();
            tree.nodes.push(Node {
                n: 0.,
                q: 0.,
                action,
                children: Vec::new(),
                parents: Vec::new(),
                groups: Vec::new(),
            });
            self.children.push(i);
        }
    }

    pub fn backpropagate(&mut self, tree: &mut Tree, value: f32) {
        for group in self.groups.iter() {
            tree.groups[*group].backpropagate(value);
        }
        self.n += 1.;
        self.q += value;
    }
}

pub struct Tree {
    pub nodes: Vec<Node>,
    pub groups: Vec<Group>,
    pub rave_table: RaveTable,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            nodes: vec![Node::empty(); 100_000],
            groups: vec![Group::empty(); 1_000],
            rave_table: RaveTable::default(),
        }
    }

    pub fn iteration(
        &mut self,
        node: &mut Node,
        al: &mut ActionList,
        state: &mut GameState,
        rng: &mut SmallRng,
        rave_table: &mut RaveTable,
    ) -> f32 {
        let delta;
        if node.children.is_empty() {
            if !state.is_game_over() {
                node.expand(self, state, al);
                let result = playout(&mut state.clone(), rng, rave_table);
                delta = if state.ply % 2 == 0 {
                    1. - result
                } else {
                    result
                };
            } else if node.n == 0. {
                let result = state.game_result();
                node.q = match result * state.get_team() {
                    r if r > 0 => 0.999 + (result.abs() as f32) / 100_000.,
                    r if r < 0 => 0.001 - (result.abs() as f32) / 100_000.,
                    _ => 0.5,
                };
                node.n = 1.;
                delta = node.q / node.n;
            } else {
                delta = node.q / node.n;
            }
            node.backpropagate(self, delta);
            return 1. - delta;
        }
        let group_to_pick_child_from = node.group_with_max_uct_value(self, node.n);
        let random_node_of_group = self.groups[group_to_pick_child_from].random_node(rng);
        let next_child = self.nodes[random_node_of_group];
        state.do_action(next_child.action);
        delta = self.iteration(next_child, al, state, rng, rave_table);
        node.backpropagate(self, delta);
        1. - delta
    }
}

pub struct Searcher {
    tree: Tree,
}

impl Searcher {
    pub fn new() -> Self {
        Self { tree: Tree::new() }
    }
    pub fn search(&mut self, state: &GameState) {
        self.tree.nodes.truncate(0);
        self.tree.nodes.push(Node::empty());
        let mut al = ActionList::default();
        let mut rng = SmallRng::from_entropy();

        for _ in 0..10_000 {
            self.tree.iteration(
                0,
                &mut al,
                &mut self.tree.root_state,
                &mut rng,
                &mut self.tree.rave_table,
            );
        }
    }
}
*/
