use super::*;

use std::collections::BTreeMap;

use crate::query::group as query_group;

pub fn summarize_default_query_records_for_variance(
    items: &[query_group::DefaultQueryRecord],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_capped_sink_variances_from_record(items: &[query_group::DefaultQueryRecord]) -> Vec<CappedSinkVariance> {
    let mut out: Vec<CappedSinkVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_sink_variance(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_capped_sink_variance(item: &CappedSinkVariance) -> String {
    query_group::seasonal_query_forecast_bucket(item.weighted_exposure())
}
