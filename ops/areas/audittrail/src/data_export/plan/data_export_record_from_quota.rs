use super::*;

use std::collections::BTreeMap;

use crate::sink::quota as sink_quota;

pub fn summarize_sink_quotas_for_record(items: &[sink_quota::SinkQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_data_export_records_from_quota(items: &[sink_quota::SinkQuota]) -> Vec<DataExportRecord> {
    let mut out: Vec<DataExportRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_data_export_record(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_data_export_record(item: &DataExportRecord) -> f64 {
    sink_quota::sink_exception_weight(2, true)
}
