use std::cmp::min;
use std::ops;
use std::convert::From;
use std::hash::Hash;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

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
        Point {r, c}
    }

    fn rot(&self) -> Point {
        // (1, 2) -> (-2, 1)
        Point {r: -self.c, c: self.r}
    }

    fn rrot(&self) -> Point {
        // (-2, 1) -> (1, 2) 
        Point {r: self.c, c: -self.r}
    }
}

impl ops::Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        return Point { r: self.r + rhs.r, c: self.c + rhs.c };
    }
}

impl ops::Mul<i8> for Point {
    type Output = Point;

    fn mul(self, rhs: i8) -> Self::Output {
        return Point { r: self.r * rhs, c: self.c * rhs };
    }
}

#[derive(Debug, Clone, Copy, Default, EnumIter)]
pub enum Direction {
    #[default] Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

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

#[derive(Debug, PartialEq, Eq)]
pub enum TargetType {
    Any,
    Particular(usize),
}

#[derive(Debug, Default, Clone)]
pub struct GameMove {
    pub robot_index: u8,
    pub direction: Direction,
}

type WallBoard = [[bool; WALL_MAP_SIZE]; WALL_MAP_SIZE];
type WallCache = [[[u8; 4]; BOARD_SIZE]; BOARD_SIZE];
#[derive(Debug)]
pub struct GameSpec {
    pub walls: WallBoard,
    pub goal: Point,
    pub target_type: TargetType,

    wall_cache: WallCache,
}

fn _has_wall(walls: &WallBoard, position: Point, direction: Direction) -> bool {
    let wall_position = position * 2 + Point::new(1, 1) + Point::from(direction);
    return walls[wall_position.r as usize][wall_position.c as usize];
}

impl GameSpec {
    pub fn new(walls: WallBoard, goal: Point, target_type: TargetType) -> GameSpec {
        let mut wall_cache = [[[0; 4]; BOARD_SIZE]; BOARD_SIZE];
        for r in 0..BOARD_SIZE {
            for c in 0..BOARD_SIZE {
                for direction in Direction::iter() {
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
                    wall_cache[r][c][direction as usize] = steps;
                }
            }
        }

        return GameSpec { walls, goal, target_type, wall_cache }
    }

    pub fn prev_states(&self, current_state: &GameState) -> Vec<GameState> {
        let mut results = Vec::new();
        for robot_index in 0..ROBOT_COUNT {
            for back_direction in Direction::iter() {
                let direction = back_direction.reverse();
                let mut position = current_state.robots[robot_index];
                if !(_has_wall(&self.walls, position, direction) || current_state.has_robot(position + Point::from(direction))) {
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
        return results;
    }

    pub fn next_states(&self, current_state: &GameState) -> Vec<(GameMove, GameState)> {
        let mut results: Vec<(GameMove, GameState)> = vec![(GameMove { robot_index: 0, direction: Direction::Up}, current_state.clone()); ROBOT_COUNT * 4];
        let mut ptr = 0;
        for robot_index in 0..ROBOT_COUNT {
            for direction in Direction::iter() {
                let position = current_state.robots[robot_index]; 
                let wall_steps = self.wall_cache[position.r as usize][position.c as usize][direction as usize];
                let robot_steps = if wall_steps > 0 { current_state.robot_steps(robot_index, direction) } else { 0 };
                let steps = min(wall_steps, robot_steps);
                results[ptr].0 = GameMove { robot_index: robot_index as u8, direction };
                results[ptr].1.robots[robot_index] = position + Point::from(direction) * steps as i8;
                ptr += 1;
            }
        }
        return results;
    }

    pub fn is_winning_state(&self, state: &GameState) -> bool {
        match self.target_type {
            TargetType::Any => state.robots.into_iter().any(|position| { position == self.goal }),
            TargetType::Particular(robot_index) => state.robots[robot_index] == self.goal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub robots: [Point; ROBOT_COUNT],
}

impl Hash for GameState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.to_u32());
    }
}

fn calc_up_steps(from: Point, to: Point) -> u8 {
    if from.c != to.c {
        return BOARD_SIZE as u8;
    } else if from.r <= to.r {
        return BOARD_SIZE as u8;
    } else {
        return (from.r - to.r - 1) as u8;
    }
}

impl GameState {
    fn robot_steps(&self, moving_robot_index: usize, direction: Direction) -> u8 {
        let mut steps = BOARD_SIZE as u8;
        for robot_index in 0..ROBOT_COUNT {
            let steps_candidate = match direction {
                Direction::Up => calc_up_steps(self.robots[moving_robot_index], self.robots[robot_index]),
                Direction::Right => calc_up_steps(self.robots[moving_robot_index].rot(), self.robots[robot_index].rot()),
                Direction::Down => calc_up_steps(self.robots[moving_robot_index].rot().rot(), self.robots[robot_index].rot().rot()),
                Direction::Left => calc_up_steps(self.robots[moving_robot_index].rrot(), self.robots[robot_index].rrot()),
            };
            steps = min(steps, steps_candidate);
        }
        return steps;
    }

    pub fn to_u32(&self) -> u32 {
        let mut x: u32 = 0;
        for position in self.robots {
            x = x << 8 | (position.r as u32) << 4 | position.c as u32;
        }
        x
    }

    fn has_robot(&self, position: Point) -> bool {
        self.robots.into_iter().any(|robot_position| { robot_position == position })
    }
}

#[allow(dead_code)]
pub fn visualize_walls(spec: &GameSpec) {
    for r in 0..WALL_MAP_SIZE {
        for c in 0..WALL_MAP_SIZE {
            print!("{}", if spec.walls[r][c] || (r % 2 == 0 && c % 2 == 0) { '#' } else { ' ' })
        }
        println!("");
    }
}