use super::*;

use std::collections::BTreeMap;

use crate::silence::variance as silence_variance;

pub fn summarize_silence_variances_for_budget(items: &[silence_variance::SilenceVariance]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_adaptive_silence_budgets_from_variance(
    items: &[silence_variance::SilenceVariance],
) -> Vec<AdaptiveSilenceBudget> {
    let mut out: Vec<AdaptiveSilenceBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_silence_budget(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_adaptive_silence_budget(item: &AdaptiveSilenceBudget) -> String {
    silence_variance::silence_tier_bucket(item.projected_volume())
}
