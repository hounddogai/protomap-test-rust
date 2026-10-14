use super::*;

use std::collections::BTreeMap;

use crate::postmortem::preference as postmortem_preference;

pub fn summarize_postmortem_preferences_for_variance(
    items: &[postmortem_preference::PostmortemPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_primary_escalation_variances_from_preference(
    items: &[postmortem_preference::PostmortemPreference],
) -> Vec<PrimaryEscalationVariance> {
    let mut out: Vec<PrimaryEscalationVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_escalation_variance(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_primary_escalation_variance(item: &PrimaryEscalationVariance) -> String {
    postmortem_preference::postmortem_plan_bucket(item.effective_value())
}
