use super::*;

use std::collections::BTreeMap;

use crate::approval::approval_agreement as approval_approval_agreement;

pub fn summarize_default_approval_agreements_for_cycle(
    items: &[approval_approval_agreement::DefaultApprovalAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_manual_canary_cycles_from_agreement(
    items: &[approval_approval_agreement::DefaultApprovalAgreement],
) -> Vec<ManualCanaryCycle> {
    let mut out: Vec<ManualCanaryCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_canary_cycle(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_manual_canary_cycle(item: &ManualCanaryCycle) -> String {
    approval_approval_agreement::default_approval_agreement_bucket(item.net_exposure())
}
