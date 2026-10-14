use super::*;

use std::collections::BTreeMap;

use crate::dns_record::draft as dns_record_draft;

pub fn summarize_dns_record_drafts_for_milestone(items: &[dns_record_draft::DnsRecordDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_regional_delegation_milestones_from_draft(
    items: &[dns_record_draft::DnsRecordDraft],
) -> Vec<RegionalDelegationMilestone> {
    let mut out: Vec<RegionalDelegationMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_delegation_milestone(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_regional_delegation_milestone(item: &RegionalDelegationMilestone) -> f64 {
    dns_record_draft::dns_record_draft_weight(4, true)
}
