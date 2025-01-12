use ricochet_robots::{serialize, solver};

fn main() {
    let board = "rKXKNKXKX6N----------n-------X-Zv--X--B-N-A---N-LL_--6----RL-YvZf-LVuX----RL-7Hg1-_--n--RJ07-m-Vv-B-LK--L-N-_L-7---7-lfm-------ZvL--------R-B--Zf----n-L---L--_leXKXAXKVeWD3RdjkRdjkRdjkRdjkRdjkR3Mkd5E0";
    let (spec, state) = serialize::load(board);

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
