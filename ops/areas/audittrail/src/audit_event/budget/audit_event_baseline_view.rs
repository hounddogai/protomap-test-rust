use super::*;

use acme_kit::text;

/// How screens and exports show a audit event baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBaselineView {
    pub code: String,
    pub title_text: String,
    pub cost: String,
    pub approved: String,
    pub quantity: String,
    pub currency: String,
    pub rotation_days_text: String,
    pub created_at_text: String,
    pub minimum: String,
    pub rate_text: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_audit_event_baseline_view(item: &AuditEventBaseline) -> AuditEventBaselineView {
    AuditEventBaselineView {
        code: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        cost: format!("{:.2}", item.cost),
        approved: format!("{}", item.approved),
        quantity: item.quantity.to_string(),
        currency: item.currency.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        created_at_text: item.created_at.to_string(),
        minimum: format!("{:.2}", item.minimum),
        rate_text: format!("{:.2}", item.rate),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_baseline_views(items: &[AuditEventBaseline]) -> Vec<AuditEventBaselineView> {
    let mut views: Vec<AuditEventBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_baseline_view(item));
    }
    views
}

pub fn sum_audit_event_baseline_view(view: &AuditEventBaselineView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_cost + (parsed_quantity as f64) + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_baseline("code-231", "title-257");
        let view = to_audit_event_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
