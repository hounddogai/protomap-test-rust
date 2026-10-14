use super::*;

use std::collections::BTreeMap;

use crate::verifier::template as verifier_template;

pub fn summarize_verifier_templates_for_batch(items: &[verifier_template::VerifierTemplate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_verifier_batches_from_template(items: &[verifier_template::VerifierTemplate]) -> Vec<VerifierBatch> {
    let mut out: Vec<VerifierBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_batch(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_verifier_batch(item: &VerifierBatch) -> String {
    verifier_template::classify_verifier_bundle(item.net_total())
}
