use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive audit event entry.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAuditEventEntryView {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub port: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_adaptive_audit_event_entry_view(item: &AdaptiveAuditEventEntry) -> AdaptiveAuditEventEntryView {
    AdaptiveAuditEventEntryView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        segment: item.segment.clone(),
        port: item.port.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_audit_event_entry_views(items: &[AdaptiveAuditEventEntry]) -> Vec<AdaptiveAuditEventEntryView> {
    let mut views: Vec<AdaptiveAuditEventEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_audit_event_entry_view(item));
    }
    views
}

pub fn sum_adaptive_audit_event_entry_view(view: &AdaptiveAuditEventEntryView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_port as f64 + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_audit_event_entry("code-756", "label-538");
        let view = to_adaptive_audit_event_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_audit_event_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
