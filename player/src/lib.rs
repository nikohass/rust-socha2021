pub mod cache;
pub mod evaluation;
pub mod mcts;
pub mod neural_network;
pub mod search;

pub mod simple_client {
    use game_sdk::Player;
    use game_sdk::{Action, ActionList, GameState};
    use rand::{rngs::SmallRng, RngCore, SeedableRng};

    pub struct SimpleClient {
        rng: SmallRng,
        action_list: ActionList,
    }

    impl SimpleClient {
        pub fn get_action(&mut self, state: &GameState) -> Action {
            state.get_possible_actions(&mut self.action_list);
            self.action_list[self.rng.next_u64() as usize % self.action_list.size]
        }
    }

    impl Player for SimpleClient {
        fn on_move_request(&mut self, state: &GameState) -> Action {
            self.get_action(state)
        }
    }

    impl Default for SimpleClient {
        fn default() -> Self {
            Self {
                rng: SmallRng::from_entropy(),
                action_list: ActionList::default(),
            }
        }
    }
}

// Some of the built-in float functions caused the client to crash on the Software-Challenge server
pub mod float_stuff {
    #[inline(always)]
    pub fn sqrt(x: f32) -> f32 {
        let bits = f32::to_bits(x);
        f32::from_bits((bits >> 1) & 0x1fbb4000 | (bits & !0x1fbb4000))
    }

    #[inline(always)]
    pub fn relu(x: f32) -> f32 {
        f32::max(0., x)
    }

    // functions from https://github.com/loony-bean/fastapprox-rs

    pub fn ln(x: f32) -> f32 {
        std::f32::consts::LN_2 * log2(x)
    }

    pub fn log2(x: f32) -> f32 {
        let vx = f32::to_bits(x);
        let mx = f32::from_bits((vx & 0x007FFFFF_u32) | 0x3f000000);
        let mut y = vx as f32;
        y *= 1.192_092_9e-7_f32;
        y - 124.225_52_f32 - 1.498_030_3_f32 * mx - 1.725_88_f32 / (0.352_088_72_f32 + mx)
    }

    #[inline]
    fn pow2(p: f32) -> f32 {
        let offset = if p < 0.0 { 1.0_f32 } else { 0.0_f32 };
        let clipp = if p < -126.0 { -126.0_f32 } else { p };
        let w = clipp as i32;
        let z = clipp - (w as f32) + offset;
        let v = ((1 << 23) as f32
            * (clipp + 121.274_06_f32 + 27.728_02_f32 / (4.842_526_f32 - z) - 1.490_129_1_f32 * z))
            as u32;
        f32::from_bits(v)
    }

    #[inline]
    fn exp(p: f32) -> f32 {
        pow2(std::f32::consts::LOG2_E * p)
    }

    #[inline]
    pub fn sigmoid(x: f32) -> f32 {
        1.0_f32 / (1.0_f32 + exp(-x))
    }
}
