use super::harness::*;
use libnexus::registry::Registry;
use tokio::runtime::Runtime;

pub fn run(rt: &Runtime, reg: &Registry, r: &mut Runner) {
    run!(r, reg, "pool.create (invalid raid)", {
        let err = rt.block_on(reg.execute("pool", "create", vec![
            "mypool".into(), "stripe".into(), "vdb".into(), "vdc".into(),
        ])).unwrap_err();
        assert_contains!(err.to_string(), "invalid");
    });

    run!(r, reg, "pool.create (too few devices)", {
        let err = rt.block_on(reg.execute("pool", "create", vec![
            "mypool".into(), "mirror".into(), "vda".into(),
        ])).unwrap_err();
        assert_contains!(err.to_string(), "at least");
    });

    run!(r, reg, "pool.list", {
        let result = rt.block_on(reg.execute("pool", "list", vec![]));
        match result {
            Ok(json) => {
                let _: serde_json::Value = serde_json::from_str(&json).unwrap();
            }
            Err(e) => assert_contains!(e.to_string(), "zpool"),
        }
    });

    run!(r, reg, "pool.destroy (missing name)", {
        let err = rt.block_on(reg.execute("pool", "destroy", vec![]))
            .unwrap_err();
        assert_contains!(err.to_string(), "missing");
    });
}
