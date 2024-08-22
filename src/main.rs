use std::io::{Read, Write};

use colored::*;

use poker_indexer::Indexer;
use poker_solver::{
    interfaces::game::Game,
    poker::game::{Poker, State},
    solver::{normalize, solve, Infoset},
};

fn main() {
    let game = Poker::new("data/abstraction/".to_string());

    let start = std::time::Instant::now();

    let data = solve(2000000000, 420, &game);

    println!("Elapsed: {:?}", start.elapsed());

    let buffer = bincode::serialize(&data).unwrap();
    std::fs::File::create("data/solution.bin".to_string())
        .unwrap()
        .write_all(&buffer)
        .unwrap();

    // let mut buffer = Vec::new();

    // std::fs::File::open("data/solution.bin".to_string())
    //     .unwrap()
    //     .read_to_end(&mut buffer)
    //     .unwrap();

    // let infosets: Vec<Infoset> = bincode::deserialize(&buffer).unwrap();

    // let mut board = [0, 1 << 6 | 1 << 3 | 1 << 34, 1 << 8, 1 << 39];
    // for i in 1..4 {
    //     board[i] |= board[i - 1];
    // }

    // let state = State::from([1 << 47 | 1 << 21, 1 << 12 | 1 << 1], board);

    // let actions = vec![1, 0, 0, 0, 2, 1, 2, 2];

    // let mut node = game.root();

    // for i in 0..actions.len() + 1 {
    //     let index = game.index(node, &state);
    //     let infoset = &infosets[index];

    //     println!(
    //         "{}: [{:?}]",
    //         game.display(node, &state),
    //         normalize(infoset.s.clone())
    //             .into_iter()
    //             .map(|x| format!("{:.2}", x))
    //             .collect::<Vec<String>>()
    //             .join(" ")
    //     );

    //     if i == actions.len() {
    //         break;
    //     }

    //     let action = actions[i];

    //     node = game.play(node, action);
    // }

    // let indexer = Indexer::new(vec![2, 3]);

    // let mut node = game.root();
    // node = game.play(node, 1);
    // node = game.play(node, 2);
    // // node = game.play(node, 2);
    // // node = game.play(node, 2);

    // for i in 0..169 {
    //     let cards = indexer.unindex(i, 0);

    //     let state = State::from([cards[0], cards[0]], [0, 0, 0, 0]);

    //     let infoset = &infosets[game.index(node, &state)];

    //     println!(
    //         "{}: [{:?}]",
    //         game.display(node, &State::from([cards[0], 0], [0, 0, 0, 0])),
    //         normalize(infoset.s.clone())
    //             .into_iter()
    //             .map(|x| format!("{:.2}", x))
    //             .collect::<Vec<String>>()
    //             .join(" ")
    //     );
    // }

    // let node = game.root();
    // let node = game.play(node, 3);
    // let node = game.play(node, 3);
    // let node = game.play(node, 3);
    // let node = game.play(node, 1);

    // println!("{}: {} {:?}", node.h, node.r, node.s);

    // let mut board = [0, 1 << 6 | 1 << 3 | 1 << 34, 1 << 8, 1 << 39];
    // for i in 1..4 {
    //     board[i] |= board[i - 1];
    // }


    // let mut matrix = vec![vec![vec![0.0; 13]; 13]; node.children.len()];
    // for a in 0..13 {
    //     for b in 0..13 {
    //         let a_card = 1 << a;
    //         let b_card = 1 << b << if a < b { 0 } else { 13 };

    //         let i = game.index(node, &State::from([a_card | b_card, a_card | b_card], board));

    //         let s = normalize(infosets[i].s.clone());

    //         for i in 0..matrix.len() {
    //             matrix[i][a][b] = s[i];
    //         }
    //     }
    // }

    // let display = |matrix: &Vec<Vec<f64>>| {
    //     let mut res = String::new();

    //     for row in 0..13 {
    //         for col in 0..13 {
    //             let s = &format!("{:.2} ", matrix[row][col]);

    //             if matrix[row][col] > 0.5 {
    //                 res += &s.green().to_string();
    //                 continue;
    //             }

    //             if matrix[row][col] < 0.1 {
    //                 res += &s.red().to_string();
    //                 continue;
    //             }

    //             res += s;
    //         }
    //         res += "\n";
    //     }

    //     res
    // };

    // println!("{}", game.display(node, &State::from([0, 0], board)));

    // for (i, x) in matrix.into_iter().enumerate() {
    //     println!("{:?}:\n{}", game.play(node, i).s, display(&x));
    // }
}
