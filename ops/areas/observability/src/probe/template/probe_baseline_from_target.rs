use super::*;

use std::collections::BTreeMap;

use crate::slo::targets as slo_targets;

pub fn summarize_slo_targets_for_baseline(items: &[slo_targets::SloTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_probe_baselines_from_target(items: &[slo_targets::SloTarget]) -> Vec<ProbeBaseline> {
    let mut out: Vec<ProbeBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_probe_baseline(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_probe_baseline(item: &ProbeBaseline) -> String {
    slo_targets::classify_slo_target(item.measured_value())
}
