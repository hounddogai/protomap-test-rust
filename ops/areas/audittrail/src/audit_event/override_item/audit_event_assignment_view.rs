use super::*;

use acme_kit::text;

/// How screens and exports show a audit event assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAssignmentView {
    pub code: String,
    pub name: String,
    pub revision: String,
    pub batch_size: String,
    pub category: String,
    pub issuer: String,
    pub ceiling_text: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_audit_event_assignment_view(item: &AuditEventAssignment) -> AuditEventAssignmentView {
    AuditEventAssignmentView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        revision: item.revision.to_string(),
        batch_size: item.batch_size.to_string(),
        category: item.category.clone(),
        issuer: text::truncate(&item.issuer, 40),
        ceiling_text: format!("{:.2}", item.ceiling),
        locale: text::truncate(&item.locale, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_assignment_views(items: &[AuditEventAssignment]) -> Vec<AuditEventAssignmentView> {
    let mut views: Vec<AuditEventAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_assignment_view(item));
    }
    views
}

pub fn sum_audit_event_assignment_view(view: &AuditEventAssignmentView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_batch_size as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_assignment("code-348", "name-679");
        let view = to_audit_event_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
