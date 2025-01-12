use std::collections::VecDeque;

use bitvec::{bitvec, order::Msb0};

use crate::model::{GameMove, GameSpec, GameState, GAME_MOVES};

pub fn solve_bfs(spec: &GameSpec, initial_state: &GameState) -> Vec<GameMove> {
    let mut back_edge = Vec::with_capacity(100_000_000);
    let mut vis = bitvec![u64, Msb0; 0; 1 << 32];

    let mut q = VecDeque::new();
    q.push_back(initial_state.clone());
    vis.set(initial_state.to_u32() as usize, true);

    let mut final_state: Option<GameState> = None;
    'mainloop: while !q.is_empty() {
        let current_state = q.pop_front().unwrap();

        for (i, next_state) in spec.next_states(&current_state).into_iter().enumerate() {
            let next_state_id = next_state.to_u32() as usize;
            if *vis.get(next_state_id).unwrap() {
                continue;
            }
            back_edge.push((next_state.clone(), i, current_state.clone()));
            vis.set(next_state_id, true);
            if spec.is_winning_state(&next_state) {
                final_state = Some(next_state);
                break 'mainloop;
            }
            q.push_back(next_state);
        }
    }

    if final_state.is_none() {
        return Vec::new();
    }
    let mut state = final_state.unwrap();
    let mut moves: Vec<GameMove> = vec![];

    while state != *initial_state {
        for (current_state, game_move_index, prev_state) in back_edge.iter() {
            if *current_state == state {
                moves.push(GAME_MOVES[*game_move_index].clone());
                state = prev_state.clone();
                break;
            }
        }
    }
    moves.reverse();
    moves
}
