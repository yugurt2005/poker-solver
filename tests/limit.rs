use std::{fs::File, io::BufReader};

use smallvec::smallvec;

use colored::*;

use rand::prelude::*;
use serde::{Deserialize, Serialize};

use poker_abstraction::tables::{get, load};
use poker_evaluator::Evaluator;
use poker_indexer::Indexer;
use poker_solver::{
    interfaces::game::Game,
    solver::{normalize, solve},
};

const CLUSTERS: [usize; 4] = [169, 2197, 2197, 2197];

#[derive(Serialize, Deserialize)]
pub struct Node {
    pub i: usize,
    pub t: u8,
    pub r: u8,
    pub a: char,
    pub h: String,

    pub s: [i32; 2],

    x: Vec<usize>,
}

#[derive(Clone)]
pub struct State {
    used: u64,
    cards: [u64; 2],
    board: [u64; 4],
}

impl State {
    pub fn new(rng: &mut impl Rng) -> Self {
        let mut state = Self {
            used: 0,
            cards: [rng.gen(), rng.gen()],
            board: [0; 4],
        };

        state.cards[0] = state.gen(rng) | state.gen(rng);
        state.cards[1] = state.gen(rng) | state.gen(rng);

        state.board[1] = state.gen(rng) | state.gen(rng) | state.gen(rng);
        state.board[2] = state.gen(rng) | state.board[1];
        state.board[3] = state.gen(rng) | state.board[2];

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

    pub fn from(cards: [u64; 2], board: [u64; 4]) -> Self {
        Self {
            used: 0,
            cards,
            board,
        }
    }
}

pub struct Mock {
    evaluator: Evaluator,

    cluster_1: Vec<u16>,
    cluster_2: Vec<u16>,
    cluster_3: Vec<u16>,

    indexer_0: Indexer,
    indexer_1: Indexer,
    indexer_2: Indexer,
    indexer_3: Indexer,

    nodes: Vec<Node>,
}

impl Mock {
    pub fn new() -> Self {
        let path = "data/abstraction/".to_string();

        Self {
            evaluator: Evaluator::new("data/evaluator".to_string()),

            cluster_1: load(&(path.clone() + "cluster_1.bin")),
            cluster_2: load(&(path.clone() + "cluster_2.bin")),
            cluster_3: load(&(path.clone() + "cluster_3.bin")),

            indexer_0: Indexer::new(vec![2, 0]),
            indexer_1: Indexer::new(vec![2, 3]),
            indexer_2: Indexer::new(vec![2, 4]),
            indexer_3: Indexer::new(vec![2, 5]),

            nodes: serde_json::from_reader(BufReader::new(
                File::open("tests/data/limit-tree.json").unwrap(),
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
        node.t as usize
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
            .map(|node| {
                if node.x.len() > 0 {
                    CLUSTERS[node.r as usize]
                } else {
                    0
                }
            })
            .sum();

        let mut answer = vec![0; n];

        for node in &self.nodes {
            if node.x.len() > 0 {
                for i in 0..CLUSTERS[node.r as usize] {
                    answer[node.i + i] = node.x.len();
                }
            }
        }

        answer
    }

    fn eval(&self, node: &Node, state: &State) -> f64 {
        let me = (node.t ^ 0) as usize;
        let op = (node.t ^ 1) as usize;

        if node.a == 'f' {
            (node.s[op] * if me == 0 { 1 } else { -1 }) as f64
        } else {
            let me_score = self.evaluator.evaluate(state.cards[0] | state.board[3]);
            let op_score = self.evaluator.evaluate(state.cards[1] | state.board[3]);

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
        let input = smallvec![state.cards[node.t as usize], state.board[node.r as usize]];
        node.i
            + match node.r {
                0 => self.indexer_0.index(input) as usize,
                1 => self.cluster_1[self.indexer_1.index(input) as usize] as usize,
                2 => self.cluster_2[self.indexer_2.index(input) as usize] as usize,
                3 => self.cluster_3[self.indexer_3.index(input) as usize] as usize,
                _ => panic!("not possible"),
            }
    }

    fn display(&self, node: &Node, state: &State) -> String {
        let ranks = [
            "2", "3", "4", "5", "6", "7", "8", "9", "T", "J", "Q", "K", "A",
        ];

        let suits = ["♠", "♦", "♣", "♥"];

        let mut res = String::new();

        let deal = [state.cards[node.t as usize], state.board[node.r as usize]];
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
fn test_solve() {
    let infosets = get(
        &"tests/data/limit-sol.bin".to_string(),
        Box::new(|| solve(10000000, 42, &Mock::new())),
    );

    let game = Mock::new();

    let indexer = Indexer::new(vec![2, 3, 1, 1]);

    let mut node = game.root();
    node = game.play(node, 1);
    node = game.play(node, 1);

    let mut rng = SmallRng::seed_from_u64(42);

    for _ in 0..50 {
        let index = rng.gen_range(0..indexer.count[3]);
        let input = indexer.unindex(index, 3);

        let cards = input[0];
        let board = [
            0,
            input[1],
            input[1] | input[2],
            input[1] | input[2] | input[3],
        ];

        let state = State::from([cards, cards], board);

        let i = game.index(node, &state);

        println!(
            "{}: [{}] ({})",
            game.display(node, &state),
            normalize(infosets[i].s.clone())
                .into_iter()
                .map(|x| format!("{:.2}", x))
                .collect::<Vec<String>>()
                .join(" "),
            infosets[i].c
        );
    }
}

#[test]
fn test_solve_pre() {
    let infosets = get(
        &"tests/data/limit-sol.bin".to_string(),
        Box::new(|| solve(10000000, 42, &Mock::new())),
    );

    let game = Mock::new();

    let mut matrix = vec![vec![vec![0.0; 13]; 13]; 2];

    for a in 0..13 {
        for b in 0..13 {
            let a_card = 1 << a;
            let b_card = 1 << b << if a <= b { 13 } else { 0 };

            let node = game.root();

            let i = game.index(node, &State::from([a_card | b_card; 2], [0; 4]));

            let s = normalize(infosets[i].s.clone());
            matrix[0][a][b] = s[0];
            matrix[1][a][b] = s[1];
        }
    }

    let display = |matrix: &Vec<Vec<f64>>| {
        let mut res = String::new();

        for row in 0..13 {
            for col in 0..13 {
                let s = &format!("{:.2} ", matrix[row][col]);

                if matrix[row][col] > 0.75 {
                    res += &s.green().to_string();
                    continue;
                }

                if matrix[row][col] < 0.25 {
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
    println!("C:\n{}", display(&matrix[0]));
    println!("B:\n{}", display(&matrix[1]));
}
