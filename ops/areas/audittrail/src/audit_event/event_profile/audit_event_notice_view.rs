use super::*;

use acme_kit::text;

/// How screens and exports show a audit event notice.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventNoticeView {
    pub code: String,
    pub title_text: String,
    pub margin: String,
    pub summary: String,
    pub floor_value: String,
    pub ratio: String,
    pub maximum: String,
    pub contact_email: String,
    pub currency_text: String,
    pub status_label: String,
}

pub fn to_audit_event_notice_view(item: &AuditEventNotice) -> AuditEventNoticeView {
    AuditEventNoticeView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        margin: format!("{:.2}", item.margin),
        summary: item.summary.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        ratio: format!("{:.2}", item.ratio),
        maximum: format!("{:.2}", item.maximum),
        contact_email: text::truncate(&item.contact_email, 64),
        currency_text: text::truncate(&item.currency, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_notice_views(items: &[AuditEventNotice]) -> Vec<AuditEventNoticeView> {
    let mut views: Vec<AuditEventNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_notice_view(item));
    }
    views
}

pub fn sum_audit_event_notice_view(view: &AuditEventNoticeView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_margin + parsed_floor_value + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_notice("code-427", "title-812");
        let view = to_audit_event_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
