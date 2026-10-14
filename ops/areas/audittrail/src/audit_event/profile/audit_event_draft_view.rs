use super::*;

use acme_kit::text;

/// How screens and exports show a audit event draft.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventDraftView {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub active: String,
    pub currency: String,
    pub priority: String,
    pub port: String,
    pub capacity_text: String,
    pub visible: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_audit_event_draft_view(item: &AuditEventDraft) -> AuditEventDraftView {
    AuditEventDraftView {
        code: item.code.clone(),
        name: item.name.clone(),
        external_ref: text::truncate(&item.external_ref, 40),
        active: format!("{}", item.active),
        currency: text::truncate(&item.currency, 80),
        priority: item.priority.to_string(),
        port: item.port.to_string(),
        capacity_text: item.capacity.to_string(),
        visible: format!("{}", item.visible),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_draft_views(items: &[AuditEventDraft]) -> Vec<AuditEventDraftView> {
    let mut views: Vec<AuditEventDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_draft_view(item));
    }
    views
}

pub fn sum_audit_event_draft_view(view: &AuditEventDraftView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_port as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_draft("code-729", "name-336");
        let view = to_audit_event_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
