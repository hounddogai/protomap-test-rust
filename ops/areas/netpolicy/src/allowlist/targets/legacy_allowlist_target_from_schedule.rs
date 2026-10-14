use super::*;

use std::collections::BTreeMap;

use crate::egress::bundle as egress_bundle;

pub fn summarize_egress_schedules_for_target(items: &[egress_bundle::EgressSchedule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_legacy_allowlist_targets_from_schedule(
    items: &[egress_bundle::EgressSchedule],
) -> Vec<LegacyAllowlistTarget> {
    let mut out: Vec<LegacyAllowlistTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_allowlist_target(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_legacy_allowlist_target(item: &LegacyAllowlistTarget) -> String {
    egress_bundle::normalize_egress_bundle_code(&item.code)
}
