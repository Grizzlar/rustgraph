use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use crate::NodePool;
use crate::event::{Event, to_u32};

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
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32
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
        let _rounds = data.rounds;

        let event = Event::new(data.id, 1, 0, 0, 0, now());
        let mut eguard = data.events.lock().unwrap();
        eguard.push(Arc::new(Mutex::new(event)));
        data.graphs.get(&data.id).unwrap().lock().unwrap().push(Arc::clone(eguard.last().unwrap()));
        drop(eguard);

        let recv = Arc::clone(&data);
        let send = Arc::clone(&data);
        let receiver = thread::spawn(move || send_sync(recv));
        let sender = thread::spawn(move || recv_sync(send));
        receiver.join().unwrap();
        sender.join().unwrap();
    })
}

fn recv_sync(data: Arc<NodeData>) {
    let listener = TcpListener::bind(format!("127.0.0.1:212{}", data.id)).unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        stream.set_read_timeout(Some(Duration::new(1, 0))).unwrap();
        let mut buf = [0; 25];
        let mut event_buff = [0; 21];
        let mut buffered = 0;
        loop {
            if let Ok(read) = stream.read(&mut buf) {
                if read == 0 {
                    break;
                }

                if buf[0] == 90 {
                    println!("{} GOT SYNC REQ", data.id);
                    let gossip_graph = data.graphs.get(&buf[1]).unwrap().lock().unwrap();
                    let ts = match gossip_graph.last() {
                        Some(event) => event.lock().unwrap().id(),
                        _ => 0
                    };
                    stream.write(&ts.to_le_bytes()).unwrap();
                    continue;
                } else if buf[0] == 91 {
                    println!("{} GOT SYNC DONE", data.id);
                    break;
                }
                
                let rem = 21 - buffered;
                let mut i = 0;
                while i < rem && i < read {
                    event_buff[buffered + i] = buf[i];
                    i += 1;
                }
                buffered += i;

                if buffered == 21 {
                    // TODO: Verify data
                    let event = Event::from_bytes(&event_buff[..]);
                    event.visualize();

                    // TODO: HASH
                    if let Some(gossip_events) = data.graphs.get(&event.sender()) {
                        let gossip_parent_hash = event.hash();
                        let event = Arc::new(Mutex::new(event));
                        
                        let mut gossip_events = gossip_events.lock().unwrap();
                        gossip_events.push(Arc::clone(&event));
                        drop(gossip_events);

                        let mut self_events = data.graphs.get(&data.id).unwrap().lock().unwrap();

                        let self_parent = self_events.last().unwrap().lock().unwrap();
                        let self_parent_hash = self_parent.hash();
                        let self_parent_id = self_parent.id();
                        drop(self_parent);

                        let nevent = Arc::new(Mutex::new(Event::new(data.id, self_parent_id + 1, 0, self_parent_hash, gossip_parent_hash, now())));
                        self_events.push(Arc::clone(&nevent));
                        drop(self_events);

                        let mut eguard = data.events.lock().unwrap();  
                        eguard.push(event);
                        eguard.push(nevent);
                        let elock = eguard[eguard.len() - 2].lock().unwrap();
                        let nelock = eguard[eguard.len() - 1].lock().unwrap();
                        println!("{} RECORDED EVENT{} FROM: {} RESULTING IN {}", data.id, elock.id(), elock.sender(), nelock.stringify());
                    }
                }
                buffered = 0;
                while i + buffered < 21 && i + buffered < read {
                    event_buff[buffered] = buf[i + buffered];
                    buffered += 1;
                }
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
                stream.set_read_timeout(Some(Duration::new(1, 0))).unwrap();
                let mut buf = [0; 25];
                stream.write(&[90, data.id]).unwrap(); // 90 = LATEST FROM ME?
                if let Ok(_read) = stream.read(&mut buf) {
                    let id = to_u32(&buf[0..4]);
                    let self_graph = data.graphs.get(&data.id).unwrap().lock().unwrap();
                    for event in self_graph.iter().skip(id as usize) {
                        let event = event.lock().unwrap();
                        if event.id() > id {
                            stream.write(&event.to_bytes()).unwrap();
                        } else {
                            break;
                        }
                    }
                }
                stream.write(&[91]).unwrap(); // 91 = FINISH
            }
            _ => { println!("Connection Error: {}", peer) }
        }
    }
}
