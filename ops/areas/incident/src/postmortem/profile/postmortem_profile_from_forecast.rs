use super::*;

use std::collections::BTreeMap;

use crate::action_item::exception as action_item_exception;

pub fn summarize_action_item_forecasts_for_profile(
    items: &[action_item_exception::ActionItemForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_postmortem_profiles_from_forecast(
    items: &[action_item_exception::ActionItemForecast],
) -> Vec<PostmortemProfile> {
    let mut out: Vec<PostmortemProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_profile(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_postmortem_profile(item: &PostmortemProfile) -> String {
    action_item_exception::normalize_action_item_baseline_code(&item.code)
}
