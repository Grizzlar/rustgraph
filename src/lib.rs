mod node;
mod event;

use std::thread;
use std::cell::RefCell;
use std::time::Duration;
use crate::node::Node;
pub struct NodePool<'a> {
    nodes: RefCell<Vec<RefCell<Node<'a>>>>
}

impl <'a> NodePool<'a> {
    pub fn new() -> NodePool<'a> {
        let nodes = Vec::new();
        NodePool { nodes: RefCell::new(nodes) }
    }

    pub fn run(&'a self, node_count: u8, coin_round: u8) {
        for id in 0..node_count {
            self.nodes.borrow_mut().push(
                RefCell::new(Node::new(id, coin_round, self))
            );
        }

        for node in self.nodes.borrow().iter() {
            node.borrow_mut().start();
        }

        loop {
            // Keep alive for now
            thread::sleep(Duration::new(60, 0));
        }
    }
}