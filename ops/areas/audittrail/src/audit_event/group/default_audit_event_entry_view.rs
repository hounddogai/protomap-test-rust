use super::*;

use acme_kit::text;

/// How screens and exports show a default audit event entry.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAuditEventEntryView {
    pub code: String,
    pub label_text: String,
    pub rotation_days_text: String,
    pub category: String,
    pub active_text: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_default_audit_event_entry_view(item: &DefaultAuditEventEntry) -> DefaultAuditEventEntryView {
    DefaultAuditEventEntryView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        rotation_days_text: item.rotation_days.to_string(),
        category: item.category.clone(),
        active_text: format!("{}", item.active),
        title: text::truncate(&item.title, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_audit_event_entry_views(items: &[DefaultAuditEventEntry]) -> Vec<DefaultAuditEventEntryView> {
    let mut views: Vec<DefaultAuditEventEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_audit_event_entry_view(item));
    }
    views
}

pub fn sum_default_audit_event_entry_view(view: &DefaultAuditEventEntryView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_rotation_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_audit_event_entry("code-377", "label-176");
        let view = to_default_audit_event_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_audit_event_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
