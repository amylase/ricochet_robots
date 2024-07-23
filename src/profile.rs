mod model;
mod serialize;
mod solver;

fn main() {
    let board = "rIrKXKX6XKN--Zf----j-n---L_n--XZv-B-__----A--L-m----L6------_-RJv-_-_X-Yv-RVvYvg1----nX--Z07LVvZv----KRLX-B-B-Q-_----7-L-_-Yv--Zf-_-----Q-RL-YvX---_-n---Zr----ZeXAXKXKjKXBdc_nswFJGt55_ObqFQksG5Mu0_7YO";
    let (spec, state) = serialize::load(&board);

    let result = solver::solve_bfs(&spec, &state);
    println!("found a solution with {} moves", result.len());
    for game_move in result {
        println!("> Move {} to {:?} ", serialize::robot_index_to_color(game_move.robot_index), game_move.direction)
    }
}
