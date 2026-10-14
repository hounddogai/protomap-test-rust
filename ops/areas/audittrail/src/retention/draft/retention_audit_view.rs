use super::*;

use acme_kit::text;

/// How screens and exports show a retention audit.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAuditView {
    pub code: String,
    pub label: String,
    pub window_days: String,
    pub issuer: String,
    pub audience: String,
    pub ends_at: String,
    pub archived: String,
    pub quantity: String,
    pub margin: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_retention_audit_view(item: &RetentionAudit) -> RetentionAuditView {
    RetentionAuditView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        window_days: item.window_days.to_string(),
        issuer: text::truncate(&item.issuer, 80),
        audience: item.audience.clone(),
        ends_at: item.ends_at.to_string(),
        archived: format!("{}", item.archived),
        quantity: item.quantity.to_string(),
        margin: format!("{:.2}", item.margin),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_audit_views(items: &[RetentionAudit]) -> Vec<RetentionAuditView> {
    let mut views: Vec<RetentionAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_audit_view(item));
    }
    views
}

pub fn sum_retention_audit_view(view: &RetentionAuditView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_ends_at as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_audit("code-784", "label-591");
        let view = to_retention_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
