use super::*;

use std::collections::BTreeMap;

use crate::query::query_milestone as query_query_milestone;

pub fn summarize_query_rollups_for_policy(items: &[query_query_milestone::QueryRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_global_data_export_policies_from_rollup(
    items: &[query_query_milestone::QueryRollup],
) -> Vec<GlobalDataExportPolicy> {
    let mut out: Vec<GlobalDataExportPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_global_data_export_policy(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_global_data_export_policy(item: &GlobalDataExportPolicy) -> String {
    query_query_milestone::normalize_query_slot_code(&item.code)
}
