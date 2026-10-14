use super::*;

use std::collections::BTreeMap;

use crate::redaction::entry as redaction_entry;

pub fn summarize_redaction_claims_for_template(items: &[redaction_entry::RedactionClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_primary_redaction_templates_from_claim(
    items: &[redaction_entry::RedactionClaim],
) -> Vec<PrimaryRedactionTemplate> {
    let mut out: Vec<PrimaryRedactionTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_redaction_template(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_primary_redaction_template(item: &PrimaryRedactionTemplate) -> String {
    redaction_entry::normalize_redaction_claim_code(&item.code)
}
