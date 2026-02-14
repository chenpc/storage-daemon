use super::harness::*;
use libnexus::registry::Registry;
use tokio::runtime::Runtime;

pub fn run(rt: &Runtime, reg: &Registry, r: &mut Runner) {
    run!(r, reg, "pool.create", {
        let resp = rt.block_on(reg.execute("pool", "create", vec!["mypool".into()]))
            .map_err(|e| e.to_string())?;
        assert_contains!(resp, "mypool");
        assert_contains!(resp, "created");
    });

    run!(r, reg, "pool.destroy", {
        let resp = rt.block_on(reg.execute("pool", "destroy", vec!["mypool".into()]))
            .map_err(|e| e.to_string())?;
        assert_contains!(resp, "mypool");
        assert_contains!(resp, "destroyed");
    });
}
