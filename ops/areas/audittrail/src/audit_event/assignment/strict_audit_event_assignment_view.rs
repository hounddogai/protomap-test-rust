use super::*;

use acme_kit::text;

/// How screens and exports show a strict audit event assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAuditEventAssignmentView {
    pub code: String,
    pub title_text: String,
    pub expires_at_text: String,
    pub country: String,
    pub updated_at: String,
    pub audience: String,
    pub segment: String,
    pub quantity: String,
    pub created_at: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_strict_audit_event_assignment_view(item: &StrictAuditEventAssignment) -> StrictAuditEventAssignmentView {
    StrictAuditEventAssignmentView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        expires_at_text: item.expires_at.to_string(),
        country: item.country.clone(),
        updated_at: item.updated_at.to_string(),
        audience: text::truncate(&item.audience, 64),
        segment: text::truncate(&item.segment, 120),
        quantity: item.quantity.to_string(),
        created_at: item.created_at.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_audit_event_assignment_views(
    items: &[StrictAuditEventAssignment],
) -> Vec<StrictAuditEventAssignmentView> {
    let mut views: Vec<StrictAuditEventAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_audit_event_assignment_view(item));
    }
    views
}

pub fn sum_strict_audit_event_assignment_view(view: &StrictAuditEventAssignmentView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_updated_at as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_audit_event_assignment("code-776", "title-718");
        let view = to_strict_audit_event_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_audit_event_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
