use super::*;

use std::collections::BTreeMap;

use crate::sink::review as sink_review;

pub fn summarize_sink_records_for_target(items: &[sink_review::SinkRecord]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_sink_targets_from_record(items: &[sink_review::SinkRecord]) -> Vec<SinkTarget> {
    let mut out: Vec<SinkTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_target(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_sink_target(item: &SinkTarget) -> String {
    sink_review::classify_sink_record(item.measured_value())
}
