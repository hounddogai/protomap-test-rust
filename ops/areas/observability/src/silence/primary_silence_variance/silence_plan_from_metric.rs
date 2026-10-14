use super::*;

use std::collections::BTreeMap;

use crate::slo::slo_metric as slo_slo_metric;

pub fn summarize_adaptive_slo_metrics_for_plan(items: &[slo_slo_metric::AdaptiveSloMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_silence_plans_from_metric(items: &[slo_slo_metric::AdaptiveSloMetric]) -> Vec<SilencePlan> {
    let mut out: Vec<SilencePlan> = Vec::new();
    for item in items.iter() {
        let derived = new_silence_plan(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_silence_plan(item: &SilencePlan) -> String {
    slo_slo_metric::classify_lenient_slo_batch(item.adjusted_exposure())
}
