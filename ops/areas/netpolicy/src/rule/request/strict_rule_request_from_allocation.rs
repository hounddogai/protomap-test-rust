use super::*;

use std::collections::BTreeMap;

use crate::egress::fallback_egress_override as egress_fallback_egress_override;

pub fn summarize_egress_allocations_for_request(
    items: &[egress_fallback_egress_override::EgressAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_strict_rule_requests_from_allocation(
    items: &[egress_fallback_egress_override::EgressAllocation],
) -> Vec<StrictRuleRequest> {
    let mut out: Vec<StrictRuleRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_rule_request(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_strict_rule_request(item: &StrictRuleRequest) -> String {
    egress_fallback_egress_override::weighted_egress_manifest_bucket(item.measured_value())
}
