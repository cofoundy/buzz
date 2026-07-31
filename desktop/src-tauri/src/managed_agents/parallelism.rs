// ── Per-harness parallelism cap ───────────────────────────────────────────────

/// Maximum parallelism for the OpenClaw harness.
///
/// Each buzz-acp worker spawned by the Desktop is a client of the single
/// shared OpenClaw Gateway daemon — running more than this number of workers
/// is both resource-expensive and architecturally wrong per the OpenClaw
/// design. Tyler's ruling: "try 5 and lower if needed."
pub const OPENCLAW_MAX_PARALLELISM: u32 = 5;

/// Return the maximum allowed `ManagedAgentRecord.parallelism` for the given
/// harness command, or `None` when the harness has no cap.
///
/// Keyed on [`super::discovery::normalize_command_identity`] so path prefixes, the `.exe`
/// suffix on Windows, and other cosmetic differences are ignored.
pub fn harness_max_parallelism(command: &str) -> Option<u32> {
    match super::discovery::normalize_command_identity(command).as_str() {
        "openclaw" => Some(OPENCLAW_MAX_PARALLELISM),
        _ => None,
    }
}

/// Return the effective parallelism for an instance record given the harness
/// command it will run under.
///
/// Precedence is always resolved *before* calling this function (explicit →
/// definition → `DEFAULT_AGENT_PARALLELISM`). This function only applies the
/// harness cap: `min(value, harness_max_parallelism(command))`.
///
/// For harnesses without a cap this is the identity function.
pub fn effective_parallelism(command: &str, value: u32) -> u32 {
    match harness_max_parallelism(command) {
        Some(cap) => value.min(cap),
        None => value,
    }
}

/// Resolve the effective command for an agent record for parallelism-policy
/// purposes, without requiring a loaded personas slice.
///
/// Uses the record's `runtime` id (set at create/snapshot-apply time) to look
/// up the primary command for the harness. Falls back to `record.agent_command`
/// for legacy records that pre-date the `runtime` field, then to the default.
/// This is intentionally narrower than `record_agent_command` (no personas
/// lookup) — for policy purposes the runtime id stored on the record is
/// sufficient, and the inbound/reconcile paths have no personas context.
pub(crate) fn policy_command_for_record(record: &super::types::ManagedAgentRecord) -> String {
    if let Some(id) = record.runtime.as_deref().filter(|r| !r.is_empty()) {
        if let Some(cmd) =
            super::known_acp_runtime_exact(id).and_then(|r| r.commands.first().copied())
        {
            return cmd.to_string();
        }
        if let Some(def) = super::custom_harnesses::lookup_loaded_harness_by_id(id) {
            return def.command.clone();
        }
    }
    if !record.agent_command.is_empty() {
        return record.agent_command.clone();
    }
    super::default_agent_command()
}

/// Apply the harness parallelism cap to an instance record in place.
///
/// This is the shared normalization helper consumed by every instance
/// persistence boundary. The `effective_command` argument is the resolved
/// harness command for this record — callers must supply the command they
/// actually execute so the policy follows the right identity.
pub(crate) fn normalize_instance_parallelism(
    record: &mut super::types::ManagedAgentRecord,
    effective_command: &str,
) {
    record.parallelism = effective_parallelism(effective_command, record.parallelism);
}

#[cfg(test)]
mod tests {
    use crate::managed_agents::types::ManagedAgentRecord;

    fn record_with(runtime: Option<&str>, parallelism: u32) -> ManagedAgentRecord {
        ManagedAgentRecord {
            pubkey: String::new(),
            name: "r".to_string(),
            persona_id: None,
            private_key_nsec: String::new(),
            auth_tag: None,
            relay_url: String::new(),
            avatar_url: None,
            acp_command: String::new(),
            agent_command: String::new(),
            agent_command_override: None,
            agent_args: vec![],
            mcp_command: String::new(),
            turn_timeout_seconds: 0,
            idle_timeout_seconds: None,
            max_turn_duration_seconds: None,
            parallelism,
            system_prompt: None,
            model: None,
            provider: None,
            persona_source_version: None,
            start_on_app_launch: false,
            auto_restart_on_config_change: true,
            runtime_pid: None,
            backend: Default::default(),
            backend_agent_id: None,
            provider_binary_path: None,
            team_id: None,
            persona_team_dir: None,
            persona_name_in_team: None,
            env_vars: std::collections::BTreeMap::new(),
            created_at: String::new(),
            updated_at: String::new(),
            last_started_at: None,
            last_stopped_at: None,
            last_exit_code: None,
            last_error: None,
            last_error_code: None,
            respond_to: Default::default(),
            respond_to_allowlist: vec![],
            display_name: None,
            slug: None,
            runtime: runtime.map(str::to_string),
            name_pool: Vec::new(),
            is_builtin: false,
            is_active: true,
            shared: false,
            source_team: None,
            source_team_persona_slug: None,
            catalog_source: None,
            definition_respond_to: None,
            definition_respond_to_allowlist: Vec::new(),
            definition_parallelism: None,
            relay_mesh: None,
        }
    }

