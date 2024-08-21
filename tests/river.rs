use std::{fs::File, io::BufReader};

use smallvec::smallvec;

use rand::prelude::*;
use serde::{Deserialize, Serialize};

use poker_abstraction::tables::load;
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

struct River {
    evaluator: Evaluator,

    clusters: Vec<u16>,

    indexer: Indexer,

    nodes: Vec<Node>,
}

impl River {
    pub fn new(path: String) -> Self {
        Self {
            evaluator: Evaluator::new("data/evaluator".to_string()),

            clusters: load(&"tests/data/river-clusters.bin".to_string()),

            indexer: Indexer::new(vec![2, 5]),

            nodes: serde_json::from_reader(BufReader::new(File::open(path).unwrap())).unwrap(),
        }
    }
}

impl Game<Node, State> for River {
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
            .map(|node| if node.x.len() > 0 { 2197 } else { 0 })
            .sum();

        let mut answer = vec![0; n];

        for node in &self.nodes {
            if node.x.len() > 0 {
                for i in 0..2197 {
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
        if node.x.is_empty() {
            panic!("invalid input");
        }

        node.i
            + self.clusters[self
                .indexer
                .index(smallvec![state.cards[node.t], state.board])
                as usize] as usize
    }

    fn display(&self, node: &Node, state: &State) -> String {
        let ranks = [
            "2", "3", "4", "5", "6", "7", "8", "9", "T", "J", "Q", "K", "A",
        ];

        let suits = ["♠", "♦", "♣", "♥"];

        let mut res = String::new();

        let deal = [state.cards[0], state.cards[1], state.board];
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
fn test_size() {
    let game = River::new("tests/data/river-tree.json".to_string());

    let sizes = game.size();

    assert_eq!(sizes.len(), 2197 * 4);
    for size in sizes {
        assert_eq!(size, 2);
    }
}

#[test]
fn test_solve() {
    let game = River::new("tests/data/river-tree.json".to_string());

    let infosets = solve(1000000, 42, &game);

    let mut rng = SmallRng::seed_from_u64(42);
    for _ in 0..50 {
        let index = rng.gen_range(0..game.indexer.count[1]);
        let input = game.indexer.unindex(index, 1);

        let cards = input[0];
        let board = input[1];

        let state = State::from([cards, cards], board);

        let node = game.root();

        println!(
            "({}) {}: [{}] -> evaluation: {}",
            index,
            game.display(node, &state),
            normalize(infosets[game.index(node, &state)].s.clone())
                .into_iter()
                .map(|x| format!("{:.2}", x))
                .collect::<Vec<String>>()
                .join(" "),
            game.evaluator.evaluate(cards | board)
        );
    }
}
