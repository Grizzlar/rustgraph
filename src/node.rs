use std::io::{Read, Write};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use crate::NodePool;
use crate::event::Event;

pub struct Node<'a> {
    id: u8,
    thread: Option<JoinHandle<()>>,
    events: Vec<Event>,
    rounds: u8,
    pool: &'a NodePool<'a>
}

impl <'a> Node<'a> {
    pub fn new(id: u8, rounds: u8, pool: &'a NodePool<'a>) -> Node<'a> {
        let events = Vec::new();
        Node { id, thread: None, events, rounds, pool }
    }

    pub fn start(&mut self) {
        let node_count = self.pool.nodes.borrow().len() as u8;
        let mut map = HashMap::new();
        for i in 0..node_count {
            if i != self.id {
                map.insert(i, format!("127.0.0.1:212{}", i));
            }
        }
        let id = self.id;

        let thread = thread::spawn(move || loop {
            let listener = TcpListener::bind(format!("127.0.0.1:212{}", id)).unwrap();
            for stream in listener.incoming() {
                let stream = stream.unwrap();
                handle_connection(stream, &map);
            }
        });
        self.thread = Some(thread);
    }
}

fn handle_connection(mut stream: TcpStream, peers: &HashMap<u8, String>) {
    stream.set_read_timeout(Some(Duration::new(2000, 0)));
    let mut buf = [0; 20];
    stream.read(&mut buf);

    if buf.starts_with(b"START") {
        let event = Event::new(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        event.visualize();

        let mut send = TcpStream::connect(peers.values().next().unwrap()).unwrap();
        send.write(&event.to_bytes());
    } else {
        let event = Event::new(&buf[..]);
        event.visualize();
    }

}