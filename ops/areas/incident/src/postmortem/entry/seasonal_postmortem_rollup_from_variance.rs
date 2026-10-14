use super::*;

use std::collections::BTreeMap;

use crate::page::batch as page_batch;

pub fn summarize_lenient_page_variances_for_rollup(items: &[page_batch::LenientPageVariance]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_seasonal_postmortem_rollups_from_variance(
    items: &[page_batch::LenientPageVariance],
) -> Vec<SeasonalPostmortemRollup> {
    let mut out: Vec<SeasonalPostmortemRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_postmortem_rollup(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_seasonal_postmortem_rollup(item: &SeasonalPostmortemRollup) -> String {
    page_batch::lenient_page_variance_bucket(item.weighted_volume())
}
