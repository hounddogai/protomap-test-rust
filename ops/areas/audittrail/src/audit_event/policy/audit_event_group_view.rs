use super::*;

use acme_kit::text;

/// How screens and exports show a audit event group.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventGroupView {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub issuer: String,
    pub retries: String,
    pub threshold_text: String,
    pub region: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_audit_event_group_view(item: &AuditEventGroup) -> AuditEventGroupView {
    AuditEventGroupView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        owner_id: item.owner_id.clone(),
        issuer: item.issuer.clone(),
        retries: item.retries.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        region: item.region.clone(),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_group_views(items: &[AuditEventGroup]) -> Vec<AuditEventGroupView> {
    let mut views: Vec<AuditEventGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_group_view(item));
    }
    views
}

pub fn sum_audit_event_group_view(view: &AuditEventGroupView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_retries as f64 + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_group("code-158", "label-870");
        let view = to_audit_event_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
