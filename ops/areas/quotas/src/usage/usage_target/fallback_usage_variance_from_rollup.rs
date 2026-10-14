use super::*;

use std::collections::BTreeMap;

use crate::grant::grant_template as grant_grant_template;

pub fn summarize_legacy_grant_rollups_for_variance(
    items: &[grant_grant_template::LegacyGrantRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_fallback_usage_variances_from_rollup(
    items: &[grant_grant_template::LegacyGrantRollup],
) -> Vec<FallbackUsageVariance> {
    let mut out: Vec<FallbackUsageVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_usage_variance(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_fallback_usage_variance(item: &FallbackUsageVariance) -> String {
    grant_grant_template::grant_score_bucket(item.net_value())
}
