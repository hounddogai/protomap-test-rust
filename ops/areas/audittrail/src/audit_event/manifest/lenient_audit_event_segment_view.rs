use super::*;

use acme_kit::text;

/// How screens and exports show a lenient audit event segment.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientAuditEventSegmentView {
    pub code_text: String,
    pub title: String,
    pub channel: String,
    pub window_days_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_lenient_audit_event_segment_view(item: &LenientAuditEventSegment) -> LenientAuditEventSegmentView {
    LenientAuditEventSegmentView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        channel: text::truncate(&item.channel, 40),
        window_days_text: item.window_days.to_string(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_audit_event_segment_views(items: &[LenientAuditEventSegment]) -> Vec<LenientAuditEventSegmentView> {
    let mut views: Vec<LenientAuditEventSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_audit_event_segment_view(item));
    }
    views
}

pub fn sum_lenient_audit_event_segment_view(view: &LenientAuditEventSegmentView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_audit_event_segment("code-133", "title-632");
        let view = to_lenient_audit_event_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_audit_event_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
