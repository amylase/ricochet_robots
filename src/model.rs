use std::convert::From;
use std::hash::Hash;
use std::ops;
use std::sync::LazyLock;
use std::{array, cmp::min};

use crate::algorithm::{factorial, permutation_swaps};

pub const ROBOT_COUNT: usize = 4;
pub const BOARD_SIZE: usize = 16;
pub const WALL_MAP_SIZE: usize = BOARD_SIZE * 2 + 1;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Point {
    pub r: i8,
    pub c: i8,
}

impl Point {
    pub fn new(r: i8, c: i8) -> Point {
        Point { r, c }
    }

    fn rot(&self) -> Point {
        // (1, 2) -> (-2, 1)
        Point {
            r: -self.c,
            c: self.r,
        }
    }

    fn rrot(&self) -> Point {
        // (-2, 1) -> (1, 2)
        Point {
            r: self.c,
            c: -self.r,
        }
    }
}

impl ops::Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point {
            r: self.r + rhs.r,
            c: self.c + rhs.c,
        }
    }
}

impl ops::Mul<i8> for Point {
    type Output = Point;

    fn mul(self, rhs: i8) -> Self::Output {
        Point {
            r: self.r * rhs,
            c: self.c * rhs,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

pub const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Left,
    Direction::Right,
];

impl Direction {
    fn reverse(&self) -> Direction {
        match &self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

impl From<Direction> for Point {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Up => Point::new(-1, 0),
            Direction::Down => Point::new(1, 0),
            Direction::Left => Point::new(0, -1),
            Direction::Right => Point::new(0, 1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetType {
    Any,
    Particular(usize),
}

impl TargetType {
    pub fn robot_index(self, or_else: usize) -> usize {
        match self {
            Self::Any => or_else,
            Self::Particular(robot_index) => robot_index,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameMove {
    pub robot_index: u8,
    pub direction: Direction,
}

pub static GAME_MOVES: LazyLock<[GameMove; ROBOT_COUNT * 4]> = LazyLock::new(|| {
    array::from_fn(|i| GameMove {
        robot_index: i as u8 / 4,
        direction: DIRECTIONS[i % 4],
    })
});

type WallBoard = [[bool; WALL_MAP_SIZE]; WALL_MAP_SIZE];
type WallCache = [[[u8; 4]; BOARD_SIZE]; BOARD_SIZE];

static THREE_PERMUTATION_SWAPS: LazyLock<[usize; 5]> = LazyLock::new(|| {
    let v = permutation_swaps(3);
    array::from_fn(|i| *v.get(i).unwrap())
});
static FOUR_PERMUTATION_SWAPS: LazyLock<[usize; 23]> = LazyLock::new(|| {
    let v = permutation_swaps(4);
    array::from_fn(|i| *v.get(i).unwrap())
});

#[derive(Debug)]
pub struct GameSpec {
    pub walls: WallBoard,
    pub goal: Point,
    pub target_type: TargetType,

    wall_cache: WallCache,
}

fn _has_wall(walls: &WallBoard, position: Point, direction: Direction) -> bool {
    let wall_position = position * 2 + Point::new(1, 1) + Point::from(direction);
    walls[wall_position.r as usize][wall_position.c as usize]
}

impl GameSpec {
    pub fn new(walls: WallBoard, goal: Point, target_type: TargetType) -> GameSpec {
        let mut wall_cache = [[[0; 4]; BOARD_SIZE]; BOARD_SIZE];
        wall_cache.iter_mut().enumerate().for_each(|(r, row)| {
            row.iter_mut().enumerate().for_each(|(c, cell)| {
                for direction in DIRECTIONS {
                    let mut steps: u8 = 0;
                    let mut position = Point::new(r as i8, c as i8);
                    loop {
                        if _has_wall(&walls, position, direction) {
                            break;
                        }
                        let next_position = position + Point::from(direction);
                        position = next_position;
                        steps += 1;
                    }
                    cell[direction as usize] = steps;
                }
            })
        });

        GameSpec {
            walls,
            goal,
            target_type,
            wall_cache,
        }
    }

    pub fn prev_states(&self, current_state: &GameState) -> Vec<GameState> {
        let mut results = Vec::new();
        for robot_index in 0..ROBOT_COUNT {
            for back_direction in DIRECTIONS {
                let direction = back_direction.reverse();
                let mut position = current_state.robots[robot_index];
                if !(_has_wall(&self.walls, position, direction)
                    || current_state.has_robot(position + Point::from(direction)))
                {
                    continue;
                }

                loop {
                    if _has_wall(&self.walls, position, back_direction) {
                        break;
                    }
                    let next_position = position + Point::from(back_direction);
                    if current_state.has_robot(next_position) {
                        break;
                    }
                    position = next_position;

                    let mut next_state = current_state.clone();
                    next_state.robots[robot_index] = position;
                    results.push(next_state);
                }
            }
        }
        results
    }

    pub fn next_states(&self, current_state: &GameState) -> [GameState; ROBOT_COUNT * 4] {
        let mut results = array::from_fn(|_| current_state.clone());
        let mut ptr = 0;
        for robot_index in 0..ROBOT_COUNT {
            for direction in DIRECTIONS {
                let position = current_state.robots[robot_index];
                let wall_steps =
                    self.wall_cache[position.r as usize][position.c as usize][direction as usize];
                let robot_steps = if wall_steps > 0 {
                    current_state.robot_steps(robot_index, direction)
                } else {
                    0
                };
                let steps = min(wall_steps, robot_steps);
                results[ptr].robots[robot_index] = position + Point::from(direction) * steps as i8;
                ptr += 1;
            }
        }
        results
    }

    pub fn is_winning_state(&self, state: &GameState) -> bool {
        match self.target_type {
            TargetType::Any => state
                .robots
                .into_iter()
                .any(|position| position == self.goal),
            TargetType::Particular(robot_index) => state.robots[robot_index] == self.goal,
        }
    }

    pub fn equivalent_states_any(&self, state: &GameState) -> [GameState; factorial(ROBOT_COUNT)] {
        let mut result: [GameState; factorial(ROBOT_COUNT)] = array::from_fn(|_| state.clone());
        let mut state = state.clone();
        for (i, pos) in FOUR_PERMUTATION_SWAPS.into_iter().enumerate() {
            state.robots.swap(pos, pos + 1);
            result[i + 1] = state.clone();
        }
        result
    }

    pub fn equivalent_states_particular(
        &self,
        state: &GameState,
        robot_index: usize,
    ) -> [GameState; factorial(ROBOT_COUNT - 1)] {
        let mut result: [GameState; factorial(ROBOT_COUNT - 1)] = array::from_fn(|_| state.clone());
        let mut state = state.clone();
        for (i, pos) in THREE_PERMUTATION_SWAPS.into_iter().enumerate() {
            state
                .robots
                .swap(skipone(pos, robot_index), skipone(pos + 1, robot_index));
            result[i + 1] = state.clone();
        }
        result
    }

    pub fn equivalent_states(&self, state: &GameState) -> Vec<GameState> {
        match self.target_type {
            TargetType::Any => self.equivalent_states_any(state).to_vec(),
            TargetType::Particular(robot_index) => self
                .equivalent_states_particular(state, robot_index)
                .to_vec(),
        }
    }
}

fn skipone(x: usize, to_skip: usize) -> usize {
    if x < to_skip {
        x
    } else {
        x + 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub robots: [Point; ROBOT_COUNT],
}

fn calc_up_steps(from: Point, to: Point) -> u8 {
    if from.c != to.c || from.r <= to.r {
        BOARD_SIZE as u8
    } else {
        (from.r - to.r - 1) as u8
    }
}

impl GameState {
    fn robot_steps(&self, moving_robot_index: usize, direction: Direction) -> u8 {
        let mut steps = BOARD_SIZE as u8;
        for robot_index in 0..ROBOT_COUNT {
            let steps_candidate = match direction {
                Direction::Up => {
                    calc_up_steps(self.robots[moving_robot_index], self.robots[robot_index])
                }
                Direction::Right => calc_up_steps(
                    self.robots[moving_robot_index].rot(),
                    self.robots[robot_index].rot(),
                ),
                Direction::Down => calc_up_steps(
                    self.robots[moving_robot_index].rot().rot(),
                    self.robots[robot_index].rot().rot(),
                ),
                Direction::Left => calc_up_steps(
                    self.robots[moving_robot_index].rrot(),
                    self.robots[robot_index].rrot(),
                ),
            };
            steps = min(steps, steps_candidate);
        }
        steps
    }

    pub fn to_u32(&self) -> u32 {
        let mut x: u32 = 0;
        for position in self.robots {
            x = x << 8 | (position.r as u32) << 4 | position.c as u32;
        }
        x
    }

    fn has_robot(&self, position: Point) -> bool {
        self.robots
            .into_iter()
            .any(|robot_position| robot_position == position)
    }
}
