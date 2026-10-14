use super::*;

use std::collections::BTreeMap;

use crate::egress::baseline as egress_baseline;

pub fn summarize_strict_egress_baselines_for_digest(
    items: &[egress_baseline::StrictEgressBaseline],
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

pub fn derive_tiered_rule_digests_from_baseline(
    items: &[egress_baseline::StrictEgressBaseline],
) -> Vec<TieredRuleDigest> {
    let mut out: Vec<TieredRuleDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_rule_digest(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_tiered_rule_digest(item: &TieredRuleDigest) -> String {
    egress_baseline::classify_strict_egress_baseline(item.weighted_value())
}
