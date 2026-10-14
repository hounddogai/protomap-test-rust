use super::*;

use std::collections::BTreeMap;

use crate::query::manifest as query_manifest;

pub fn summarize_query_manifests_for_template(items: &[query_manifest::QueryManifest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_data_export_templates_from_manifest(items: &[query_manifest::QueryManifest]) -> Vec<DataExportTemplate> {
    let mut out: Vec<DataExportTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_data_export_template(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_data_export_template(item: &DataExportTemplate) -> String {
    query_manifest::normalize_query_record_code(&item.code)
}
