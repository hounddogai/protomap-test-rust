use super::*;

use acme_kit::text;

/// How screens and exports show a audit event target.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventTargetView {
    pub code_text: String,
    pub label: String,
    pub ttl_seconds: String,
    pub checksum: String,
    pub approved: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_audit_event_target_view(item: &AuditEventTarget) -> AuditEventTargetView {
    AuditEventTargetView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 40),
        ttl_seconds: item.ttl_seconds.to_string(),
        checksum: item.checksum.clone(),
        approved: format!("{}", item.approved),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_target_views(items: &[AuditEventTarget]) -> Vec<AuditEventTargetView> {
    let mut views: Vec<AuditEventTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_target_view(item));
    }
    views
}

pub fn sum_audit_event_target_view(view: &AuditEventTargetView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_ttl_seconds as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_target("code-163", "label-357");
        let view = to_audit_event_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
