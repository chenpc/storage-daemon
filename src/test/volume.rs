use super::harness::*;
use libnexus::registry::Registry;
use tokio::runtime::Runtime;

pub fn run(rt: &Runtime, reg: &Registry, r: &mut Runner) {
    run!(r, reg, "volume.create", {
        let resp = rt.block_on(reg.execute("volume", "create", vec!["myvol".into(), "vda".into()]))
            .map_err(|e| e.to_string())?;
        assert_contains!(resp, "myvol");
        assert_contains!(resp, "vda");
    });

    run!(r, reg, "volume.delete", {
        let resp = rt.block_on(reg.execute("volume", "delete", vec!["myvol".into()]))
            .map_err(|e| e.to_string())?;
        assert_contains!(resp, "myvol");
        assert_contains!(resp, "deleted");
    });

    run!(r, reg, "volume.list", {
        let resp = rt.block_on(reg.execute("volume", "list", vec![]))
            .map_err(|e| e.to_string())?;
        assert_contains!(resp, "vol0");
    });
}
