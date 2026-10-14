use super::*;

use std::collections::BTreeMap;

use crate::sink::draft as sink_draft;

pub fn summarize_sink_drafts_for_metric(items: &[sink_draft::SinkDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_sink_metrics_from_draft(items: &[sink_draft::SinkDraft]) -> Vec<SinkMetric> {
    let mut out: Vec<SinkMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_metric(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_sink_metric(item: &SinkMetric) -> String {
    sink_draft::classify_sink_mapping(item.effective_value())
}
