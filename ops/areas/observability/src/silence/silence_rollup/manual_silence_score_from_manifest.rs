use super::*;

use std::collections::BTreeMap;

use crate::slo::budget as slo_budget;

pub fn summarize_tiered_slo_manifests_for_score(items: &[slo_budget::TieredSloManifest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_manual_silence_scores_from_manifest(items: &[slo_budget::TieredSloManifest]) -> Vec<ManualSilenceScore> {
    let mut out: Vec<ManualSilenceScore> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_silence_score(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_manual_silence_score(item: &ManualSilenceScore) -> String {
    slo_budget::normalize_tiered_slo_manifest_code(&item.code)
}
