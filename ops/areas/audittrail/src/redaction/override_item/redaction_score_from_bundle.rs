use super::*;

use std::collections::BTreeMap;

use crate::sink::bundle as sink_bundle;

pub fn summarize_sink_bundles_for_score(items: &[sink_bundle::SinkBundle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_redaction_scores_from_bundle(items: &[sink_bundle::SinkBundle]) -> Vec<RedactionScore> {
    let mut out: Vec<RedactionScore> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_score(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_redaction_score(item: &RedactionScore) -> f64 {
    sink_bundle::sink_metric_weight(4, true)
}
