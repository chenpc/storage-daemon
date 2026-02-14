#[macro_use]
mod harness;
#[path = "../volume.rs"]
mod volume;
#[path = "../network.rs"]
mod network;
#[path = "../pool.rs"]
mod pool;
#[path = "../block.rs"]
mod block;
#[path = "../registry.rs"]
mod registry;

use harness::{registry, Runner};

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let reg = registry();
    let mut r = Runner::new();

    volume::run(&rt, &reg, &mut r);
    network::run(&rt, &reg, &mut r);
    pool::run(&rt, &reg, &mut r);
    block::run(&rt, &reg, &mut r);
    registry::run(&rt, &reg, &mut r);

    std::process::exit(r.summary());
}
