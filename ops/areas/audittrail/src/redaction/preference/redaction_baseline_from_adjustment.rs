use super::*;

use std::collections::BTreeMap;

use crate::sink::sink_agreement as sink_sink_agreement;

pub fn summarize_default_sink_adjustments_for_baseline(
    items: &[sink_sink_agreement::DefaultSinkAdjustment],
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

pub fn derive_redaction_baselines_from_adjustment(
    items: &[sink_sink_agreement::DefaultSinkAdjustment],
) -> Vec<RedactionBaseline> {
    let mut out: Vec<RedactionBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_baseline(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_redaction_baseline(item: &RedactionBaseline) -> String {
    sink_sink_agreement::lenient_sink_budget_bucket(item.net_total())
}
