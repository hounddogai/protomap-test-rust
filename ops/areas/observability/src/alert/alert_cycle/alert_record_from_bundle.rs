use super::*;

use std::collections::BTreeMap;

use crate::silence::bundle as silence_bundle;

pub fn summarize_manual_silence_bundles_for_record(
    items: &[silence_bundle::ManualSilenceBundle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_alert_records_from_bundle(items: &[silence_bundle::ManualSilenceBundle]) -> Vec<AlertRecord> {
    let mut out: Vec<AlertRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_alert_record(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_alert_record(item: &AlertRecord) -> String {
    silence_bundle::normalize_manual_silence_bundle_code(&item.code)
}
