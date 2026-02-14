use super::harness::*;
use libnexus::registry::Registry;
use tokio::runtime::Runtime;

pub fn run(rt: &Runtime, reg: &Registry, r: &mut Runner) {
    run!(r, reg, "network.list", {
        let resp = rt.block_on(reg.execute("network", "list", vec![]))
            .map_err(|e| e.to_string())?;
        assert_contains!(resp, "eth0");
    });

    run!(r, reg, "network.info", {
        let resp = rt.block_on(reg.execute("network", "info", vec!["eth0".into()]))
            .map_err(|e| e.to_string())?;
        assert_contains!(resp, "eth0");
        assert_contains!(resp, "10.0.0.1");
    });

    run!(r, reg, "network.setip", {
        let resp = rt.block_on(reg.execute("network", "setip", vec!["eth0".into(), "192.168.1.1".into(), "255.255.255.0".into()]))
            .map_err(|e| e.to_string())?;
        assert_contains!(resp, "192.168.1.1");
        assert_contains!(resp, "eth0");
    });
}
