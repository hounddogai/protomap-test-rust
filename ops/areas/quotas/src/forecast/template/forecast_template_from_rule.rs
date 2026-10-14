use super::*;

use std::collections::BTreeMap;

use crate::usage::rule as usage_rule;

pub fn summarize_default_usage_rules_for_template(items: &[usage_rule::DefaultUsageRule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_forecast_templates_from_rule(items: &[usage_rule::DefaultUsageRule]) -> Vec<ForecastTemplate> {
    let mut out: Vec<ForecastTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_forecast_template(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_forecast_template(item: &ForecastTemplate) -> String {
    usage_rule::regional_usage_template_bucket(item.measured_value())
}
