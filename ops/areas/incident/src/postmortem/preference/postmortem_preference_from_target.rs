use super::*;

use std::collections::BTreeMap;

use crate::page::forecast as page_forecast;

pub fn summarize_page_targets_for_preference(items: &[page_forecast::PageTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_postmortem_preferences_from_target(items: &[page_forecast::PageTarget]) -> Vec<PostmortemPreference> {
    let mut out: Vec<PostmortemPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_preference(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_postmortem_preference(item: &PostmortemPreference) -> String {
    page_forecast::page_forecast_bucket(item.projected_volume())
}
