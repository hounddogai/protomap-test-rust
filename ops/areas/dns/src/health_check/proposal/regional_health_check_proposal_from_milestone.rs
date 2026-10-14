use super::*;

use std::collections::BTreeMap;

use crate::health_check::check_milestone as health_check_check_milestone;

pub fn summarize_scheduled_health_check_milestones_for_proposal(
    items: &[health_check_check_milestone::ScheduledHealthCheckMilestone],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_regional_health_check_proposals_from_milestone(
    items: &[health_check_check_milestone::ScheduledHealthCheckMilestone],
) -> Vec<RegionalHealthCheckProposal> {
    let mut out: Vec<RegionalHealthCheckProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_health_check_proposal(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_regional_health_check_proposal(item: &RegionalHealthCheckProposal) -> f64 {
    health_check_check_milestone::scheduled_health_check_milestone_weight(6, true)
}
