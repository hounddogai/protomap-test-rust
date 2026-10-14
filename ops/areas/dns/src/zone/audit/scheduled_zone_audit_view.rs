use super::*;

/// How screens and exports show a scheduled zone audit.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneAuditView {
    pub code: String,
    pub label_text: String,
    pub approved: String,
    pub duration_minutes: String,
    pub expires_at: String,
    pub namespace_text: String,
    pub limit_text: String,
    pub contact_email: String,
    pub floor_value_text: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_scheduled_zone_audit_view(item: &ScheduledZoneAudit) -> ScheduledZoneAuditView {
    ScheduledZoneAuditView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        approved: format!("{}", item.approved),
        duration_minutes: item.duration_minutes.to_string(),
        expires_at: item.expires_at.to_string(),
        namespace_text: item.namespace.clone(),
        limit_text: item.limit.to_string(),
        contact_email: item.contact_email.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_zone_audit_views(items: &[ScheduledZoneAudit]) -> Vec<ScheduledZoneAuditView> {
    let mut views: Vec<ScheduledZoneAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_zone_audit_view(item));
    }
    views
}

pub fn sum_scheduled_zone_audit_view(view: &ScheduledZoneAuditView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_expires_at as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_zone_audit("code-338", "label-939");
        let view = to_scheduled_zone_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_zone_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
