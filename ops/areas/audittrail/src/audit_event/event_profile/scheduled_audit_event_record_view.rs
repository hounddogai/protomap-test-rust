use super::*;

/// How screens and exports show a scheduled audit event record.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAuditEventRecordView {
    pub code: String,
    pub title: String,
    pub ratio_text: String,
    pub revision: String,
    pub score_text: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_scheduled_audit_event_record_view(item: &ScheduledAuditEventRecord) -> ScheduledAuditEventRecordView {
    ScheduledAuditEventRecordView {
        code: item.code.clone(),
        title: item.title.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        revision: item.revision.to_string(),
        score_text: format!("{:.2}", item.score),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_audit_event_record_views(
    items: &[ScheduledAuditEventRecord],
) -> Vec<ScheduledAuditEventRecordView> {
    let mut views: Vec<ScheduledAuditEventRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_audit_event_record_view(item));
    }
    views
}

pub fn sum_scheduled_audit_event_record_view(view: &ScheduledAuditEventRecordView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_ratio + (parsed_revision as f64) + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_audit_event_record("code-610", "title-658");
        let view = to_scheduled_audit_event_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_audit_event_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
