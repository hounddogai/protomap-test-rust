use super::*;

use std::collections::BTreeMap;

use crate::query::segment as query_segment;

pub fn summarize_query_quotas_for_record(items: &[query_segment::QueryQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_sink_records_from_quota(items: &[query_segment::QueryQuota]) -> Vec<SinkRecord> {
    let mut out: Vec<SinkRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_record(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_sink_record(item: &SinkRecord) -> f64 {
    query_segment::query_quota_weight(7, true)
}
