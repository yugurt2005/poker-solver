use rand::Rng;

pub trait Game<Node, State> {
    fn done(&self, node: &Node) -> bool;

    fn turn(&self, node: &Node) -> usize;

    fn next(&self, node: &Node) -> usize;

    fn init(&self, rng: &mut impl Rng) -> State;

    fn root(&self) -> &Node;

    fn size(&self) -> Vec<usize>;

    fn eval(&self, node: &Node, state: &State) -> f64;

    fn play(&self, node: &Node, action: usize) -> &Node;

    fn index(&self, node: &Node, state: &State) -> usize;

    fn display(&self, node: &Node, state: &State) -> String;
}
