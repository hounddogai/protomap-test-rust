use super::*;

use std::collections::BTreeMap;

use crate::peering::summary as peering_summary;

pub fn summarize_manual_peering_snapshots_for_record(
    items: &[peering_summary::ManualPeeringSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_rule_records_from_snapshot(items: &[peering_summary::ManualPeeringSnapshot]) -> Vec<RuleRecord> {
    let mut out: Vec<RuleRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_rule_record(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_rule_record(item: &RuleRecord) -> f64 {
    peering_summary::pending_peering_review_weight(4, true)
}
