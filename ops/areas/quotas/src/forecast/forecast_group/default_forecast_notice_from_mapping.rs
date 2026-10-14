use super::*;

use std::collections::BTreeMap;

use crate::forecast::forecast_score as forecast_forecast_score;

pub fn summarize_weighted_forecast_mappings_for_notice(
    items: &[forecast_forecast_score::WeightedForecastMapping],
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

pub fn derive_default_forecast_notices_from_mapping(
    items: &[forecast_forecast_score::WeightedForecastMapping],
) -> Vec<DefaultForecastNotice> {
    let mut out: Vec<DefaultForecastNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_default_forecast_notice(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_default_forecast_notice(item: &DefaultForecastNotice) -> f64 {
    forecast_forecast_score::capped_forecast_score_weight(9, true)
}
