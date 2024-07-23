use std::collections::VecDeque;


use fnv::FnvHashMap;

use crate::model::{GameMove, GameSpec, GameState};

pub fn solve(spec: &GameSpec, initial_state: &GameState) -> Vec<GameMove> {
    let mut back_edge = FnvHashMap::default();

    let mut q = VecDeque::new();
    q.push_back(initial_state.clone());
    back_edge.insert(initial_state.clone(), (GameMove::default(), initial_state.clone()));

    let mut final_state: Option<GameState> = None;
    'mainloop: while !q.is_empty() {
        let current_state = q.pop_front().unwrap();
    
        for (game_move, next_state) in spec.next_states(&current_state) {
            if back_edge.contains_key(&next_state) {
                continue;
            }            
            back_edge.insert(next_state.clone(), (game_move, current_state.clone()));
            if spec.is_winning_state(&next_state) {
                final_state = Some(next_state);
                break 'mainloop;
            }
            q.push_back(next_state);
        }
    }
    assert!(final_state != None);
    let mut state = final_state.unwrap();
    let mut moves: Vec<GameMove> = vec![];

    while state != *initial_state {
        let (game_move, prev_state) = back_edge.get(&state).unwrap();
        moves.push(game_move.clone());
        state = prev_state.clone();
    }
    moves.reverse();
    return moves;
}