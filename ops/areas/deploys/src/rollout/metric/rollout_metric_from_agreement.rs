use super::*;

use std::collections::BTreeMap;

use crate::release::release_record as release_release_record;

pub fn summarize_adaptive_release_agreements_for_metric(
    items: &[release_release_record::AdaptiveReleaseAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_rollout_metrics_from_agreement(
    items: &[release_release_record::AdaptiveReleaseAgreement],
) -> Vec<RolloutMetric> {
    let mut out: Vec<RolloutMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_rollout_metric(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_rollout_metric(item: &RolloutMetric) -> String {
    release_release_record::normalize_rolling_release_record_code(&item.code)
}
