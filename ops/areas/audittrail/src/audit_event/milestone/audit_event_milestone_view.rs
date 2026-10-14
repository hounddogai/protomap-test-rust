use super::*;

/// How screens and exports show a audit event milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventMilestoneView {
    pub code: String,
    pub name: String,
    pub description: String,
    pub segment_text: String,
    pub quota_units: String,
    pub title: String,
    pub expires_at: String,
    pub checksum: String,
    pub key_id: String,
    pub batch_size_text: String,
    pub status_label: String,
}

pub fn to_audit_event_milestone_view(item: &AuditEventMilestone) -> AuditEventMilestoneView {
    AuditEventMilestoneView {
        code: item.code.clone(),
        name: item.name.clone(),
        description: item.description.clone(),
        segment_text: item.segment.clone(),
        quota_units: item.quota_units.to_string(),
        title: item.title.clone(),
        expires_at: item.expires_at.to_string(),
        checksum: item.checksum.clone(),
        key_id: item.key_id.clone(),
        batch_size_text: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_milestone_views(items: &[AuditEventMilestone]) -> Vec<AuditEventMilestoneView> {
    let mut views: Vec<AuditEventMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_milestone_view(item));
    }
    views
}

pub fn sum_audit_event_milestone_view(view: &AuditEventMilestoneView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_expires_at as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_milestone("code-364", "name-122");
        let view = to_audit_event_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
