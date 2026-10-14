use super::*;

use std::collections::BTreeMap;

use crate::silence::bundle as silence_bundle;

pub fn summarize_silence_metrics_for_milestone(items: &[silence_bundle::SilenceMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_seasonal_slo_milestones_from_metric(
    items: &[silence_bundle::SilenceMetric],
) -> Vec<SeasonalSloMilestone> {
    let mut out: Vec<SeasonalSloMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_slo_milestone(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_seasonal_slo_milestone(item: &SeasonalSloMilestone) -> String {
    silence_bundle::normalize_manual_silence_bundle_code(&item.code)
}
