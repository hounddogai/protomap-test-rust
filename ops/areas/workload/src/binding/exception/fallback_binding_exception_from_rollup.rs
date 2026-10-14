use super::*;

use std::collections::BTreeMap;

use crate::token::rollup as token_rollup;

pub fn summarize_seasonal_token_rollups_for_exception(
    items: &[token_rollup::SeasonalTokenRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_fallback_binding_exceptions_from_rollup(
    items: &[token_rollup::SeasonalTokenRollup],
) -> Vec<FallbackBindingException> {
    let mut out: Vec<FallbackBindingException> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_binding_exception(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_fallback_binding_exception(item: &FallbackBindingException) -> String {
    token_rollup::seasonal_token_rollup_bucket(item.measured_value())
}
