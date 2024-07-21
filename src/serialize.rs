// https://github.com/kaseken/ricochet_robots/blob/master/lib/domains/board/board_id.dart

use std::array::from_fn;

use crate::model::{GameSpec, GameState, Point, TargetType, BOARD_SIZE, ROBOT_COUNT, WALL_MAP_SIZE};


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
const ID_LENGTH: usize = BASE_ID_LENGTH + NORMAL_GOAL_ID_LENGTH + WILD_GOAL_ID_LENGTH + ROBOT_ID_LENGTH + GOAL_ID_LENGTH;


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
        TargetType::Particular(_) => read_point_from_array(normal_goal, (goal[0] * ROBOT_COUNT as u8 + goal[1]) as usize),
        TargetType::Any => read_point_from_array(wild_goal, 0),
    };
    let robots: [Point; 4] = from_fn(|i| { read_point_from_array(robot, i) }); 
    let mut walls: [[bool; WALL_MAP_SIZE]; WALL_MAP_SIZE] = [[false; WALL_MAP_SIZE]; WALL_MAP_SIZE];

    for r in 0..BOARD_SIZE {
        for c in 0..BOARD_SIZE {
            let wall_r = r * 2 + 1;
            let wall_c = c * 2 + 1;
            let idx = r * BOARD_SIZE + c;
            let wall_state = base[idx];
            // (msb) LDRU (lsb)
            if wall_state / 1 % 2 == 0 {
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
    return (spec, state);
}

fn read_point_from_array(arr: &[u8], i: usize) -> Point {
    let ci = i * 2;
    let ri = ci + 1;
    return Point::new(arr[ri] as i8, arr[ci] as i8);
}

fn base64char_to_index(code: u8) -> u8 {
    if '0' as u8 <= code && code <= '9' as u8 {
        return code - ('0' as u8);
    } else if 'a' as u8 <= code && code <= 'z' as u8 {
        return code - ('a' as u8) + 10;
    } else if 'A' as u8 <= code && code <= 'Z' as u8 {
        return code - ('A' as u8) + 10 + 26;
    } else if code == '_' as u8 {
        return 10 + 26 + 26;
    } else if code == '-' as u8 {
        return 10 + 26 + 26 + 1;
    } else {
        panic!();
    }
}

fn to_ints(base64: &str) -> Vec<u8> {
    assert!(base64.len() % 2 == 0);

    let bytes = base64.as_bytes();
    let mut base16: Vec<u8> = vec![];
    for fr in (0..(base64.len())).step_by(2) {
        let value = base64char_to_index(bytes[fr]) as u16 * (64 as u16) + base64char_to_index(bytes[fr + 1]) as u16;
        base16.push((value / 256) as u8);
        base16.push((value / 16 % 16) as u8);
        base16.push((value % 16) as u8);
    }
    return base16;
}