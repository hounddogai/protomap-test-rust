use super::*;

use std::collections::BTreeMap;

use crate::alert::alert_override as alert_alert_override;

pub fn summarize_lenient_alert_overrides_for_metric(
    items: &[alert_alert_override::LenientAlertOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_weighted_budget_burn_metrics_from_override(
    items: &[alert_alert_override::LenientAlertOverride],
) -> Vec<WeightedBudgetBurnMetric> {
    let mut out: Vec<WeightedBudgetBurnMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_budget_burn_metric(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_weighted_budget_burn_metric(item: &WeightedBudgetBurnMetric) -> String {
    alert_alert_override::normalize_rolling_alert_manifest_code(&item.code)
}
