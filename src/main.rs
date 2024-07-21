mod model;
mod serialize;
mod solver;

fn main() {
    let board = "rKHKNKXIrKM--7--_n---m-----__--Vv------Yv-N-B-_-----Ln-L-m--L-RJk---_XRL--R_-X-g1--j-n--NZ07-XXZf----K--N-BL---Vv----7----X----Zk-----Lj--R__---RLX--n-m------_leXKXAXKVeWDn94ytfICCphpWL97KIP4sVqKkqtUi";
    let (spec, state) = serialize::load(&board);

    let result = solver::solve_bfs(&spec, &state);
    println!("{:?}", result);
}

