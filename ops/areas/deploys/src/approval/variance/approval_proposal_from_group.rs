use super::*;

use std::collections::BTreeMap;

use crate::rollback::cycle as rollback_cycle;

pub fn summarize_rollback_groups_for_proposal(items: &[rollback_cycle::RollbackGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_approval_proposals_from_group(items: &[rollback_cycle::RollbackGroup]) -> Vec<ApprovalProposal> {
    let mut out: Vec<ApprovalProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_approval_proposal(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_approval_proposal(item: &ApprovalProposal) -> String {
    rollback_cycle::classify_rollback_cycle(item.weighted_total())
}
