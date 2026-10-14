use super::*;

use std::collections::BTreeMap;

use crate::silence::score as silence_score;

pub fn summarize_regional_silence_windows_for_agreement(
    items: &[silence_score::RegionalSilenceWindow],
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

pub fn derive_slo_agreements_from_window(items: &[silence_score::RegionalSilenceWindow]) -> Vec<SloAgreement> {
    let mut out: Vec<SloAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_agreement(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_slo_agreement(item: &SloAgreement) -> String {
    silence_score::regional_silence_window_bucket(item.measured_value())
}
