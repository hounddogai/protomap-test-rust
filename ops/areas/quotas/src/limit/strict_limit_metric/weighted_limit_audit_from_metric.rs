use super::*;

use std::collections::BTreeMap;

use crate::grant::override_item as grant_override_item;

pub fn summarize_global_grant_metrics_for_audit(
    items: &[grant_override_item::GlobalGrantMetric],
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

pub fn derive_weighted_limit_audits_from_metric(
    items: &[grant_override_item::GlobalGrantMetric],
) -> Vec<WeightedLimitAudit> {
    let mut out: Vec<WeightedLimitAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_limit_audit(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_weighted_limit_audit(item: &WeightedLimitAudit) -> f64 {
    grant_override_item::grant_override_weight(8, true)
}
