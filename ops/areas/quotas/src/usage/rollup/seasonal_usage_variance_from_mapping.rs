use super::*;

use std::collections::BTreeMap;

use crate::grant::snapshot as grant_snapshot;

pub fn summarize_grant_mappings_for_variance(items: &[grant_snapshot::GrantMapping]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_seasonal_usage_variances_from_mapping(
    items: &[grant_snapshot::GrantMapping],
) -> Vec<SeasonalUsageVariance> {
    let mut out: Vec<SeasonalUsageVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_usage_variance(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_seasonal_usage_variance_extra(item: &SeasonalUsageVariance) -> f64 {
    grant_snapshot::grant_adjustment_weight(4, true)
}
