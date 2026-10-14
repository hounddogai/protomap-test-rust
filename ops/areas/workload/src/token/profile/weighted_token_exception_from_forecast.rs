use super::*;

use std::collections::BTreeMap;

use crate::binding::template as binding_template;

pub fn summarize_binding_forecasts_for_exception(items: &[binding_template::BindingForecast]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_weighted_token_exceptions_from_forecast(
    items: &[binding_template::BindingForecast],
) -> Vec<WeightedTokenException> {
    let mut out: Vec<WeightedTokenException> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_token_exception(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_weighted_token_exception(item: &WeightedTokenException) -> String {
    binding_template::classify_legacy_binding_template(item.adjusted_load())
}
