use model::{GameSpec, GameState, Point};
use ricochet_robots::{model, serialize, solver};

use std::{
    collections::{HashSet, VecDeque},
    io::{self, BufRead},
};

use bitvec::{bitvec, order::Msb0};
use itertools::Itertools;
use model::{BOARD_SIZE, DIRECTIONS, ROBOT_COUNT, WALL_MAP_SIZE};
use serialize::dump;

fn winning_states(spec: &GameSpec, goal_robot: usize) -> Vec<GameState> {
    let mut vis = HashSet::new();
    let mut q = VecDeque::new();

    vis.insert(spec.goal);
    q.push_back(spec.goal);

    while !q.is_empty() {
        let position = q.pop_front().unwrap();
        for direction in DIRECTIONS {
            let next = position + Point::from(direction);
            if next.r < 0
                || next.r >= WALL_MAP_SIZE as i8
                || next.c < 0
                || next.c >= WALL_MAP_SIZE as i8
            {
                continue;
            }
            if spec.walls[next.r as usize][next.c as usize] {
                continue;
            }
            if vis.contains(&next) {
                continue;
            }
            vis.insert(next);
            q.push_back(next);
        }
    }

    let mut available_cells = Vec::new();
    for r in 0..BOARD_SIZE {
        for c in 0..BOARD_SIZE {
            let r = r as i8;
            let c = c as i8;
            let wall_cell = Point::new(r * 2 + 1, c * 2 + 1);
            let field_cell = Point::new(r, c);
            if vis.contains(&wall_cell) && field_cell != spec.goal {
                available_cells.push(field_cell);
            }
        }
    }

    return available_cells
        .into_iter()
        .permutations(ROBOT_COUNT - 1)
        .map(|points| {
            let mut robots = [spec.goal; ROBOT_COUNT];
            for (i, point) in points.into_iter().enumerate() {
                if i < goal_robot {
                    robots[i] = point
                } else {
                    robots[i + 1] = point
                }
            }
            GameState { robots: robots }
        })
        .collect();
}

fn all_winning_states(spec: &GameSpec) -> Vec<GameState> {
    match spec.target_type {
        model::TargetType::Particular(target_robot) => winning_states(spec, target_robot),
        model::TargetType::Any => (0..ROBOT_COUNT)
            .into_iter()
            .flat_map(|robot_index| winning_states(spec, robot_index).into_iter())
            .collect(),
    }
}

pub fn reverse_bfs(spec: &GameSpec) -> GameState {
    let mut vis = bitvec![u64, Msb0; 0; 1 << 32];
    let mut q = VecDeque::new();

    'mainloop: for winning_state in all_winning_states(spec) {
        for equivalent_state in
            spec.equivalent_states_particular(&winning_state, spec.target_type.robot_index(0))
        {
            if *vis.get(equivalent_state.to_u32() as usize).unwrap() {
                continue 'mainloop;
            }
        }
        q.push_back(winning_state.clone());
        vis.set(winning_state.to_u32() as usize, true);
    }

    let mut visiting_state = None;
    while !q.is_empty() {
        visiting_state = q.pop_front();

        'mainloop: for next_state in spec.prev_states(visiting_state.as_ref().unwrap()) {
            for equivalent_state in
                spec.equivalent_states_particular(&next_state, spec.target_type.robot_index(0))
            {
                if *vis.get(equivalent_state.to_u32() as usize).unwrap() {
                    continue 'mainloop;
                }
            }
            vis.set(next_state.to_u32() as usize, true);
            q.push_back(next_state);
        }
    }
    return visiting_state.unwrap();
}

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let board = lines.next().unwrap().unwrap();

    let (spec, _) = serialize::load(&board);

    let farthest_state = reverse_bfs(&spec);
    println!("found a farthest state. solving a problem for this.");
    println!(
        "https://kaseken.github.io/ricochet_robots/#/?id={}",
        dump(&spec, &farthest_state)
    );

    let result = solver::solve_bfs(&spec, &farthest_state);
    println!("found a solution with {} moves", result.len());
    for game_move in result {
        println!(
            "> Move {} to {:?} ",
            serialize::robot_index_to_color(game_move.robot_index),
            game_move.direction
        )
    }
}
