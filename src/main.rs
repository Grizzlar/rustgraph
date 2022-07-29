use std::process::exit;
use std::env::args;
use rustgraph::NodePool;

fn main() {
    if args().len() < 2 {
        println!("usage: {} <NODE_COUNT> <COUNT_ROUND optional>", args().next().unwrap());
        exit(1);
    }
    let mut arg = args().skip(1);
    let node_count: u8 = if let Ok(nodes) = arg.next().unwrap().trim().parse() { nodes } else {
        panic!("NODE_COUNT has to be convertible to u8");
    };
    let coin_round = if let Some(coin) = arg.next() {
        let coin: u8 = if let Ok(coin) = coin.trim().parse() { coin } else {
            panic!("COIN_ROUND has to be convertible to u8");
        };
        coin
    } else { 10 };

    println!("NODE_COUNT: {} | COIN_ROUND: {}", node_count, coin_round);

    let pool = NodePool::new();
    pool.run(node_count, coin_round);
}
