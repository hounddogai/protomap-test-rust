use super::*;

use acme_kit::text;

/// How screens and exports show a strict audit event budget.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAuditEventBudgetView {
    pub code: String,
    pub name_text: String,
    pub approved: String,
    pub locale: String,
    pub revision: String,
    pub created_at: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_strict_audit_event_budget_view(item: &StrictAuditEventBudget) -> StrictAuditEventBudgetView {
    StrictAuditEventBudgetView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 120),
        approved: format!("{}", item.approved),
        locale: item.locale.clone(),
        revision: item.revision.to_string(),
        created_at: item.created_at.to_string(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_audit_event_budget_views(items: &[StrictAuditEventBudget]) -> Vec<StrictAuditEventBudgetView> {
    let mut views: Vec<StrictAuditEventBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_audit_event_budget_view(item));
    }
    views
}

pub fn sum_strict_audit_event_budget_view(view: &StrictAuditEventBudgetView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_created_at as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_audit_event_budget("code-657", "name-281");
        let view = to_strict_audit_event_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_audit_event_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
