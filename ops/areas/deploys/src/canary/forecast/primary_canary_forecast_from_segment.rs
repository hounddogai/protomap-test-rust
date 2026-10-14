use super::*;

use std::collections::BTreeMap;

use crate::freeze::review as freeze_review;

pub fn summarize_freeze_segments_for_forecast(items: &[freeze_review::FreezeSegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_primary_canary_forecasts_from_segment(
    items: &[freeze_review::FreezeSegment],
) -> Vec<PrimaryCanaryForecast> {
    let mut out: Vec<PrimaryCanaryForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_canary_forecast(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_primary_canary_forecast(item: &PrimaryCanaryForecast) -> String {
    freeze_review::normalize_freeze_proposal_code(&item.code)
}
