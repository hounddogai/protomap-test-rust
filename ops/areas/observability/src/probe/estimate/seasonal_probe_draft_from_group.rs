use super::*;

use std::collections::BTreeMap;

use crate::silence::targets as silence_targets;

pub fn summarize_silence_groups_for_draft(items: &[silence_targets::SilenceGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_seasonal_probe_drafts_from_group(items: &[silence_targets::SilenceGroup]) -> Vec<SeasonalProbeDraft> {
    let mut out: Vec<SeasonalProbeDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_probe_draft(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_seasonal_probe_draft(item: &SeasonalProbeDraft) -> String {
    silence_targets::classify_legacy_silence_window(item.adjusted_exposure())
}
