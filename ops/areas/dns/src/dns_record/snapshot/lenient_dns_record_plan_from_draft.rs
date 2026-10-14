use super::*;

use std::collections::BTreeMap;

use crate::delegation::delegation_draft as delegation_delegation_draft;

pub fn summarize_capped_delegation_drafts_for_plan(
    items: &[delegation_delegation_draft::CappedDelegationDraft],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_lenient_dns_record_plans_from_draft(
    items: &[delegation_delegation_draft::CappedDelegationDraft],
) -> Vec<LenientDnsRecordPlan> {
    let mut out: Vec<LenientDnsRecordPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_dns_record_plan(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_lenient_dns_record_plan(item: &LenientDnsRecordPlan) -> String {
    delegation_delegation_draft::normalize_delegation_metric_code(&item.code)
}
