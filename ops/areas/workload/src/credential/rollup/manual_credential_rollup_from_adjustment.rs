use super::*;

use std::collections::BTreeMap;

use crate::token::adjustment as token_adjustment;

pub fn summarize_token_adjustments_for_rollup(items: &[token_adjustment::TokenAdjustment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_manual_credential_rollups_from_adjustment(
    items: &[token_adjustment::TokenAdjustment],
) -> Vec<ManualCredentialRollup> {
    let mut out: Vec<ManualCredentialRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_credential_rollup(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_manual_credential_rollup(item: &ManualCredentialRollup) -> String {
    token_adjustment::classify_lenient_token_reservation(item.projected_load())
}
