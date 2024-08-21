use std::sync::Mutex;

use rand::prelude::*;
use rayon::prelude::*;

use serde::{Deserialize, Serialize};

use crate::interfaces::game::Game;

pub fn normalize(input: Vec<f64>) -> Vec<f64> {
    let sum: f64 = input.iter().sum();

    if sum == 0.0 {
        return vec![1.0 / input.len() as f64; input.len()];
    }

    input.into_iter().map(|x| x / sum).collect()
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Infoset {
    pub n: usize,
    pub s: Vec<f64>,
    pub r: Vec<f64>,
}

impl Infoset {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            s: vec![0.0; n],
            r: vec![0.0; n],
        }
    }

    pub fn get_strategy(&self) -> Vec<f64> {
        normalize(
            self.r
                .iter()
                .map(|&r| if r < 0.0 { 0.0 } else { r })
                .collect(),
        )
    }

    pub fn use_strategy(&mut self) -> Vec<f64> {
        let strategy = self.get_strategy();

        for i in 0..strategy.len() {
            self.s[i] += strategy[i];
        }

        strategy
    }

    pub fn update_regret(&mut self, action: usize, regret: f64) {
        self.r[action] += regret;
    }
}

fn mccfr<Node, State>(
    player: usize,
    state: &State,
    node: &Node,
    game: &impl Game<Node, State>,
    infosets: &Vec<Mutex<Infoset>>,
    rng: &mut impl Rng,
) -> f64 {
    // println!("{}", game.display(node, &state));

    if game.done(node) {
        return game.eval(node, &state);
    }

    if game.turn(node) == player {
        let n = game.next(node);

        let mut u = Vec::with_capacity(n);
        for i in 0..n {
            u.push(mccfr(
                player,
                state,
                game.play(node, i),
                game,
                infosets,
                rng,
            ));
        }

        let infoset = &mut infosets[game.index(node, &state)].lock().unwrap();

        let s = u
            .iter()
            .zip(infoset.get_strategy())
            .fold(0.0, |acc, (x, p)| acc + x * p);

        let who = if game.turn(node) == 0 { 1.0 } else { -1.0 };

        for i in 0..n {
            infoset.update_regret(i, (u[i] - s) * who);
        }

        s
    } else {
        let action = rand::distributions::WeightedIndex::new(
            (&mut infosets[game.index(node, &state)].lock().unwrap()).use_strategy(),
        )
        .unwrap()
        .sample(rng);

        mccfr(player, state, game.play(node, action), game, infosets, rng)
    }
}

pub fn solve<Node: Sync + Send, State: Sync + Send + Clone>(
    n: u64,
    seed: u64,
    game: &(impl Game<Node, State> + Send + Sync),
) -> Vec<Infoset> {
    let infosets = game
        .size()
        .into_iter()
        .map(|size| Mutex::new(Infoset::new(size)))
        .collect::<Vec<_>>();

    let scores: [f64; 2] = (0..n)
        .into_par_iter()
        .map(|i| {
            let mut rng = SmallRng::seed_from_u64(seed + i);

            let mut scores = [0.0; 2];
            scores[0] += mccfr(
                0,
                &game.init(&mut rng),
                game.root(),
                game,
                &infosets,
                &mut rng,
            );
            scores[1] -= mccfr(
                1,
                &game.init(&mut rng),
                game.root(),
                game,
                &infosets,
                &mut rng,
            );

            scores
        })
        .reduce(|| [0.0; 2], |acc, ele| [acc[0] + ele[0], acc[1] + ele[1]]);

    println!("player 0: {}", scores[0] / n as f64);
    println!("player 1: {}", scores[1] / n as f64);

    infosets
        .into_iter()
        .map(|infoset| infoset.into_inner().unwrap())
        .collect()
}
