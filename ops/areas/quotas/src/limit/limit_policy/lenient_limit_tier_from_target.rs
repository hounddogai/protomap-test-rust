use super::*;

use std::collections::BTreeMap;

use crate::overage::template as overage_template;

pub fn summarize_fallback_overage_targets_for_tier(
    items: &[overage_template::FallbackOverageTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_lenient_limit_tiers_from_target(
    items: &[overage_template::FallbackOverageTarget],
) -> Vec<LenientLimitTier> {
    let mut out: Vec<LenientLimitTier> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_limit_tier(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_lenient_limit_tier(item: &LenientLimitTier) -> f64 {
    overage_template::overage_metric_weight(4, true)
}
