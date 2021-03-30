use super::{Action, Bitboard};
use std::fmt::{Display, Formatter, Result};
use std::ops::{Index, IndexMut};

pub const MAX_ACTIONS: usize = 1300;

#[derive(Clone)]
pub struct ActionList {
    actions: [Action; MAX_ACTIONS],
    pub size: usize,
}

impl ActionList {
    #[inline(always)]
    pub fn push(&mut self, action: Action) {
        self.actions[self.size] = action;
        self.size += 1;
    }

    #[inline(always)]
    pub fn swap(&mut self, x: usize, y: usize) {
        let tmp = self[x];
        self.actions[x] = self[y];
        self.actions[y] = tmp;
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.size = 0;
    }

    #[inline(always)]
    pub fn append(&mut self, mut destinations: Bitboard, shape: u16) {
        while destinations.0 != 0 {
            let d = destinations.0.trailing_zeros();
            destinations.0 ^= 1 << d;
            self.push(Action::set(d as u16 + 384, shape));
        }
        while destinations.1 != 0 {
            let d = destinations.1.trailing_zeros();
            destinations.1 ^= 1 << d;
            self.push(Action::set(d as u16 + 256, shape));
        }
        while destinations.2 != 0 {
            let d = destinations.2.trailing_zeros();
            destinations.2 ^= 1 << d;
            self.push(Action::set(d as u16 + 128, shape));
        }
        while destinations.3 != 0 {
            let d = destinations.3.trailing_zeros();
            destinations.3 ^= 1 << d;
            self.push(Action::set(d as u16, shape));
        }
    }
}

impl Default for ActionList {
    fn default() -> Self {
        let actions = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        Self { actions, size: 0 }
    }
}

impl Display for ActionList {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut ret = String::new();
        for i in 0..self.size {
            if i != 0 {
                ret.push_str(", ");
            }
            ret.push_str(&self[i].to_short_name());
        }
        write!(f, "{}", ret)
    }
}

impl Index<usize> for ActionList {
    type Output = Action;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        if index < self.size {
            &self.actions[index]
        } else {
            panic!(
                "Index out of bounds for ActionList, given index: {}, size: {}, actions: {:?}",
                index,
                self.size,
                self.actions[0..self.size].to_vec()
            );
        }
    }
}

pub struct ActionListStack {
    pub action_lists: Vec<ActionList>,
}

impl ActionListStack {
    pub fn with_size(size: usize) -> Self {
        Self {
            action_lists: vec![ActionList::default(); size],
        }
    }
}

impl Index<usize> for ActionListStack {
    type Output = ActionList;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.action_lists.len() {
            &self.action_lists[index]
        } else {
            panic!("Can not extend ActionListStack in non mutable index");
        }
    }
}

impl IndexMut<usize> for ActionListStack {
    fn index_mut(&mut self, index: usize) -> &mut ActionList {
        if index < self.action_lists.len() {
            &mut self.action_lists[index]
        } else {
            self.action_lists
                .append(vec![ActionList::default(); index + 1 - self.action_lists.len()].as_mut());
            self.index_mut(index)
        }
    }
}
