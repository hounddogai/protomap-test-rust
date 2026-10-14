use super::*;

use std::collections::BTreeMap;

use crate::retention::schedule as retention_schedule;

pub fn summarize_retention_profiles_for_draft(items: &[retention_schedule::RetentionProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_redaction_drafts_from_profile(items: &[retention_schedule::RetentionProfile]) -> Vec<RedactionDraft> {
    let mut out: Vec<RedactionDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_draft(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_redaction_draft(item: &RedactionDraft) -> String {
    retention_schedule::normalize_retention_profile_code(&item.code)
}
