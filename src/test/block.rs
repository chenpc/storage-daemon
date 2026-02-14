use super::harness::*;
use libnexus::registry::Registry;
use tokio::runtime::Runtime;

pub fn run(rt: &Runtime, reg: &Registry, r: &mut Runner) {
    run!(r, reg, "block.list", {
        let result = rt.block_on(reg.execute("block", "list", vec![]));

        match result {
            Ok(json) => {
                let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
                let obj = parsed.as_object().expect("block.list should return a JSON object");
                assert!(!obj.is_empty(), "block.list should return at least one device");
            }
            Err(e) => assert_contains!(e.to_string(), "lsblk"),
        }
    });

    run!(r, reg, "block.info", {
        // First get a device name from block.list
        let list_result = rt.block_on(reg.execute("block", "list", vec![]));
        let Ok(list_json) = list_result else {
            return Ok(()); // skip if lsblk not available
        };
        let parsed: serde_json::Value = serde_json::from_str(&list_json).unwrap();
        let device = parsed.as_object().unwrap().keys().next().unwrap().clone();

        let result = rt.block_on(reg.execute("block", "info", vec![device.clone()]));
        let json = result.unwrap();
        let info: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = info.as_object().expect("block.info should return a JSON object");
        assert!(obj.contains_key("size"), "block.info should contain 'size'");
        assert!(obj.contains_key("dtype"), "block.info should contain 'dtype'");
    });
}
