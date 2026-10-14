use super::*;

use std::collections::BTreeMap;

use crate::retention::review as retention_review;

pub fn summarize_pending_retention_budgets_for_draft(
    items: &[retention_review::PendingRetentionBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_default_data_export_drafts_from_budget(
    items: &[retention_review::PendingRetentionBudget],
) -> Vec<DefaultDataExportDraft> {
    let mut out: Vec<DefaultDataExportDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_default_data_export_draft(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_default_data_export_draft(item: &DefaultDataExportDraft) -> String {
    retention_review::normalize_pending_retention_budget_code(&item.code)
}
