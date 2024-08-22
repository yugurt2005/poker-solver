use colored::*;

use poker_abstraction::tables::get;
use poker_solver::{
    interfaces::game::Game,
    poker::game::{Poker, State},
    solver::{normalize, solve},
};

#[test]
fn test_poker_solve() {
    let infosets = get(
        &"tests/data/poker-sol.bin".to_string(),
        Box::new(|| solve(50000000, 42, &Poker::new("data/abstraction/".to_string()))),
    );

    let game = Poker::new("data/abstraction/".to_string());

    let node = game.root();
    // let node = game.play(node, 1);
    // let node = game.play(node, 1);

    println!("{}: {}", node.h, node.i);

    let mut matrix = vec![vec![vec![0.0; 13]; 13]; node.c.len()];
    for a in 0..13 {
        for b in 0..13 {
            let a_card = 1 << a;
            let b_card = 1 << b << if a < b { 0 } else { 13 };

            let i = game.index(
                node,
                &State::from([a_card | b_card, a_card | b_card], [0; 4]),
            );

            let s = normalize(infosets[i].s.clone());

            for i in 0..matrix.len() {
                matrix[i][a][b] = s[i];
            }
        }
    }

    let display = |matrix: &Vec<Vec<f64>>| {
        let mut res = String::new();

        for row in 0..13 {
            for col in 0..13 {
                let s = &format!("{:.2} ", matrix[row][col]);

                if matrix[row][col] > 0.5 {
                    res += &s.green().to_string();
                    continue;
                }

                if matrix[row][col] < 0.1 {
                    res += &s.red().to_string();
                    continue;
                }

                res += s;
            }
            res += "\n";
        }

        res
    };

    println!();
    for (i, x) in matrix.into_iter().enumerate() {
        println!("{:?}:\n{}", game.play(node, i).s, display(&x));
    }
}
