use super::*;

use std::collections::BTreeMap;

use crate::runbook::proposal as runbook_proposal;

pub fn summarize_runbook_proposals_for_forecast(items: &[runbook_proposal::RunbookProposal]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_seasonal_page_forecasts_from_proposal(
    items: &[runbook_proposal::RunbookProposal],
) -> Vec<SeasonalPageForecast> {
    let mut out: Vec<SeasonalPageForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_page_forecast(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_seasonal_page_forecast(item: &SeasonalPageForecast) -> f64 {
    runbook_proposal::runbook_plan_weight(2, true)
}
