use super::*;

use std::collections::BTreeMap;

use crate::slo::targets as slo_targets;

pub fn summarize_slo_targets_for_manifest(items: &[slo_targets::SloTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_alert_manifests_from_target(items: &[slo_targets::SloTarget]) -> Vec<AlertManifest> {
    let mut out: Vec<AlertManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_alert_manifest(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_alert_manifest(item: &AlertManifest) -> f64 {
    slo_targets::slo_metric_weight(6, true)
}
