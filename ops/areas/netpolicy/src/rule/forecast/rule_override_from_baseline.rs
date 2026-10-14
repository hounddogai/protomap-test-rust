use super::*;

use std::collections::BTreeMap;

use crate::allowlist::baseline as allowlist_baseline;

pub fn summarize_allowlist_baselines_for_override(
    items: &[allowlist_baseline::AllowlistBaseline],
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

pub fn derive_rule_overrides_from_baseline(items: &[allowlist_baseline::AllowlistBaseline]) -> Vec<RuleOverride> {
    let mut out: Vec<RuleOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_rule_override(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_rule_override(item: &RuleOverride) -> String {
    allowlist_baseline::classify_allowlist_assignment(item.measured_value())
}
