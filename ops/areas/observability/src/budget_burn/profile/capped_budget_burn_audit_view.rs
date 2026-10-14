use super::*;

use acme_kit::text;

/// How screens and exports show a capped budget burn audit.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnAuditView {
    pub code_text: String,
    pub label: String,
    pub grace_days: String,
    pub locale: String,
    pub window_days_text: String,
    pub ttl_seconds_text: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_capped_budget_burn_audit_view(item: &CappedBudgetBurnAudit) -> CappedBudgetBurnAuditView {
    CappedBudgetBurnAuditView {
        code_text: text::truncate(&item.code, 64),
        label: item.label.clone(),
        grace_days: item.grace_days.to_string(),
        locale: text::truncate(&item.locale, 120),
        window_days_text: item.window_days.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_budget_burn_audit_views(items: &[CappedBudgetBurnAudit]) -> Vec<CappedBudgetBurnAuditView> {
    let mut views: Vec<CappedBudgetBurnAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_budget_burn_audit_view(item));
    }
    views
}

pub fn sum_capped_budget_burn_audit_view(view: &CappedBudgetBurnAuditView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_window_days as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_budget_burn_audit("code-127", "label-74");
        let view = to_capped_budget_burn_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_budget_burn_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
