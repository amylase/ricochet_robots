// https://github.com/kaseken/ricochet_robots/blob/master/lib/domains/board/board_id.dart

use std::array::from_fn;

use crate::model::{
    GameSpec, GameState, Point, TargetType, BOARD_SIZE, ROBOT_COUNT, WALL_MAP_SIZE,
};

const POSITION_LENGTH: usize = 2;

const BASE_ID_START: usize = 0;
const BASE_ID_LENGTH: usize = BOARD_SIZE * BOARD_SIZE;
const NORMAL_GOAL_ID_START: usize = BASE_ID_START + BASE_ID_LENGTH;
const NORMAL_GOAL_ID_LENGTH: usize = ROBOT_COUNT * 4 * POSITION_LENGTH;
const WILD_GOAL_ID_START: usize = NORMAL_GOAL_ID_START + NORMAL_GOAL_ID_LENGTH;
const WILD_GOAL_ID_LENGTH: usize = POSITION_LENGTH;
const ROBOT_ID_START: usize = WILD_GOAL_ID_START + WILD_GOAL_ID_LENGTH;
const ROBOT_ID_LENGTH: usize = ROBOT_COUNT * POSITION_LENGTH;
const GOAL_ID_START: usize = ROBOT_ID_START + ROBOT_ID_LENGTH;
const GOAL_ID_LENGTH: usize = POSITION_LENGTH;
const ID_LENGTH: usize =
    BASE_ID_LENGTH + NORMAL_GOAL_ID_LENGTH + WILD_GOAL_ID_LENGTH + ROBOT_ID_LENGTH + GOAL_ID_LENGTH;

pub fn load(base64: &str) -> (GameSpec, GameState) {
    let base16 = to_ints(base64);
    assert!(base16.len() == ID_LENGTH);

    let base = &base16[BASE_ID_START..(BASE_ID_START + BASE_ID_LENGTH)];
    let normal_goal = &base16[NORMAL_GOAL_ID_START..(NORMAL_GOAL_ID_START + NORMAL_GOAL_ID_LENGTH)];
    let wild_goal = &base16[WILD_GOAL_ID_START..(WILD_GOAL_ID_START + WILD_GOAL_ID_LENGTH)];
    let robot = &base16[ROBOT_ID_START..(ROBOT_ID_START + ROBOT_ID_LENGTH)];
    let goal = &base16[GOAL_ID_START..(GOAL_ID_START + GOAL_ID_LENGTH)];

    // robot color order: RBGY
    let target_type = if goal[1] < ROBOT_COUNT as u8 {
        TargetType::Particular(goal[1] as usize)
    } else {
        TargetType::Any
    };
    let goal = match target_type {
        TargetType::Particular(_) => read_point_from_array(
            normal_goal,
            (goal[0] * ROBOT_COUNT as u8 + goal[1]) as usize,
        ),
        TargetType::Any => read_point_from_array(wild_goal, 0),
    };
    let robots: [Point; 4] = from_fn(|i| read_point_from_array(robot, i));
    let mut walls: [[bool; WALL_MAP_SIZE]; WALL_MAP_SIZE] = [[false; WALL_MAP_SIZE]; WALL_MAP_SIZE];

    for r in 0..BOARD_SIZE {
        for c in 0..BOARD_SIZE {
            let wall_r = r * 2 + 1;
            let wall_c = c * 2 + 1;
            let idx = r * BOARD_SIZE + c;
            let wall_state = base[idx];
            // (msb) LDRU (lsb)
            if wall_state % 2 == 0 {
                walls[wall_r - 1][wall_c] = true;
            }
            if wall_state / 2 % 2 == 0 {
                walls[wall_r][wall_c + 1] = true;
            }
            if wall_state / 4 % 2 == 0 {
                walls[wall_r + 1][wall_c] = true;
            }
            if wall_state / 8 % 2 == 0 {
                walls[wall_r][wall_c - 1] = true;
            }
        }
    }

    let spec = GameSpec::new(walls, goal, target_type);
    let state = GameState { robots };
    (spec, state)
}

