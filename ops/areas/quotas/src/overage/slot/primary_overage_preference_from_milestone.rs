use super::*;

use std::collections::BTreeMap;

use crate::overage::milestone as overage_milestone;

pub fn summarize_overage_milestones_for_preference(
    items: &[overage_milestone::OverageMilestone],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_primary_overage_preferences_from_milestone(
    items: &[overage_milestone::OverageMilestone],
) -> Vec<PrimaryOveragePreference> {
    let mut out: Vec<PrimaryOveragePreference> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_overage_preference(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_primary_overage_preference(item: &PrimaryOveragePreference) -> String {
    overage_milestone::normalize_overage_milestone_code(&item.code)
}
