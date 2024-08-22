use std::{fs::File, io::BufReader};

use poker_abstraction::tables::get;
use smallvec::smallvec;

use rand::Rng;
use serde::{Deserialize, Serialize};

use colored::*;

use poker_evaluator::Evaluator;
use poker_indexer::Indexer;
use poker_solver::{
    interfaces::game::Game,
    solver::{normalize, solve},
};

#[derive(Serialize, Deserialize)]
struct Node {
    i: usize,
    t: usize,
    a: char,
    h: String,

    s: [i8; 2],

    x: Vec<usize>,
}

#[derive(Clone)]
struct State {
    used: u64,
    cards: [u64; 2],
    board: u64,
}

impl State {
    pub fn new(rng: &mut impl Rng) -> Self {
        let mut state = Self {
            used: 0,
            cards: [0, 0],
            board: 0,
        };

        state.cards[0] = state.gen(rng) | state.gen(rng);
        state.cards[1] = state.gen(rng) | state.gen(rng);

        state.board =
            state.gen(rng) | state.gen(rng) | state.gen(rng) | state.gen(rng) | state.gen(rng);

        state
    }

    fn gen(&mut self, rng: &mut impl Rng) -> u64 {
        loop {
            let x = 1 << rng.gen_range(0..52);

            if self.used & x != 0 {
                continue;
            }
            self.used |= x;

            return x;
        }
    }

    pub fn from(cards: [u64; 2], board: u64) -> Self {
        Self {
            used: 0,
            cards,
            board,
        }
    }
}

struct Mock {
    evaluator: Evaluator,

    indexer: Indexer,

    nodes: Vec<Node>,
}

impl Mock {
    pub fn new() -> Self {
        Self {
            evaluator: Evaluator::new("data/evaluator".to_string()),

            indexer: Indexer::new(vec![2]),

            nodes: serde_json::from_reader(BufReader::new(
                File::open("tests/data/pre-tree.json").unwrap(),
            ))
            .unwrap(),
        }
    }
}

impl Game<Node, State> for Mock {
    fn done(&self, node: &Node) -> bool {
        node.x.is_empty()
    }

    fn turn(&self, node: &Node) -> usize {
        node.t
    }

    fn next(&self, node: &Node) -> usize {
        node.x.len()
    }

    fn init(&self, rng: &mut impl Rng) -> State {
        State::new(rng)
    }

    fn root(&self) -> &Node {
        &self.nodes[0]
    }

    fn size(&self) -> Vec<usize> {
        let n = self
            .nodes
            .iter()
            .map(|node| if node.x.len() > 0 { 169 } else { 0 })
            .sum();

        let mut answer = vec![0; n];

        for node in &self.nodes {
            if node.x.len() > 0 {
                for i in 0..169 {
                    answer[node.i + i] = node.x.len();
                }
            }
        }

        answer
    }

    fn eval(&self, node: &Node, state: &State) -> f64 {
        let me = node.t ^ 0;
        let op = node.t ^ 1;

        if node.a == 'f' {
            (node.s[op] * if me == 0 { 1 } else { -1 }) as f64
        } else {
            let me_score = self.evaluator.evaluate(state.cards[0] | state.board);
            let op_score = self.evaluator.evaluate(state.cards[1] | state.board);

            if me_score < op_score {
                return node.s[op] as f64;
            }

            if me_score > op_score {
                return node.s[me] as f64 * -1.0;
            }

            return 0.0;
        }
    }

    fn play(&self, node: &Node, action: usize) -> &Node {
        &self.nodes[node.x[action]]
    }

    fn index(&self, node: &Node, state: &State) -> usize {
        node.i + self.indexer.index(smallvec![state.cards[node.t]]) as usize
    }

    fn display(&self, node: &Node, state: &State) -> String {
        let ranks = [
            "2", "3", "4", "5", "6", "7", "8", "9", "T", "J", "Q", "K", "A",
        ];

        let suits = ["♠", "♦", "♣", "♥"];

        let mut res = String::new();

        let deal = [state.cards[node.t as usize], state.board];
        for mut cards in deal {
            while cards != 0 {
                let card = 63 - cards.leading_zeros();
                let rank = card % 13;
                let suit = card / 13;

                res += &format!("{}{} ", ranks[rank as usize], suits[suit as usize]);

                cards &= !(1 << card);
            }
            res += "| "
        }

        res += &format!("{} ({})", node.h, node.t);

        res
    }
}

#[test]
fn test_solve_pre() {
    let infosets = get(
        &"tests/data/pre-sol.bin".to_string(),
        Box::new(|| solve(10000000, 42, &Mock::new())),
    );

    let game = Mock::new();

    let node = game.root();
    let node = game.play(node, 3);

    let mut matrix = vec![vec![vec![0.0; 13]; 13]; node.x.len()];
    for a in 0..13 {
        for b in 0..13 {
            let a_card = 1 << a;
            let b_card = 1 << b << if a < b { 0 } else { 13 };

            let i = game.index(node, &State::from([a_card | b_card, a_card | b_card], 0));

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

    for (i, x) in matrix.into_iter().enumerate() {
        println!("{:?}:\n{}", game.play(node, i).s, display(&x));
    }
}
