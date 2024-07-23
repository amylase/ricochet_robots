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
    r: i8,
    c: i8,
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
    walls: WallBoard,
    goal: Point,
    target_type: TargetType,

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

    pub fn next_states(&self, current_state: &GameState) -> Vec<(GameMove, GameState)> {
        let mut results = Vec::with_capacity(ROBOT_COUNT * 4 - 1);
        for robot_index in 0..ROBOT_COUNT {
            for direction in Direction::iter() {
                let position = current_state.robots[robot_index]; 
                let steps: u8 = min(
                    self.wall_cache[position.r as usize][position.c as usize][direction as usize], 
                    current_state.robot_steps(robot_index, direction)
                );
                if steps > 0 {
                    let mut next_state = current_state.clone();
                    next_state.robots[robot_index] = position + Point::from(direction) * steps as i8;
                    results.push((GameMove { robot_index: robot_index as u8, direction } , next_state));
                }
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
            if robot_index == moving_robot_index {
                continue;
            }
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