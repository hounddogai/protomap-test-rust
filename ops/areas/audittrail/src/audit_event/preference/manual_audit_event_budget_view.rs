use super::*;

use acme_kit::text;

/// How screens and exports show a manual audit event budget.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventBudgetView {
    pub code: String,
    pub label_text: String,
    pub budget_text: String,
    pub region: String,
    pub expires_at: String,
    pub cluster: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_manual_audit_event_budget_view(item: &ManualAuditEventBudget) -> ManualAuditEventBudgetView {
    ManualAuditEventBudgetView {
        code: text::truncate(&item.code, 40),
        label_text: text::truncate(&item.label, 120),
        budget_text: format!("{:.2}", item.budget),
        region: item.region.clone(),
        expires_at: item.expires_at.to_string(),
        cluster: item.cluster.clone(),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_audit_event_budget_views(items: &[ManualAuditEventBudget]) -> Vec<ManualAuditEventBudgetView> {
    let mut views: Vec<ManualAuditEventBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_audit_event_budget_view(item));
    }
    views
}

pub fn sum_manual_audit_event_budget_view(view: &ManualAuditEventBudgetView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_budget + (parsed_expires_at as f64) + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_audit_event_budget("code-420", "label-534");
        let view = to_manual_audit_event_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_audit_event_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
