use std::io::Write;

use poker_solver::{poker::Poker, solver::solve};

fn main() {
    let game = Poker::new("data/abstraction/".to_string());

    let start = std::time::Instant::now();

    let data = solve(1000000000, 42, &game);

    println!("Elapsed: {:?}", start.elapsed());

    let buffer = bincode::serialize(&data).unwrap();
    std::fs::File::create("data/solution.bin".to_string())
        .unwrap()
        .write_all(&buffer)
        .unwrap();
}
