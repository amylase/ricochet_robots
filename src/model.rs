use std::ops;
use std::convert::From;
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

impl ops::Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        return Point { r: self.r + rhs.r, c: self.c + rhs.c };
    }
}

#[derive(Debug, Clone, Copy, Default, EnumIter)]
pub enum Direction {
    #[default] Up,
    Down,
    Left,
    Right,
}

impl From<Direction> for Point {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Up => Point { r: -1, c: 0 },
            Direction::Down => Point { r: 1, c: 0 },
            Direction::Left => Point { r: 0, c: -1 },
            Direction::Right => Point { r: 0, c: 1 },
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

#[derive(Debug)]
pub struct GameSpec {
    pub walls: [[bool; WALL_MAP_SIZE]; WALL_MAP_SIZE],
    pub goal: Point,
    pub target_type: TargetType,
}

impl GameSpec {
    fn has_wall(&self, position: Point, direction: Direction) -> bool {
        let wall_position = Point {
            r: position.r * 2 + 1,
            c: position.c * 2 + 1,
        } + Point::from(direction);
        return self.walls[wall_position.r as usize][wall_position.c as usize];
    }

    pub fn next_states(&self, current_state: &GameState) -> Vec<(GameMove, GameState)> {
        let mut results = vec![];
        for robot_index in 0..ROBOT_COUNT {
            for direction in Direction::iter() {
                let mut steps: i32 = 0;
                let mut position = current_state.robots[robot_index];
                loop {
                    if self.has_wall(position, direction) {
                        break;
                    }
                    let next_position = position + Point::from(direction);
                    if current_state.has_robot(next_position) {
                        break;
                    }
                    position = next_position;
                    steps += 1;
                }
                if steps > 0 {
                    let mut next_state = current_state.clone();
                    next_state.robots[robot_index] = position;
                    results.push((GameMove { robot_index: robot_index as u8, direction } , next_state));
                }
            }
        }
        return results;
    }

    pub fn is_winning_state(&self, state: &GameState) -> bool {
        for robot_index in 0..ROBOT_COUNT {
            if state.robots[robot_index] == self.goal {
                if self.target_type == TargetType::Any || self.target_type == TargetType::Particular(robot_index) {
                    return true;
                }
            }
        }
        return false;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameState {
    pub robots: [Point; ROBOT_COUNT],
}

impl GameState {
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