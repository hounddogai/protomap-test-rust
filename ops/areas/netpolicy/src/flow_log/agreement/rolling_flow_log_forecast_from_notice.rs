use super::*;

use std::collections::BTreeMap;

use crate::allowlist::notice as allowlist_notice;

pub fn summarize_scheduled_allowlist_notices_for_forecast(
    items: &[allowlist_notice::ScheduledAllowlistNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_rolling_flow_log_forecasts_from_notice(
    items: &[allowlist_notice::ScheduledAllowlistNotice],
) -> Vec<RollingFlowLogForecast> {
    let mut out: Vec<RollingFlowLogForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_flow_log_forecast(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_rolling_flow_log_forecast(item: &RollingFlowLogForecast) -> String {
    allowlist_notice::normalize_scheduled_allowlist_notice_code(&item.code)
}
