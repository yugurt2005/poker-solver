use std::io::Write;

use poker_solver::{poker::Poker, solver::solve};

fn main() {
    let game = Poker::new("data/abstraction/".to_string());

    let data = solve(10000000, 42, &game);

    let buffer = bincode::serialize(&data).unwrap();
    std::fs::File::create("data/solution.bin".to_string())
        .unwrap()
        .write_all(&buffer)
        .unwrap();
}
