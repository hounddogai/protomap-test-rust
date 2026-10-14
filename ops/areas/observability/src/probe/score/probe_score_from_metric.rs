use super::*;

use std::collections::BTreeMap;

use crate::probe::plan as probe_plan;

pub fn summarize_probe_metrics_for_score(items: &[probe_plan::ProbeMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_probe_scores_from_metric(items: &[probe_plan::ProbeMetric]) -> Vec<ProbeScore> {
    let mut out: Vec<ProbeScore> = Vec::new();
    for item in items.iter() {
        let derived = new_probe_score(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_probe_score(item: &ProbeScore) -> String {
    probe_plan::normalize_probe_metric_code(&item.code)
}
