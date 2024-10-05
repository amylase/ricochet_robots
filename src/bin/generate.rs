use model::{GameSpec, GameState, Point};
use ricochet_robots::{model::{self, TargetType}, serialize::{self}, solver};

use std::collections::{HashSet, VecDeque};

use bitvec::{bitvec, order::Msb0};
use itertools::Itertools;
use rand::{self, Rng};
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

    available_cells
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
            GameState { robots }
        })
        .collect()
}

fn all_winning_states(spec: &GameSpec) -> Vec<GameState> {
    match spec.target_type {
        model::TargetType::Particular(target_robot) => winning_states(spec, target_robot),
        model::TargetType::Any => (0..ROBOT_COUNT)
            .flat_map(|robot_index| winning_states(spec, robot_index).into_iter())
            .collect(),
    }
}


fn is_acceptable_final_state(spec: &GameSpec, state: &GameState) -> bool {
    state.robots.iter()
    .map(|robot| {
        spec.walls[robot.r as usize * 2 + 1][robot.c as usize * 2 + 2] as i64 +
        spec.walls[robot.r as usize * 2 + 1][robot.c as usize * 2] as i64 +
        spec.walls[robot.r as usize * 2 + 2][robot.c as usize * 2 + 1] as i64 +
        spec.walls[robot.r as usize * 2][robot.c as usize * 2 + 1] as i64
    })
    .sum::<i64>() <= 0
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

    let mut result_state = None;
    while !q.is_empty() {
        let visiting_state = q.pop_front().unwrap();
        if is_acceptable_final_state(spec, &visiting_state) {
            result_state = Some(visiting_state.clone());
        }

        'mainloop: for next_state in spec.prev_states(&visiting_state) {
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
    result_state.unwrap()
}

fn generate_spec() -> GameSpec {
    let mut rng = rand::thread_rng();

    'mainloop: loop {
        const GOAL_SPOTS: usize = 17;
        const DIRECTION_POINTS: [Point; 4] = [  // needs to be clockwise or counter-clock wise order
            Point::new(-1, 0),
            Point::new(0, 1),
            Point::new(1, 0),
            Point::new(0, -1),
        ];

        let mut walls = [[false; WALL_MAP_SIZE]; WALL_MAP_SIZE];
        (7..=8).for_each(|r| {
            (7..=8).for_each(|c| {
                let center = Point::new(r * 2 + 1, c * 2 + 1);
                DIRECTION_POINTS.iter().for_each(|dp| {
                    let wall_position = center + *dp;
                    walls[wall_position.r as usize][wall_position.c as usize] = true;
                });
            });
        });
        (0..WALL_MAP_SIZE).for_each(|i| {
            walls[0][i] = true;
            walls[WALL_MAP_SIZE - 1][i] = true;
            walls[i][0] = true;
            walls[i][WALL_MAP_SIZE - 1] = true;
        });

        let mut goals = [Point::new(0, 0); GOAL_SPOTS];
        for i in 0..GOAL_SPOTS {
            'sampling: loop {
                let candidate = Point::new(rng.gen::<i8>().abs() % BOARD_SIZE as i8, rng.gen::<i8>().abs() % BOARD_SIZE as i8);
                if 7 <= candidate.r && candidate.r <= 8 && 7 <= candidate.c && candidate.c <= 8 {
                    continue 'sampling;
                }
                if candidate.r == 0 || candidate.r + 1 == BOARD_SIZE as i8 || candidate.c == 0 || candidate.c + 1 == BOARD_SIZE as i8 {
                    continue 'sampling;
                }

                if goals.iter().take(i).any(|other| candidate.chebyshev(other) <= 1) {
                    continue 'sampling;
                }
                goals[i] = candidate;
                let wall_direction = rng.gen::<usize>() % 4;
                (0..2).for_each(|offset| {
                    let direction_index = (wall_direction + offset) % 4;
                    let wall_position = candidate * 2 + Point::new(1, 1) + DIRECTION_POINTS[direction_index];
                    walls[wall_position.r as usize][wall_position.c as usize] = true;
                });
                break 'sampling;
            }
        }

        (0..2).for_each(|_i| {
            let x = rng.gen::<usize>() % (BOARD_SIZE - 1);
            walls[1][x * 2 + 2] = true;

            let x = rng.gen::<usize>() % (BOARD_SIZE - 1);
            walls[WALL_MAP_SIZE - 2][x * 2 + 2] = true;

            let x = rng.gen::<usize>() % (BOARD_SIZE - 1);
            walls[x * 2 + 2][1] = true;

            let x = rng.gen::<usize>() % (BOARD_SIZE - 1);
            walls[x * 2 + 2][WALL_MAP_SIZE - 2] = true;
        });

        for r in 0..BOARD_SIZE {
            for c in 0..BOARD_SIZE {
                if (7..=8).contains(&r) && (7..=8).contains(&c) {
                    continue;
                }
                let center = Point::new(r as i8 * 2 + 1, c as i8 * 2 + 1);
                let surrounding_walls: usize = DIRECTION_POINTS.iter().map(|dp| {
                    let wall_position = center + *dp;
                    if walls[wall_position.r as usize][wall_position.c as usize] { 1 } else { 0 }
                }).sum();
                if surrounding_walls >= 3 {
                    continue 'mainloop;
                }
            }
        }
        
        let target_index: usize = rng.gen::<usize>() % GOAL_SPOTS;
        let target_type = if target_index < (GOAL_SPOTS - 1) { 
            TargetType::Particular(target_index / 4) 
        } else { 
            TargetType::Any 
        }; 
        return GameSpec::new(walls, goals[target_index], target_type)
    }
}

fn main() {
    let spec = generate_spec();
    println!("generated a board. search for the robot arrangement that maximizes the answer.");

    let farthest_state = reverse_bfs(&spec);
    let maximized_id = dump(&spec, &farthest_state);
    println!("found a farthest state. solving a problem for this.");
    println!(
        "https://kaseken.github.io/ricochet_robots/#/?id={}",
        maximized_id
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
