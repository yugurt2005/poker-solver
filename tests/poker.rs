use rand::{rngs::SmallRng, thread_rng, SeedableRng};

use poker_solver::{interfaces::game::Game, poker::game::Poker};

#[test]
fn test_poker_eval() {
    let game = Poker::new("data/abstraction/".to_string());

    let mut rng = SmallRng::seed_from_u64(0);

    let actions = vec![1, 0, 2, 0];

    let mut node = game.root();
    for action in actions {
        node = game.play(node, action);
        println!("{} {:?}", node.h, node.s);
    }

    println!("{}", node.h);

    let actual = game.eval(node, &game.init(&mut rng));
    let expect = 200.0;

    assert_eq!(actual, expect);
}

#[test]
fn test_poker_eval_1() {
    let game = Poker::new("data/abstraction/".to_string());

    /*
    c: 200 + 200 = 400
    c: 200 + 200 = 400
    b: 440 + 200 = 640
    r: 440 + 920 = 1360
    c: 920 + 920 = 1840
    c: 920 + 920 = 1840
    b: 2024 + 920 = 2944
    a: 2024 + 6000 = 8024
    f
     */
    let actions = vec![1, 0, 2, 3, 1, 0, 2, 2, 0];

    let mut node = game.root();
    for action in actions {
        node = game.play(node, action);
        println!("{} {:?}", node.h, node.s);
    }

    println!("{}", node.h);

    let actual = game.eval(node, &game.init(&mut thread_rng()));
    let expect = 200.0;

    assert_eq!(actual, expect);
}

#[test]
fn test_poker_eval_2() {
    let game = Poker::new("data/abstraction/".to_string());

    let mut rng = SmallRng::seed_from_u64(42);

    /*
    c: 200 + 200 = 400
    c: 200 + 200 = 400
    b: 440 + 200 = 640
    r: 440 + 920 = 1360
    c: 920 + 920 = 1840
    c: 920 + 920 = 1840
    b: 2024 + 920 = 2944
    a: 2024 + 6000 = 8024
    c: 6000 + 6000 = 12000
     */
    let actions = vec![1, 0, 2, 3, 1, 0, 2, 2, 1];

    let mut node = game.root();
    for action in actions {
        node = game.play(node, action);
    }

    let state = game.init(&mut rng);

    println!("{}", game.display(node, &state));

    let actual = game.eval(node, &state);
    let expect = 6000.0;

    assert_eq!(actual, expect);
}
