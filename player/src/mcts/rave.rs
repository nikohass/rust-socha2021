use game_sdk::Action;

const SHAPES: usize = 91;
const DESTINATIONS: usize = 418;
const COLORS: usize = 4;
const MAX_SET_INDEX: usize =
    ((DESTINATIONS - 1) + DESTINATIONS * (SHAPES - 1)) * COLORS + (COLORS - 1);

pub struct RaveTable {
    pub actions: Vec<(f32, f32)>,
}

impl RaveTable {
    fn index(action: Action, color: usize) -> usize {
        if action.is_set() {
            let destination = action.get_destination() as usize;
            let shape = action.get_shape() as usize;
            (destination + DESTINATIONS * shape) * COLORS + color
        } else {
            MAX_SET_INDEX + color
        }
    }

    pub fn get_values(&self, action: Action, color: usize) -> (f32, f32) {
        self.actions[Self::index(action, color)]
    }

    pub fn add_value(&mut self, action: Action, color: usize, value: f32) {
        let entry = self.actions.get_mut(Self::index(action, color)).unwrap();
        entry.0 += 1.;
        entry.1 += value;
    }
}

impl Default for RaveTable {
    fn default() -> Self {
        let actions: Vec<(f32, f32)> = vec![(0., 0.); MAX_SET_INDEX + COLORS];
        Self { actions }
    }
}
