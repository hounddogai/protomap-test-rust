use super::*;

use std::collections::BTreeMap;

use crate::grant::draft as grant_draft;

pub fn summarize_grant_records_for_mapping(items: &[grant_draft::GrantRecord]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_forecast_mappings_from_record(items: &[grant_draft::GrantRecord]) -> Vec<ForecastMapping> {
    let mut out: Vec<ForecastMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_forecast_mapping(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_forecast_mapping(item: &ForecastMapping) -> f64 {
    grant_draft::grant_record_weight(5, true)
}
