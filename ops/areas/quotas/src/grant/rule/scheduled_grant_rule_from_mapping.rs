use super::*;

use std::collections::BTreeMap;

use crate::forecast::forecast_score as forecast_forecast_score;

pub fn summarize_weighted_forecast_mappings_for_rule(
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

pub fn derive_scheduled_grant_rules_from_mapping(
    items: &[forecast_forecast_score::WeightedForecastMapping],
) -> Vec<ScheduledGrantRule> {
    let mut out: Vec<ScheduledGrantRule> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_grant_rule(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_scheduled_grant_rule(item: &ScheduledGrantRule) -> String {
    forecast_forecast_score::classify_forecast_notice(item.weighted_value())
}
