use super::*;

use std::collections::BTreeMap;

use crate::overage::budget as overage_budget;

pub fn summarize_lenient_overage_forecasts_for_plan(
    items: &[overage_budget::LenientOverageForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_primary_limit_plans_from_forecast(
    items: &[overage_budget::LenientOverageForecast],
) -> Vec<PrimaryLimitPlan> {
    let mut out: Vec<PrimaryLimitPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_limit_plan(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_primary_limit_plan(item: &PrimaryLimitPlan) -> String {
    overage_budget::normalize_strict_overage_rule_code(&item.code)
}
