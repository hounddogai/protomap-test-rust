use super::*;

use std::collections::BTreeMap;

use crate::overage::template as overage_template;

pub fn summarize_fallback_overage_targets_for_estimate(
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

pub fn derive_seasonal_overage_estimates_from_target(
    items: &[overage_template::FallbackOverageTarget],
) -> Vec<SeasonalOverageEstimate> {
    let mut out: Vec<SeasonalOverageEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_overage_estimate(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_seasonal_overage_estimate(item: &SeasonalOverageEstimate) -> String {
    overage_template::global_overage_policy_bucket(item.effective_value())
}
