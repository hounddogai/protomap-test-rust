use super::*;

use std::collections::BTreeMap;

use crate::grant::plan as grant_plan;

pub fn summarize_grant_plans_for_baseline(items: &[grant_plan::GrantPlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_seasonal_grant_baselines_from_plan(items: &[grant_plan::GrantPlan]) -> Vec<SeasonalGrantBaseline> {
    let mut out: Vec<SeasonalGrantBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_grant_baseline(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_seasonal_grant_baseline(item: &SeasonalGrantBaseline) -> String {
    grant_plan::classify_rolling_grant_variance(item.measured_value())
}
