use super::*;

use std::collections::BTreeMap;

use crate::token::token_variance as token_token_variance;

pub fn summarize_scheduled_token_variances_for_threshold(
    items: &[token_token_variance::ScheduledTokenVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_binding_thresholds_from_variance(
    items: &[token_token_variance::ScheduledTokenVariance],
) -> Vec<BindingThreshold> {
    let mut out: Vec<BindingThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_binding_threshold(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_binding_threshold(item: &BindingThreshold) -> String {
    token_token_variance::scheduled_token_variance_bucket(item.projected_value())
}
