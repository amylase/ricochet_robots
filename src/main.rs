use std::io::{self, BufRead};

mod model;
mod serialize;
mod solver;

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let board = line.unwrap();
        let (spec, state) = serialize::load(&board);

        let result = solver::solve_bfs(&spec, &state);
        println!("found a solution with {} moves", result.len());
        for game_move in result {
            println!("> Move {} to {:?} ", robot_index_to_color(game_move.robot_index), game_move.direction)
        }
        // println!("{:?}", result);    
    }
}

fn robot_index_to_color(robot_index: u8) -> &'static str {
    match robot_index {
        0 => "Red",
        1 => "Blue",
        2 => "Green",
        3 => "Yellow",
        _ => unreachable!(),
    }
}