use super::*;

use std::collections::BTreeMap;

use crate::sink::template as sink_template;

pub fn summarize_scheduled_sink_digests_for_policy(
    items: &[sink_template::ScheduledSinkDigest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_capped_data_export_policies_from_digest(
    items: &[sink_template::ScheduledSinkDigest],
) -> Vec<CappedDataExportPolicy> {
    let mut out: Vec<CappedDataExportPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_data_export_policy(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_capped_data_export_policy(item: &CappedDataExportPolicy) -> String {
    sink_template::scheduled_sink_digest_bucket(item.measured_value())
}
