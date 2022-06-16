#[derive(Clone, Copy)]
pub struct Node {
    symbol: char,
    x: i32,
    y: i32
}

#[derive(Clone, Copy)]
pub struct Edge {
    node_from: Node,
    node_to: Node,
    weight: i32
}

pub struct _Graph{
    nodes: Vec<Node>,
    edges: Vec<Edge>
}

impl Node {
    pub fn new(symbol: char, x: i32, y: i32) -> Self {
        Self { symbol, x, y }
    }
    pub fn show_details(self) {
        println!("Co-ordinates - X: {} Y: {}\nVisited: false\n", self.x, self.y)
    }
}

impl Edge {
    pub fn new(node_from: Node, node_to: Node, weight: i32) -> Self {
        Self { node_from, node_to, weight }
    }
    pub fn show_details(self) {
        println!("Nodes: {} -> {}\nWeight: {}\n", 
            self.node_from.symbol, self.node_to.symbol, self.weight)
    }
}

