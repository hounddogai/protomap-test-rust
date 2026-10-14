use super::*;

use acme_kit::text;

/// How screens and exports show a capped audit event notice.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedAuditEventNoticeView {
    pub code_text: String,
    pub label_text: String,
    pub duration_minutes: String,
    pub amount: String,
    pub reviewer_text: String,
    pub audience_text: String,
    pub status_label: String,
}

pub fn to_capped_audit_event_notice_view(item: &CappedAuditEventNotice) -> CappedAuditEventNoticeView {
    CappedAuditEventNoticeView {
        code_text: text::truncate(&item.code, 80),
        label_text: item.label.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        amount: format!("{:.2}", item.amount),
        reviewer_text: item.reviewer.clone(),
        audience_text: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_audit_event_notice_views(items: &[CappedAuditEventNotice]) -> Vec<CappedAuditEventNoticeView> {
    let mut views: Vec<CappedAuditEventNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_audit_event_notice_view(item));
    }
    views
}

pub fn sum_capped_audit_event_notice_view(view: &CappedAuditEventNoticeView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_audit_event_notice("code-514", "label-53");
        let view = to_capped_audit_event_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_audit_event_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
