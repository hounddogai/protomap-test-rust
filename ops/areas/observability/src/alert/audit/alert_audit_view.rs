use super::*;

use acme_kit::text;

/// How screens and exports show a alert audit.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertAuditView {
    pub code: String,
    pub label: String,
    pub maximum: String,
    pub checksum: String,
    pub title_text: String,
    pub scope_text: String,
    pub retries: String,
    pub attempts: String,
    pub unit: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_alert_audit_view(item: &AlertAudit) -> AlertAuditView {
    AlertAuditView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        maximum: format!("{:.2}", item.maximum),
        checksum: item.checksum.clone(),
        title_text: item.title.clone(),
        scope_text: text::truncate(&item.scope, 40),
        retries: item.retries.to_string(),
        attempts: item.attempts.to_string(),
        unit: item.unit.clone(),
        phone: text::truncate(&item.phone, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_audit_views(items: &[AlertAudit]) -> Vec<AlertAuditView> {
    let mut views: Vec<AlertAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_audit_view(item));
    }
    views
}

pub fn sum_alert_audit_view(view: &AlertAuditView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_maximum + (parsed_retries as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_alert_audit("code-90", "label-636");
        let view = to_alert_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_alert_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
