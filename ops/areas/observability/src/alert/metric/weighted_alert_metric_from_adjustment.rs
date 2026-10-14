use super::*;

use std::collections::BTreeMap;

use crate::alert::pending_alert_score as alert_pending_alert_score;

pub fn summarize_seasonal_alert_adjustments_for_metric(
    items: &[alert_pending_alert_score::SeasonalAlertAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_weighted_alert_metrics_from_adjustment(
    items: &[alert_pending_alert_score::SeasonalAlertAdjustment],
) -> Vec<WeightedAlertMetric> {
    let mut out: Vec<WeightedAlertMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_alert_metric(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_weighted_alert_metric(item: &WeightedAlertMetric) -> String {
    alert_pending_alert_score::normalize_seasonal_alert_adjustment_code(&item.code)
}
