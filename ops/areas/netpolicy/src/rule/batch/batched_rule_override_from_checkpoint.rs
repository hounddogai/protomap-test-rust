use super::*;

use std::collections::BTreeMap;

use crate::allowlist::allowlist_milestone as allowlist_allowlist_milestone;

pub fn summarize_legacy_allowlist_checkpoints_for_override(
    items: &[allowlist_allowlist_milestone::LegacyAllowlistCheckpoint],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_batched_rule_overrides_from_checkpoint(
    items: &[allowlist_allowlist_milestone::LegacyAllowlistCheckpoint],
) -> Vec<BatchedRuleOverride> {
    let mut out: Vec<BatchedRuleOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_rule_override(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_batched_rule_override(item: &BatchedRuleOverride) -> String {
    allowlist_allowlist_milestone::normalize_batched_allowlist_entry_code(&item.code)
}
