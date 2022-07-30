use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
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
    peers: HashMap<u8, String>,
    graphs: HashMap<u8, Mutex<Vec<Arc<Mutex<Event>>>>>,
    events: Mutex<Vec<Arc<Mutex<Event>>>>
}

impl <'a> Node<'a> {
    pub fn new(id: u8, rounds: u8, pool: &'a NodePool<'a>) -> Node<'a> {
        Node { id, thread: None, rounds, pool }
    }

    pub fn start(&mut self) {
        let node_count = self.pool.nodes.borrow().len() as u8;

        let mut map = HashMap::new();
        let mut graphs: HashMap<u8, Mutex<Vec<Arc<Mutex<Event>>>>> = HashMap::new();
        let events: Vec<Arc<Mutex<Event>>> = Vec::new();
        for i in 0..node_count {
            if i != self.id {
                map.insert(i, format!("127.0.0.1:212{}", i));
            }
            graphs.insert(i, Mutex::new(Vec::new()));
        }

        let thread = spawn_tread(self.id, self.rounds, map, graphs, events);
        self.thread = Some(thread);
    }
}

fn now() -> u32 {
    // TODO: RETURNS u64
    SystemTime::now().elapsed().unwrap().as_secs() as u32
}

fn spawn_tread(id: u8, rounds: u8, peers: HashMap<u8, String>, graphs: HashMap<u8, Mutex<Vec<Arc<Mutex<Event>>>>>, events: Vec<Arc<Mutex<Event>>>) -> JoinHandle<()> {    
    thread::spawn(move || {
        let events = Mutex::new(events);
        let data = Arc::new(NodeData {
            id,
            rounds,
            peers,
            graphs,
            events
        });

        let event = Event::new(data.id, 0, 0, 0, now());
        let mut eguard = data.events.lock().unwrap();
        eguard.push(Arc::new(Mutex::new(event)));
        data.graphs.get(&data.id).unwrap().lock().unwrap().push(Arc::clone(eguard.last().unwrap()));
        drop(eguard);

        let recv = Arc::clone(&data);
        let send = Arc::clone(&data);
        let receiver = thread::spawn(move || send_sync(recv));
        let sender = thread::spawn(move || recv_sync(send));
        receiver.join();
        sender.join();
    })
}

fn recv_sync(data: Arc<NodeData>) {
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
            let mut eguard = data.events.lock().unwrap();
            let mut self_events = data.graphs.get(&data.id).unwrap().lock().unwrap();
            if let Some(gossip_events) = data.graphs.get(&event.sender()) {
                let mut gossip_events = gossip_events.lock().unwrap();
                let gossip_parent_hash = event.hash();
                let event = Mutex::new(event);
                eguard.push(Arc::new(event));
                gossip_events.push(Arc::clone(eguard.last().unwrap()));

                let self_parent_hash = self_events.last().unwrap().lock().unwrap().hash();
                let event = Mutex::new(Event::new(data.id, 0, self_parent_hash, gossip_parent_hash, now()));
                eguard.push(Arc::new(event));
                self_events.push(Arc::clone(eguard.last().unwrap()));
            }
        }
    }
}

fn send_sync(data: Arc<NodeData>) {
    let mut keys = data.peers.keys().cycle();
    while let Some(key) = keys.next() {
        let peer = data.peers.get(&key).unwrap();
        match TcpStream::connect(&peer) {
            Ok(mut stream) => {
                stream.write(&[64 + data.id, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4]);
            }
            _ => { println!("Connection Error: {}", peer) }
        }
    }
}
