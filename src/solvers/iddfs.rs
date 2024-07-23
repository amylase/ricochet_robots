use fnv::FnvHashMap;

use crate::model::{GameMove, GameSpec, GameState};

pub fn solve(spec: &GameSpec, initial_state: &GameState) -> Vec<GameMove> {
    let mut depth = 0;
    loop {
        let mut max_depth = FnvHashMap::default();
        match dfs(spec, initial_state, &mut max_depth, depth) {
            Some(moves) => return moves,
            None => (),
        }
        depth += 1;
    }
}


fn dfs(spec: &GameSpec, current_state: &GameState, max_depth: &mut FnvHashMap<GameState, usize>, depth: usize) -> Option<Vec<GameMove>> {
    if depth == 0 {
        return None;
    }
    if spec.is_winning_state(current_state) {
        return Some(Vec::new());
    }
    for (game_move, next_state) in spec.next_states(&current_state) {
        let next_depth = depth - 1;
        if *max_depth.get(&next_state).unwrap_or(&0) >= next_depth {
            continue;
        }
        max_depth.insert(next_state.clone(), next_depth);
        match dfs(spec, &next_state, max_depth, depth - 1) {
            Some(mut moves) => {
                let mut new_moves = Vec::new();
                new_moves.push(game_move);
                new_moves.append(&mut moves);
                return Some(new_moves);
            },
            None => (),
        }
    }
    return None;
}