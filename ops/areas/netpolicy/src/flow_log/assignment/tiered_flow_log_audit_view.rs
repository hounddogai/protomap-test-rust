use super::*;

use acme_kit::text;

/// How screens and exports show a tiered flow log audit.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogAuditView {
    pub code: String,
    pub title_text: String,
    pub description_text: String,
    pub summary_text: String,
    pub external_ref: String,
    pub weight: String,
    pub budget_text: String,
    pub locale: String,
    pub starts_at: String,
    pub threshold: String,
    pub name: String,
    pub status_label: String,
}

pub fn to_tiered_flow_log_audit_view(item: &TieredFlowLogAudit) -> TieredFlowLogAuditView {
    TieredFlowLogAuditView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        description_text: item.description.clone(),
        summary_text: item.summary.clone(),
        external_ref: text::truncate(&item.external_ref, 40),
        weight: format!("{:.2}", item.weight),
        budget_text: format!("{:.2}", item.budget),
        locale: item.locale.clone(),
        starts_at: item.starts_at.to_string(),
        threshold: format!("{:.2}", item.threshold),
        name: item.name.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_flow_log_audit_views(items: &[TieredFlowLogAudit]) -> Vec<TieredFlowLogAuditView> {
    let mut views: Vec<TieredFlowLogAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_flow_log_audit_view(item));
    }
    views
}

pub fn sum_tiered_flow_log_audit_view(view: &TieredFlowLogAuditView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_weight + parsed_budget + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_flow_log_audit("code-931", "title-798");
        let view = to_tiered_flow_log_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_flow_log_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
