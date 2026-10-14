use super::*;

use std::collections::BTreeMap;

use crate::data_export::metric as data_export_metric;

pub fn summarize_data_export_overrides_for_baseline(
    items: &[data_export_metric::DataExportOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_strict_retention_baselines_from_override(
    items: &[data_export_metric::DataExportOverride],
) -> Vec<StrictRetentionBaseline> {
    let mut out: Vec<StrictRetentionBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_retention_baseline(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_strict_retention_baseline(item: &StrictRetentionBaseline) -> String {
    data_export_metric::normalize_capped_data_export_policy_code(&item.code)
}
