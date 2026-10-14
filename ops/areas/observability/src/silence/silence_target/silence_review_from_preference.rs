use super::*;

use std::collections::BTreeMap;

use crate::alert::alert_preference as alert_alert_preference;

pub fn summarize_strict_alert_preferences_for_review(
    items: &[alert_alert_preference::StrictAlertPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_silence_reviews_from_preference(
    items: &[alert_alert_preference::StrictAlertPreference],
) -> Vec<SilenceReview> {
    let mut out: Vec<SilenceReview> = Vec::new();
    for item in items.iter() {
        let derived = new_silence_review(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_silence_review(item: &SilenceReview) -> f64 {
    alert_alert_preference::strict_alert_preference_weight(4, true)
}
