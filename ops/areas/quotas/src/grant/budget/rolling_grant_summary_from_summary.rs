use super::*;

use std::collections::BTreeMap;

use crate::grant::rollup as grant_rollup;

pub fn summarize_grant_summaries_for_summary(items: &[grant_rollup::GrantSummary]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rolling_grant_summaries_from_summary(items: &[grant_rollup::GrantSummary]) -> Vec<RollingGrantSummary> {
    let mut out: Vec<RollingGrantSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_grant_summary(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_rolling_grant_summary(item: &RollingGrantSummary) -> String {
    grant_rollup::grant_group_bucket(item.measured_value())
}
