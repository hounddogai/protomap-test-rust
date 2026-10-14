use super::*;

use std::collections::BTreeMap;

use crate::data_export::profile as data_export_profile;

pub fn summarize_data_export_profiles_for_estimate(
    items: &[data_export_profile::DataExportProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_data_export_estimates_from_profile(
    items: &[data_export_profile::DataExportProfile],
) -> Vec<DataExportEstimate> {
    let mut out: Vec<DataExportEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_data_export_estimate(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_data_export_estimate(item: &DataExportEstimate) -> f64 {
    data_export_profile::data_export_profile_weight(6, true)
}
