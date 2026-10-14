use super::*;

use std::collections::BTreeMap;

use crate::data_export::quota as data_export_quota;

pub fn summarize_data_export_rules_for_snapshot(items: &[data_export_quota::DataExportRule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_query_snapshots_from_rule(items: &[data_export_quota::DataExportRule]) -> Vec<QuerySnapshot> {
    let mut out: Vec<QuerySnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_query_snapshot(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_query_snapshot(item: &QuerySnapshot) -> String {
    data_export_quota::classify_seasonal_data_export_mapping(item.net_exposure())
}
