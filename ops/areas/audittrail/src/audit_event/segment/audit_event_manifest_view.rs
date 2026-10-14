use super::*;

use acme_kit::text;

/// How screens and exports show a audit event manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventManifestView {
    pub code_text: String,
    pub title: String,
    pub window_days: String,
    pub grace_days: String,
    pub namespace: String,
    pub currency: String,
    pub maximum: String,
    pub starts_at: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_audit_event_manifest_view(item: &AuditEventManifest) -> AuditEventManifestView {
    AuditEventManifestView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 64),
        window_days: item.window_days.to_string(),
        grace_days: item.grace_days.to_string(),
        namespace: item.namespace.clone(),
        currency: text::truncate(&item.currency, 120),
        maximum: format!("{:.2}", item.maximum),
        starts_at: item.starts_at.to_string(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_manifest_views(items: &[AuditEventManifest]) -> Vec<AuditEventManifestView> {
    let mut views: Vec<AuditEventManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_manifest_view(item));
    }
    views
}

pub fn sum_audit_event_manifest_view(view: &AuditEventManifestView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_grace_days as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_manifest("code-232", "title-611");
        let view = to_audit_event_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
