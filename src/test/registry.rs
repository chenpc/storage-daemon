use super::harness::*;
use libnexus::registry::Registry;
use tokio::runtime::Runtime;

pub fn run(rt: &Runtime, reg: &Registry, r: &mut Runner) {
    run!(r, reg, "unknown_service", {
        let err = rt.block_on(reg.execute("nonexistent", "list", vec![]))
            .unwrap_err();
        assert_contains!(err.to_string(), "unknown service");
    });

    run!(r, reg, "unknown_command", {
        let err = rt.block_on(reg.execute("volume", "nonexistent", vec![]))
            .unwrap_err();
        if err.to_string().is_empty() {
            return Err("expected non-empty error message".into());
        }
    });

    run!(r, reg, "list_services", {
        let services = reg.list_services();
        let names: Vec<&str> = services.iter().map(|(n, _, _)| *n).collect();
        if !names.contains(&"volume") { return Err("missing volume".into()); }
        if !names.contains(&"block") { return Err("missing block".into()); }
        if !names.contains(&"network") { return Err("missing network".into()); }
        if !names.contains(&"pool") { return Err("missing pool".into()); }
    });

    run!(r, reg, "volume_commands", {
        let services = reg.list_services();
        let (_, desc, cmds) = services.iter().find(|(n, _, _)| *n == "volume").unwrap();
        if desc.is_empty() { return Err("empty description".into()); }
        let names: Vec<&str> = cmds.iter().map(|c| c.name.as_str()).collect();
        if !names.contains(&"create") { return Err("missing create".into()); }
        if !names.contains(&"delete") { return Err("missing delete".into()); }
        if !names.contains(&"list") { return Err("missing list".into()); }
    });

    run!(r, reg, "volume_create_arg_metadata", {
        let services = reg.list_services();
        let (_, _, cmds) = services.iter().find(|(n, _, _)| *n == "volume").unwrap();
        let create = cmds.iter().find(|c| c.name == "create").unwrap();
        assert_eq_!(create.args.len(), 2);
        assert_eq_!(create.args[0].name, "name");
        if create.args[0].hint.is_empty() { return Err("empty hint".into()); }
        assert_eq_!(create.args[1].completer, "block.list");
    });
}
