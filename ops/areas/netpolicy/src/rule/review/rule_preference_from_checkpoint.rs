use super::*;

use std::collections::BTreeMap;

use crate::allowlist::milestone as allowlist_milestone;

pub fn summarize_tiered_allowlist_checkpoints_for_preference(
    items: &[allowlist_milestone::TieredAllowlistCheckpoint],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_rule_preferences_from_checkpoint(
    items: &[allowlist_milestone::TieredAllowlistCheckpoint],
) -> Vec<RulePreference> {
    let mut out: Vec<RulePreference> = Vec::new();
    for item in items.iter() {
        let derived = new_rule_preference(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_rule_preference(item: &RulePreference) -> String {
    allowlist_milestone::normalize_tiered_allowlist_checkpoint_code(&item.code)
}
