use libnexus::registry::Registry;
use storage_daemon::services::{
    block::Block, network::Network, pool::Pool, volume::Volume,
};

pub fn registry() -> Registry {
    let mut reg = Registry::new();
    reg.register(Volume);
    reg.register(Block);
    reg.register(Network);
    reg.register(Pool);
    reg
}

pub struct Runner {
    passed: u32,
    failed: u32,
}

impl Runner {
    pub fn new() -> Self {
        Self { passed: 0, failed: 0 }
    }

    pub fn check(&mut self, name: &str, result: Result<(), String>) {
        match result {
            Ok(()) => {
                println!("  PASS  {}", name);
                self.passed += 1;
            }
            Err(e) => {
                println!("  FAIL  {} — {}", name, e);
                self.failed += 1;
            }
        }
    }

    pub fn summary(self) -> i32 {
        println!();
        println!("{} passed, {} failed", self.passed, self.failed);
        if self.failed > 0 { 1 } else { 0 }
    }
}

macro_rules! run {
    ($runner:expr, $reg:expr, $name:expr, $body:expr) => {{
        let result: Result<(), String> = (|| -> Result<(), String> {
            $body;
            Ok(())
        })();
        $runner.check($name, result);
    }};
}

macro_rules! assert_contains {
    ($haystack:expr, $needle:expr) => {
        if !$haystack.contains($needle) {
            return Err(format!("expected {:?} to contain {:?}", $haystack, $needle));
        }
    };
}

macro_rules! assert_eq_ {
    ($left:expr, $right:expr) => {
        if $left != $right {
            return Err(format!("expected {:?} == {:?}", $left, $right));
        }
    };
}
