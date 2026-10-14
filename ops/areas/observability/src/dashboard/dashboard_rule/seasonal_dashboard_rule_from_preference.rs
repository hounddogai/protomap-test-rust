use super::*;

use std::collections::BTreeMap;

use crate::alert::alert_preference as alert_alert_preference;

pub fn summarize_strict_alert_preferences_for_rule(
    items: &[alert_alert_preference::StrictAlertPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_seasonal_dashboard_rules_from_preference(
    items: &[alert_alert_preference::StrictAlertPreference],
) -> Vec<SeasonalDashboardRule> {
    let mut out: Vec<SeasonalDashboardRule> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_dashboard_rule(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_seasonal_dashboard_rule(item: &SeasonalDashboardRule) -> String {
    alert_alert_preference::seasonal_alert_record_bucket(item.weighted_total())
}
