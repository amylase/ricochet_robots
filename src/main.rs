use std::io::{self, BufRead};

mod model;
mod serialize;
mod solvers;

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let board = line.unwrap();
        let (spec, state) = serialize::load(&board);

        let result = solvers::iddfs::solve(&spec, &state);
        println!("found a solution with {} moves", result.len());
        for game_move in result {
            println!("> Move {} to {:?} ", serialize::robot_index_to_color(game_move.robot_index), game_move.direction)
        }
    }
}
