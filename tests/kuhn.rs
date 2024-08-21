use std::{fs::File, io::BufReader};

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

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
    cards: [u8; 2],
}

impl State {
    pub fn new(rng: &mut impl Rng) -> Self {
        let a = rng.gen_range(0..3);
        let b = rng.gen_range(0..2);

        let b = (a + b + 1) % 3;

        Self { cards: [a, b] }
    }
}

struct Kuhn {
    nodes: Vec<Node>,
}

impl Kuhn {
    pub fn new(path: String) -> Self {
        Self {
            nodes: serde_json::from_reader(BufReader::new(File::open(path).unwrap())).unwrap(),
        }
    }
}

impl Game<Node, State> for Kuhn {
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
            .map(|node| if node.x.len() > 0 { 3 } else { 0 })
            .sum();

        let mut answer = vec![0; n];

        for node in &self.nodes {
            if node.x.len() > 0 {
                for i in 0..3 {
                    answer[node.i + i] = node.x.len();
                }
            }
        }

        answer
    }

    fn eval(&self, node: &Node, state: &State) -> f64 {
        let me = node.t ^ 0;
        let op = node.t ^ 1;

        (if node.a == 'f' {
            node.s[op] * if me == 0 { 1 } else { -1 }
        } else {
            // showdown
            if state.cards[0] > state.cards[1] {
                node.s[op]
            } else {
                -node.s[me]
            }
        }) as f64
    }

    fn play(&self, node: &Node, action: usize) -> &Node {
        &self.nodes[node.x[action]]
    }

    fn index(&self, node: &Node, state: &State) -> usize {
        node.i + state.cards[node.t] as usize
    }

    fn display(&self, node: &Node, state: &State) -> String {
        format!(
            "{}) {} - {}",
            node.t,
            ["J", "Q", "K"][state.cards[node.t] as usize],
            node.h
        )
        // format!("{:?} - {}", state.cards, node.h)
    }
}

fn display(mut index: usize, kuhn: &Kuhn) -> String {
    let x = index % 3;

    index -= x;

    for node in &kuhn.nodes {
        if node.x.len() != 0 && node.i == index {
            return format!("{}) {} - {}", node.t, ["J", "Q", "K"][x], node.h);
        }
    }

    panic!("invalid index");
}

#[test]
fn test_khun_tree() {
    let game = Kuhn::new("tests/data/kuhn-tree.json".to_string());

    let states = vec![
        State { cards: [0, 0] },
        State { cards: [1, 1] },
        State { cards: [2, 2] },
    ];

    let mut used = vec![0; 12];

    for node in &game.nodes {
        if node.x.len() != 0 {
            for state in &states {
                used[game.index(node, state)] += 1;
            }
        }
    }

    for i in 0..12 {
        assert_eq!(used[i], 1);
    }
}

#[test]
fn test_khun_size() {
    let game = Kuhn::new("tests/data/kuhn-tree.json".to_string());

    let sizes = game.size();

    assert_eq!(sizes.len(), 12);
    for size in sizes {
        assert_eq!(size, 2);
    }
}

#[test]
fn test_khun_solve() {
    let game = Kuhn::new("tests/data/kuhn-tree.json".to_string());

    let infosets = solve(100000, 42, &game);

    for i in 0..infosets.len() {
        let infoset = infosets[i].clone();

        println!(
            "{}: [{}]",
            display(i, &game),
            normalize(infoset.s)
                .into_iter()
                .map(|x| format!("{:.2}", x))
                .collect::<Vec<String>>()
                .join(" ")
        );
    }
}
