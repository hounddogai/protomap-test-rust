use super::*;

use acme_kit::text;

/// How screens and exports show a global alert audit.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertAuditView {
    pub code: String,
    pub label: String,
    pub rate: String,
    pub cluster_text: String,
    pub ttl_seconds: String,
    pub issuer_text: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_global_alert_audit_view(item: &GlobalAlertAudit) -> GlobalAlertAuditView {
    GlobalAlertAuditView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        rate: format!("{:.2}", item.rate),
        cluster_text: item.cluster.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        issuer_text: item.issuer.clone(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_alert_audit_views(items: &[GlobalAlertAudit]) -> Vec<GlobalAlertAuditView> {
    let mut views: Vec<GlobalAlertAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_alert_audit_view(item));
    }
    views
}

pub fn sum_global_alert_audit_view(view: &GlobalAlertAuditView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_rate + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_alert_audit("code-123", "label-247");
        let view = to_global_alert_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_alert_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
