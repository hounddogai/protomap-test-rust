use super::*;

use std::collections::BTreeMap;

use crate::postmortem::allocation as postmortem_allocation;

pub fn summarize_legacy_postmortem_profiles_for_summary(
    items: &[postmortem_allocation::LegacyPostmortemProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_postmortem_summaries_from_profile(
    items: &[postmortem_allocation::LegacyPostmortemProfile],
) -> Vec<PostmortemSummary> {
    let mut out: Vec<PostmortemSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_summary(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_postmortem_summary(item: &PostmortemSummary) -> String {
    postmortem_allocation::tiered_postmortem_digest_bucket(item.net_value())
}
