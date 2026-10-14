use super::*;

use std::collections::BTreeMap;

use crate::allowlist::override_item as allowlist_override_item;

pub fn summarize_allowlist_overrides_for_schedule(
    items: &[allowlist_override_item::AllowlistOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_rule_schedules_from_override(items: &[allowlist_override_item::AllowlistOverride]) -> Vec<RuleSchedule> {
    let mut out: Vec<RuleSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_rule_schedule(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_rule_schedule(item: &RuleSchedule) -> f64 {
    allowlist_override_item::adaptive_allowlist_draft_weight(8, true)
}
