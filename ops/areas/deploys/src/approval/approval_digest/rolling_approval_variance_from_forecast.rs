use super::*;

use std::collections::BTreeMap;

use crate::freeze::forecast as freeze_forecast;

pub fn summarize_freeze_forecasts_for_variance(items: &[freeze_forecast::FreezeForecast]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_rolling_approval_variances_from_forecast(
    items: &[freeze_forecast::FreezeForecast],
) -> Vec<RollingApprovalVariance> {
    let mut out: Vec<RollingApprovalVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_approval_variance(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_rolling_approval_variance(item: &RollingApprovalVariance) -> String {
    freeze_forecast::classify_freeze_forecast(item.effective_value())
}
