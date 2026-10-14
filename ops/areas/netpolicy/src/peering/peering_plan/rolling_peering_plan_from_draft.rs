use super::*;

use std::collections::BTreeMap;

use crate::flow_log::draft as flow_log_draft;

pub fn summarize_lenient_flow_log_drafts_for_plan(
    items: &[flow_log_draft::LenientFlowLogDraft],
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

pub fn derive_rolling_peering_plans_from_draft(
    items: &[flow_log_draft::LenientFlowLogDraft],
) -> Vec<RollingPeeringPlan> {
    let mut out: Vec<RollingPeeringPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_peering_plan(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_rolling_peering_plan(item: &RollingPeeringPlan) -> String {
    flow_log_draft::lenient_flow_log_draft_bucket(item.projected_total())
}
