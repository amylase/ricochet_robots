use std::io::{self, BufRead};

use ricochet_robots::{serialize, solver};

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let board = line.unwrap().replace("https://kaseken.github.io/ricochet_robots/#/?id=", "");
        let (spec, state) = serialize::load(&board);

        let result = solver::solve_bfs(&spec, &state);
        println!("found a solution with {} moves", result.len());
        for game_move in result {
            println!(
                "> Move {} to {:?} ",
                serialize::robot_index_to_color(game_move.robot_index),
                game_move.direction
            )
        }
    }
}
