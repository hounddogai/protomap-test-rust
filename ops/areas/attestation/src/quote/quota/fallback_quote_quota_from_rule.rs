use super::*;

use std::collections::BTreeMap;

use crate::measurement::measurement_group as measurement_measurement_group;

pub fn summarize_adaptive_measurement_rules_for_quota(
    items: &[measurement_measurement_group::AdaptiveMeasurementRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_fallback_quote_quotas_from_rule(
    items: &[measurement_measurement_group::AdaptiveMeasurementRule],
) -> Vec<FallbackQuoteQuota> {
    let mut out: Vec<FallbackQuoteQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_quote_quota(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_fallback_quote_quota(item: &FallbackQuoteQuota) -> String {
    measurement_measurement_group::normalize_weighted_measurement_group_code(&item.code)
}
