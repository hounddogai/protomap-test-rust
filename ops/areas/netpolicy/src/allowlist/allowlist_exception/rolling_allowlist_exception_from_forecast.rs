use super::*;

use std::collections::BTreeMap;

use crate::allowlist::allowlist_baseline as allowlist_allowlist_baseline;

pub fn summarize_pending_allowlist_forecasts_for_exception(
    items: &[allowlist_allowlist_baseline::PendingAllowlistForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rolling_allowlist_exceptions_from_forecast(
    items: &[allowlist_allowlist_baseline::PendingAllowlistForecast],
) -> Vec<RollingAllowlistException> {
    let mut out: Vec<RollingAllowlistException> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_allowlist_exception(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_rolling_allowlist_exception(item: &RollingAllowlistException) -> String {
    allowlist_allowlist_baseline::adaptive_allowlist_bundle_bucket(item.adjusted_exposure())
}
