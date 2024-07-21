mod model;
mod serialize;
mod solver;

fn main() {
    let board = "rIrGXKX6HKN--Zr---RL-n-----Zf--Zv---B--L--Rj---L----_nXX---Vu--Yf-N-_XXYv-RL---g1----n---Z05f--Zu-Q--G-L-VlYv_--N---Wj---X-----Yr--Zr-L-Q-R-----RL-_-n_n-------ZeWAXKXKVeXD1T1FzbzqyDoFIkrokBuB9G08TAL8j";
    let (spec, state) = serialize::load(&board);

    let result = solver::solve_bfs(&spec, &state);
    println!("found a solution with {} moves", result.len());
    for game_move in result {
        println!("> Move {} to {:?} ", serialize::robot_index_to_color(game_move.robot_index), game_move.direction)
    }
}
