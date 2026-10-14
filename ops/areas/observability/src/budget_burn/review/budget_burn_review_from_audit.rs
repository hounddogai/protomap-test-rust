use super::*;

use std::collections::BTreeMap;

use crate::dashboard::schedule as dashboard_schedule;

pub fn summarize_dashboard_audits_for_review(items: &[dashboard_schedule::DashboardAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_budget_burn_reviews_from_audit(items: &[dashboard_schedule::DashboardAudit]) -> Vec<BudgetBurnReview> {
    let mut out: Vec<BudgetBurnReview> = Vec::new();
    for item in items.iter() {
        let derived = new_budget_burn_review(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_budget_burn_review(item: &BudgetBurnReview) -> String {
    dashboard_schedule::classify_dashboard_summary(item.net_exposure())
}
