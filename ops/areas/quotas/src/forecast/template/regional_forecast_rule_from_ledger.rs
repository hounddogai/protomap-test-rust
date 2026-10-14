use super::*;

use std::collections::BTreeMap;

use crate::overage::ledger as overage_ledger;

pub fn summarize_overage_ledgers_for_rule(items: &[overage_ledger::OverageLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_regional_forecast_rules_from_ledger(
    items: &[overage_ledger::OverageLedger],
) -> Vec<RegionalForecastRule> {
    let mut out: Vec<RegionalForecastRule> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_forecast_rule(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_regional_forecast_rule(item: &RegionalForecastRule) -> String {
    overage_ledger::overage_draft_bucket(item.net_total())
}
