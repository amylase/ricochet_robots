pub const fn factorial(n: usize) -> usize {
    if n == 0 {
        1
    } else {
        n * factorial(n - 1)
    }
}

pub fn permutation_swaps(n: usize) -> Vec<usize> {
    // https://en.wikipedia.org/wiki/Steinhaus%E2%80%93Johnson%E2%80%93Trotter_algorithm#Recursive_structure
    let perms = permutations(n);
    let mut swap_position = vec![];
    for i in 0..(perms.len() - 1) {
        for pos in 0..perms[i].len() {
            if perms[i][pos] != perms[i + 1][pos] {
                swap_position.push(pos);
                break;
            }
        }
    }
    return swap_position;
}

fn permutations(n: usize) -> Vec<Vec<usize>> {
    if n == 0 {
        return vec![vec![]];
    }
    permutations(n - 1)
        .into_iter()
        .enumerate()
        .flat_map(|(i, sub)| {
            let positions: Vec<usize> = if i % 2 == 1 {
                (0..=(n - 1)).collect()
            } else {
                (0..=(n - 1)).rev().collect()
            };
            positions.into_iter().map(move |position| {
                let mut perm = sub.clone();
                perm.insert(position, n);
                perm
            })
        })
        .collect()
}
