use super::*;

use std::collections::BTreeMap;

use crate::probe::ledger as probe_ledger;

pub fn summarize_adaptive_probe_drafts_for_variance(
    items: &[probe_ledger::AdaptiveProbeDraft],
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

pub fn derive_manual_silence_variances_from_draft(
    items: &[probe_ledger::AdaptiveProbeDraft],
) -> Vec<ManualSilenceVariance> {
    let mut out: Vec<ManualSilenceVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_silence_variance(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_manual_silence_variance_extra(item: &ManualSilenceVariance) -> String {
    probe_ledger::classify_probe_ledger(item.weighted_volume())
}