pub fn dump(spec: &GameSpec, state: &GameState) -> String {
    let mut base16 = vec![];

    for r in 0..BOARD_SIZE {
        for c in 0..BOARD_SIZE {
            let wall_r = r * 2 + 1;
            let wall_c = c * 2 + 1;
            let mut wall_state = 0;
            // (msb) LDRU (lsb)
            if !spec.walls[wall_r - 1][wall_c] {
                wall_state |= 1 << 0;
            }
            if !spec.walls[wall_r][wall_c + 1] {
                wall_state |= 1 << 1;
            }
            if !spec.walls[wall_r + 1][wall_c] {
                wall_state |= 1 << 2;
            }
            if !spec.walls[wall_r][wall_c - 1] {
                wall_state |= 1 << 3;
            }
            base16.push(wall_state)
        }
    }

    // normal goal + wild goal
    let nongoal = Point::new((spec.goal.r + 1) & 15, (spec.goal.c + 1) & 15);
    for _ in 0..(ROBOT_COUNT * 4 + 1) {
        write_point_to_vec(&mut base16, nongoal);
    }

    for robot_point in state.robots {
        write_point_to_vec(&mut base16, robot_point);
    }

    match spec.target_type {
        TargetType::Any => {
            base16.push(4);
            base16.push(4);
            base16[WILD_GOAL_ID_START] = spec.goal.c as u8;
            base16[WILD_GOAL_ID_START + 1] = spec.goal.r as u8;
        }
        TargetType::Particular(robot_index) => {
            base16.push(0);
            base16.push(robot_index as u8);
            base16[NORMAL_GOAL_ID_START + robot_index * POSITION_LENGTH] = spec.goal.c as u8;
            base16[NORMAL_GOAL_ID_START + robot_index * POSITION_LENGTH + 1] = spec.goal.r as u8;
        }
    }

    assert!(base16.len() == ID_LENGTH);
    to_base64(base16)
}

fn read_point_from_array(arr: &[u8], i: usize) -> Point {
    let ci = i * 2;
    let ri = ci + 1;
    Point::new(arr[ri] as i8, arr[ci] as i8)
}

fn write_point_to_vec(vec: &mut Vec<u8>, point: Point) {
    vec.push(point.c as u8);
    vec.push(point.r as u8);
}

fn base64char_to_int(code: u8) -> u8 {
    if code.is_ascii_digit() {
        code - b'0'
    } else if code.is_ascii_lowercase() {
        code - b'a' + 10
    } else if code.is_ascii_uppercase() {
        code - b'A' + 10 + 26
    } else if code == b'_' {
        10 + 26 + 26
    } else if code == b'-' {
        10 + 26 + 26 + 1
    } else {
        panic!();
    }
}

fn int_to_base64char(int: u8) -> char {
    if int < 10 {
        (b'0' + int) as char
    } else if (10..36).contains(&int) {
        (b'a' + int - 10) as char
    } else if (36..62).contains(&int) {
        (b'A' + int - 36) as char
    } else if int == 62 {
        '_'
    } else if int == 63 {
        '-'
    } else {
        panic!();
    }
}

fn to_ints(base64: &str) -> Vec<u8> {
    assert!(base64.len() % 2 == 0);

    let bytes = base64.as_bytes();
    let mut base16: Vec<u8> = vec![];
    for fr in (0..(base64.len())).step_by(2) {
        let value =
            base64char_to_int(bytes[fr]) as u16 * 64_u16 + base64char_to_int(bytes[fr + 1]) as u16;
        base16.push((value / 256) as u8);
        base16.push((value / 16 % 16) as u8);
        base16.push((value % 16) as u8);
    }
    base16
}

fn to_base64(ints: Vec<u8>) -> String {
    assert!(ints.len() % 3 == 0);

    let mut base64 = String::new();
    for fr in (0..ints.len()).step_by(3) {
        let value = ints[fr] as u16 * 256 + ints[fr + 1] as u16 * 16 + ints[fr + 2] as u16;
        base64.push(int_to_base64char((value / 64) as u8));
        base64.push(int_to_base64char((value % 64) as u8));
    }
    base64
}

pub fn robot_index_to_color(robot_index: u8) -> &'static str {
    match robot_index {
        0 => "Red",
        1 => "Blue",
        2 => "Green",
        3 => "Yellow",
        _ => unreachable!(),
    }
}
