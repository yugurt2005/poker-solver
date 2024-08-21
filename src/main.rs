use std::io::{Read, Write};

use poker_indexer::Indexer;
use poker_solver::{
    interfaces::game::Game,
    poker::game::{Poker, State},
    solver::{normalize, solve, Infoset},
};

fn main() {
    let game = Poker::new("data/abstraction/".to_string());

    let start = std::time::Instant::now();

    let data = solve(1000000000, 420, &game);

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
}
