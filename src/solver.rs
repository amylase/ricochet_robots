use std::collections::{HashMap, VecDeque};

use crate::model::{GameSpec, GameState};

pub fn solve_bfs(spec: &GameSpec, initial_state: &GameState) -> u32 {
    let mut memo = HashMap::new();

    let mut q = VecDeque::new();
    q.push_back(initial_state.clone());
    memo.insert(initial_state.clone(), 0);

    while !q.is_empty() {
        let current_state = q.pop_front().unwrap();
        let current_cost = memo.get(&current_state).unwrap().clone();
        let next_cost = current_cost + 1;
    
        for next_state in spec.next_states(&current_state) {
            if spec.is_winning_state(&next_state) {
                return next_cost;
            }
            if memo.contains_key(&next_state) {
                continue;
            }
            memo.insert(next_state.clone(), next_cost);
            q.push_back(next_state);
        }
    }
    unreachable!();
}