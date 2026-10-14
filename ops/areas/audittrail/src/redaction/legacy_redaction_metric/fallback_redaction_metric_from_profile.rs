use super::*;

use std::collections::BTreeMap;

use crate::data_export::template as data_export_template;

pub fn summarize_manual_data_export_profiles_for_metric(
    items: &[data_export_template::ManualDataExportProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_fallback_redaction_metrics_from_profile(
    items: &[data_export_template::ManualDataExportProfile],
) -> Vec<FallbackRedactionMetric> {
    let mut out: Vec<FallbackRedactionMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_redaction_metric(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_fallback_redaction_metric(item: &FallbackRedactionMetric) -> String {
    data_export_template::normalize_manual_data_export_profile_code(&item.code)
}
