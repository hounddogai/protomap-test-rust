use super::*;

use std::collections::BTreeMap;

use crate::forecast::exception as forecast_exception;

pub fn summarize_forecast_batches_for_baseline(items: &[forecast_exception::ForecastBatch]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_grant_baselines_from_batch(items: &[forecast_exception::ForecastBatch]) -> Vec<GrantBaseline> {
    let mut out: Vec<GrantBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_baseline(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_grant_baseline(item: &GrantBaseline) -> String {
    forecast_exception::adaptive_forecast_cycle_bucket(item.projected_total())
}
