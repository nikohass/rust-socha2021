use super::action::Action;
use std::ops::{Index, IndexMut};

pub const MAX_ACTIONS: usize = 1300;

#[derive(Clone)]
pub struct ActionList {
    actions: [Action; MAX_ACTIONS],
    pub size: usize,
}

impl ActionList {
    pub fn push(&mut self, action: Action) {
        self.actions[self.size] = action;
        self.size += 1;
    }
}

impl Default for ActionList {
    fn default() -> Self {
        let actions = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        ActionList { actions, size: 0 }
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
        ActionListStack {
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
