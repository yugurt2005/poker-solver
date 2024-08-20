use rand::prelude::*;
use serde::{Deserialize, Serialize};
use smallvec::smallvec;
use std::{fs::File, io::BufReader};

use poker_abstraction::tables::load;
use poker_evaluator::Evaluator;
use poker_indexer::Indexer;

use crate::interfaces::game::Game;

const CLUSTERS: [usize; 4] = [169, 2197, 2197, 2197];

#[derive(Serialize, Deserialize)]
pub struct Node {
    pub i: usize,
    pub t: u8,
    pub r: u8,
    pub a: char,
    pub h: String,

    pub s: [i32; 2],

    children: Vec<usize>,
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

pub struct Poker {
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

impl Poker {
    pub fn new(path: String) -> Self {
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
                File::open(path + "action-tree.json").unwrap(),
            ))
            .unwrap(),
        }
    }
}

impl Game<Node, State> for Poker {
    fn done(&self, node: &Node) -> bool {
        node.children.is_empty()
    }

    fn turn(&self, node: &Node) -> usize {
        node.t as usize
    }

    fn next(&self, node: &Node) -> usize {
        node.children.len()
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
                if node.children.len() > 0 {
                    CLUSTERS[node.r as usize]
                } else {
                    0
                }
            })
            .sum();

        let mut answer = vec![0; n];

        for node in &self.nodes {
            if node.children.len() > 0 {
                for i in 0..CLUSTERS[node.r as usize] {
                    answer[node.i + i] = node.children.len();
                }
            }
        }

        answer
    }

    fn eval(&self, node: &Node, state: &State) -> f64 {
        let me = (node.t ^ 0) as usize;
        let op = (node.t ^ 1) as usize;

        if node.a == 'f' {
            node.s[op] as f64
        } else {
            let me_score = self.evaluator.evaluate(state.cards[me] | state.board[3]);
            let op_score = self.evaluator.evaluate(state.cards[op] | state.board[3]);

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
        &self.nodes[node.children[action]]
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

        let deal = [state.cards[0], state.cards[1], state.board[node.r as usize]];
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
