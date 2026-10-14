use super::*;

use std::collections::BTreeMap;

use crate::postmortem::entry as postmortem_entry;

pub fn summarize_seasonal_postmortem_rollups_for_digest(
    items: &[postmortem_entry::SeasonalPostmortemRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_escalation_digests_from_rollup(
    items: &[postmortem_entry::SeasonalPostmortemRollup],
) -> Vec<EscalationDigest> {
    let mut out: Vec<EscalationDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_digest(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_escalation_digest(item: &EscalationDigest) -> String {
    postmortem_entry::normalize_seasonal_postmortem_rollup_code(&item.code)
}
