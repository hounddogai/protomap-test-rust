use super::*;

use std::collections::BTreeMap;

use crate::suspension::record_item as suspension_record_item;

pub fn summarize_suspension_drafts_for_proposal(
    items: &[suspension_record_item::SuspensionDraft],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_default_tenant_proposals_from_draft(
    items: &[suspension_record_item::SuspensionDraft],
) -> Vec<DefaultTenantProposal> {
    let mut out: Vec<DefaultTenantProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_default_tenant_proposal(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_default_tenant_proposal(item: &DefaultTenantProposal) -> f64 {
    suspension_record_item::suspension_draft_weight(7, true)
}
