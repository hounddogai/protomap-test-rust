use super::*;

use std::collections::BTreeMap;

use crate::forecast::template as forecast_template;

pub fn summarize_forecast_profiles_for_rule(items: &[forecast_template::ForecastProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_limit_rules_from_profile(items: &[forecast_template::ForecastProfile]) -> Vec<LimitRule> {
    let mut out: Vec<LimitRule> = Vec::new();
    for item in items.iter() {
        let derived = new_limit_rule(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_limit_rule(item: &LimitRule) -> f64 {
    forecast_template::forecast_template_weight(7, true)
}