    #[test]
    fn harness_max_parallelism_openclaw_returns_5() {
        assert_eq!(
            super::harness_max_parallelism("openclaw"),
            Some(super::OPENCLAW_MAX_PARALLELISM)
        );
    }

    #[test]
    fn harness_max_parallelism_openclaw_normalizes_path_prefix_and_exe() {
        assert_eq!(
            super::harness_max_parallelism("/usr/local/bin/openclaw"),
            Some(super::OPENCLAW_MAX_PARALLELISM)
        );
        assert_eq!(
            super::harness_max_parallelism("openclaw.exe"),
            Some(super::OPENCLAW_MAX_PARALLELISM)
        );
        assert_eq!(
            super::harness_max_parallelism(r"C:\Tools\openclaw.exe"),
            Some(super::OPENCLAW_MAX_PARALLELISM)
        );
    }

    #[test]
    fn harness_max_parallelism_unknown_harness_returns_none() {
        assert_eq!(super::harness_max_parallelism("goose"), None);
        assert_eq!(super::harness_max_parallelism("buzz-agent"), None);
        assert_eq!(super::harness_max_parallelism("custom-agent"), None);
        assert_eq!(super::harness_max_parallelism(""), None);
    }

    #[test]
    fn effective_parallelism_clamps_above_cap() {
        assert_eq!(
            super::effective_parallelism("openclaw", 10),
            super::OPENCLAW_MAX_PARALLELISM
        );
        assert_eq!(
            super::effective_parallelism("openclaw", 32),
            super::OPENCLAW_MAX_PARALLELISM
        );
    }

    #[test]
    fn effective_parallelism_honors_value_below_cap() {
        assert_eq!(super::effective_parallelism("openclaw", 5), 5);
        assert_eq!(super::effective_parallelism("openclaw", 1), 1);
        assert_eq!(super::effective_parallelism("openclaw", 3), 3);
    }

    #[test]
    fn effective_parallelism_identity_for_uncapped_harness() {
        assert_eq!(super::effective_parallelism("goose", 10), 10);
        assert_eq!(super::effective_parallelism("goose", 99), 99);
        assert_eq!(super::effective_parallelism("buzz-agent", 32), 32);
        assert_eq!(super::effective_parallelism("custom", 1), 1);
    }

    #[test]
    fn policy_command_for_record_uses_runtime_id() {
        let mut record = record_with(Some("openclaw"), 1);
        record.agent_command = String::new();
        assert_eq!(super::policy_command_for_record(&record), "openclaw");

        record.runtime = Some("goose".to_string());
        assert_eq!(super::policy_command_for_record(&record), "goose");
    }

    #[test]
    fn policy_command_for_record_falls_back_to_agent_command() {
        let mut r = record_with(None, 1);
        r.agent_command = "goose".to_string();
        assert_eq!(super::policy_command_for_record(&r), "goose");
    }

    #[test]
    fn normalize_instance_parallelism_clamps_openclaw_record() {
        let mut record = record_with(None, 10);
        super::normalize_instance_parallelism(&mut record, "openclaw");
        assert_eq!(record.parallelism, super::OPENCLAW_MAX_PARALLELISM);
    }

    #[test]
    fn normalize_instance_parallelism_leaves_non_openclaw_unchanged() {
        let mut record = record_with(None, 10);
        super::normalize_instance_parallelism(&mut record, "goose");
        assert_eq!(record.parallelism, 10);
    }
}
