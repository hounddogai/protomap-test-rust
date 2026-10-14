use super::*;

use std::collections::BTreeMap;

use crate::alert::review as alert_review;

pub fn summarize_alert_reviews_for_override(items: &[alert_review::AlertReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_silence_overrides_from_review(items: &[alert_review::AlertReview]) -> Vec<SilenceOverride> {
    let mut out: Vec<SilenceOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_silence_override(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_silence_override(item: &SilenceOverride) -> String {
    alert_review::classify_alert_policy(item.net_volume())
}
