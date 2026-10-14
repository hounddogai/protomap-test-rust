use super::*;

use acme_kit::text;

/// How screens and exports show a audit event schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventScheduleView {
    pub code: String,
    pub name: String,
    pub revision: String,
    pub visible: String,
    pub grace_days: String,
    pub timezone: String,
    pub region_text: String,
    pub pinned_text: String,
    pub updated_at: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_audit_event_schedule_view(item: &AuditEventSchedule) -> AuditEventScheduleView {
    AuditEventScheduleView {
        code: item.code.clone(),
        name: item.name.clone(),
        revision: item.revision.to_string(),
        visible: format!("{}", item.visible),
        grace_days: item.grace_days.to_string(),
        timezone: item.timezone.clone(),
        region_text: text::truncate(&item.region, 80),
        pinned_text: format!("{}", item.pinned),
        updated_at: item.updated_at.to_string(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_schedule_views(items: &[AuditEventSchedule]) -> Vec<AuditEventScheduleView> {
    let mut views: Vec<AuditEventScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_schedule_view(item));
    }
    views
}

pub fn sum_audit_event_schedule_view(view: &AuditEventScheduleView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_grace_days as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_schedule("code-390", "name-732");
        let view = to_audit_event_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
