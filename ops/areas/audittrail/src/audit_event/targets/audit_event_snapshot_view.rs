use super::*;

use acme_kit::text;

/// How screens and exports show a audit event snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSnapshotView {
    pub code: String,
    pub label: String,
    pub contact_email_text: String,
    pub priority: String,
    pub ratio: String,
    pub revision: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_audit_event_snapshot_view(item: &AuditEventSnapshot) -> AuditEventSnapshotView {
    AuditEventSnapshotView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        contact_email_text: item.contact_email.clone(),
        priority: item.priority.to_string(),
        ratio: format!("{:.2}", item.ratio),
        revision: item.revision.to_string(),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_snapshot_views(items: &[AuditEventSnapshot]) -> Vec<AuditEventSnapshotView> {
    let mut views: Vec<AuditEventSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_snapshot_view(item));
    }
    views
}

pub fn sum_audit_event_snapshot_view(view: &AuditEventSnapshotView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_priority as f64 + parsed_ratio + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_snapshot("code-420", "label-433");
        let view = to_audit_event_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
