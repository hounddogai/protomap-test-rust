use super::*;

use std::collections::BTreeMap;

use crate::grant::rule as grant_rule;

pub fn summarize_lenient_grant_cycles_for_snapshot(items: &[grant_rule::LenientGrantCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_regional_overage_snapshots_from_cycle(
    items: &[grant_rule::LenientGrantCycle],
) -> Vec<RegionalOverageSnapshot> {
    let mut out: Vec<RegionalOverageSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_overage_snapshot(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_regional_overage_snapshot(item: &RegionalOverageSnapshot) -> String {
    grant_rule::pending_grant_metric_bucket(item.effective_value())
}
