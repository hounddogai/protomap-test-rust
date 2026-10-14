use super::*;

/// How screens and exports show a seasonal audit event notice.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAuditEventNoticeView {
    pub code_text: String,
    pub name: String,
    pub attempts: String,
    pub ends_at_text: String,
    pub pinned_text: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_seasonal_audit_event_notice_view(item: &SeasonalAuditEventNotice) -> SeasonalAuditEventNoticeView {
    SeasonalAuditEventNoticeView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        attempts: item.attempts.to_string(),
        ends_at_text: item.ends_at.to_string(),
        pinned_text: format!("{}", item.pinned),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_audit_event_notice_views(items: &[SeasonalAuditEventNotice]) -> Vec<SeasonalAuditEventNoticeView> {
    let mut views: Vec<SeasonalAuditEventNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_audit_event_notice_view(item));
    }
    views
}

pub fn sum_seasonal_audit_event_notice_view(view: &SeasonalAuditEventNoticeView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_audit_event_notice("code-319", "name-139");
        let view = to_seasonal_audit_event_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_audit_event_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
