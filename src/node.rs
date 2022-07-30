use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::cell::RefCell;
use crate::NodePool;
use crate::event::Event;

pub struct Node<'a> {
    id: u8,
    thread: Option<JoinHandle<()>>,
    rounds: u8,
    pool: &'a NodePool<'a>
}

pub struct NodeData {
    id: u8,
    rounds: u8,
    peers: HashMap<u8, String>
}

impl <'a> Node<'a> {
    pub fn new(id: u8, rounds: u8, pool: &'a NodePool<'a>) -> Node<'a> {
        Node { id, thread: None, rounds, pool }
    }

    pub fn start(&mut self) {
        let node_count = self.pool.nodes.borrow().len() as u8;

        let mut map = HashMap::new();
        let mut graphs: HashMap<u8, RefCell<Vec<Event>>> = HashMap::new();
        for i in 0..node_count {
            if i != self.id {
                map.insert(i, format!("127.0.0.1:212{}", i));
            }
            graphs.insert(i, RefCell::new(Vec::new()));
        }

        let thread = spawn_tread(self.id, self.rounds, map, graphs);
        self.thread = Some(thread);
    }
}

fn now() -> u32 {
    // TODO: RETURNS u64
    SystemTime::now().elapsed().unwrap().as_secs() as u32
}

fn spawn_tread(id: u8, rounds: u8, peers: HashMap<u8, String>, graphs: HashMap<u8, RefCell<Vec<Event>>>) -> JoinHandle<()> {    
    thread::spawn(move || {
        let graphs = Arc::new(Mutex::new(graphs));
        let data = Arc::new(NodeData {
            id,
            rounds,
            peers
        });

        let mut guard = graphs.lock().unwrap();
        guard.get(&data.id).unwrap().borrow_mut().push(
            Event::new(data.id, 0, 0, 0, now())
        );
        drop(guard);

        let recv = (Arc::clone(&graphs), Arc::clone(&data));
        let send = (Arc::clone(&graphs), Arc::clone(&data));
        let receiver = thread::spawn(move || send_sync(recv));
        let sender = thread::spawn(move || recv_sync(send));
        receiver.join();
        sender.join();
    })
}

fn recv_sync(params: (Arc<Mutex<HashMap<u8, RefCell<Vec<Event>>>>>, Arc<NodeData>)) {
    let graph = params.0;
    let data = params.1;

    let listener = TcpListener::bind(format!("127.0.0.1:212{}", data.id)).unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        stream.set_read_timeout(Some(Duration::new(2000, 0))).unwrap();
        let mut buf = [0; 20];
        if let Ok(_read) = stream.read(&mut buf) {
            // TODO: Verify data
            let event = Event::from_bytes(&buf[..]);
            event.visualize();
            
            // TODO: HASH
            let guard = graph.lock().unwrap();
            let mut self_events = guard.get(&data.id).unwrap().borrow_mut();
            if let Some(gossip_events) = guard.get(&event.sender()) {
                let mut gossip_events = gossip_events.borrow_mut();
                gossip_events.push(event);
                let self_parent_hash = self_events.last().unwrap().hash();
                self_events.push(
                    Event::new(data.id, 0, self_parent_hash, gossip_events.last().unwrap().hash(), now())
                );
            }
        }
    }
}

fn send_sync(params: (Arc<Mutex<HashMap<u8, RefCell<Vec<Event>>>>>, Arc<NodeData>)) {
    loop {

    }
}