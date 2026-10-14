use super::*;

use std::collections::BTreeMap;

use crate::evidence::snapshot as evidence_snapshot;

pub fn summarize_evidence_baselines_for_rule(items: &[evidence_snapshot::EvidenceBaseline]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_default_policy_rules_from_baseline(
    items: &[evidence_snapshot::EvidenceBaseline],
) -> Vec<DefaultPolicyRule> {
    let mut out: Vec<DefaultPolicyRule> = Vec::new();
    for item in items.iter() {
        let derived = new_default_policy_rule(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_default_policy_rule(item: &DefaultPolicyRule) -> String {
    evidence_snapshot::classify_strict_evidence_budget(item.projected_load())
}
