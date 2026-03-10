use super::harness::*;
use libnexus::registry::Registry;
use tokio::runtime::Runtime;

pub fn run(rt: &Runtime, reg: &Registry, r: &mut Runner) {
    run!(r, reg, "volume.create", {
        let result = rt.block_on(reg.execute("volume", "create", vec!["myvol".into(), "tank".into()]));
        match result {
            Ok(resp) => {
                assert_contains!(resp, "myvol");
                assert_contains!(resp, "tank");
            }
            // zfs or samba not available in test env — acceptable
            Err(e) => assert_contains!(e.to_string(), "zfs"),
        }
    });

    run!(r, reg, "volume.delete", {
        let result = rt.block_on(reg.execute("volume", "delete", vec!["tank/myvol".into()]));
        match result {
            Ok(resp) => assert_contains!(resp, "deleted"),
            Err(e) => assert_contains!(e.to_string(), "zfs"),
        }
    });

    run!(r, reg, "volume.list", {
        let result = rt.block_on(reg.execute("volume", "list", vec![]));
        match result {
            Ok(json) => {
                let _: serde_json::Value = serde_json::from_str(&json).unwrap();
            }
            Err(e) => assert_contains!(e.to_string(), "zfs"),
        }
    });

    run!(r, reg, "volume.create (missing pool)", {
        let err = rt.block_on(reg.execute("volume", "create", vec!["myvol".into()]))
            .unwrap_err();
        assert_contains!(err.to_string(), "missing");
    });
}
