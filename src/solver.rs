use std::collections::VecDeque;

use bitvec::{order::Msb0, bitvec};

use crate::model::{GameMove, GameSpec, GameState};

pub fn solve_bfs(spec: &GameSpec, initial_state: &GameState) -> Vec<Vec<GameMove>> {
    let mut back_edge = Vec::new();
    let mut vis = bitvec![u64, Msb0; 0; 1 << 32];

    let mut q = VecDeque::new();
    q.push_back(initial_state.clone());
    vis.set(initial_state.to_u32() as usize, true);

    let mut final_states: Vec<GameState> = Vec::new();
    let mut current_last = initial_state.to_u32();
    'mainloop: while !q.is_empty() {
        let current_state = q.pop_front().unwrap();
    
        for (game_move, next_state) in spec.next_states(&current_state) {
            let next_state_id = next_state.to_u32();
            if *vis.get(next_state_id as usize).unwrap() {
                continue;
            }            
            back_edge.push((next_state.clone(), game_move, current_state.clone()));
            vis.set(next_state_id as usize, true);
            if spec.is_winning_state(&next_state) {
                final_states.push(next_state.clone());
            }
            q.push_back(next_state);
        }
        if current_state.to_u32() == current_last {
            if q.is_empty() || !final_states.is_empty() {
                break 'mainloop;
            }
            current_last = q.back().unwrap().to_u32();
        }
    }
    assert!(!final_states.is_empty());
    let mut answers = vec![];
    for final_state in final_states {
        let mut state = final_state; 
        let mut moves: Vec<GameMove> = vec![];
    
        while state != *initial_state {
            for (current_state, game_move, prev_state) in back_edge.iter() {
                if *current_state == state {
         
                    moves.push(game_move.clone());
                    state = prev_state.clone();
                    break;
                }
            }
        }
        moves.reverse();
        answers.push(moves);
    }
    
    return answers;
}