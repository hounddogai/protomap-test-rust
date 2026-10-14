use super::*;

use acme_kit::text;

/// How screens and exports show a audit event allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAllocationView {
    pub code_text: String,
    pub title: String,
    pub margin: String,
    pub weight: String,
    pub name: String,
    pub currency: String,
    pub reviewer_text: String,
    pub status_label: String,
}

pub fn to_audit_event_allocation_view(item: &AuditEventAllocation) -> AuditEventAllocationView {
    AuditEventAllocationView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        margin: format!("{:.2}", item.margin),
        weight: format!("{:.2}", item.weight),
        name: item.name.clone(),
        currency: item.currency.clone(),
        reviewer_text: text::truncate(&item.reviewer, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_allocation_views(items: &[AuditEventAllocation]) -> Vec<AuditEventAllocationView> {
    let mut views: Vec<AuditEventAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_allocation_view(item));
    }
    views
}

pub fn sum_audit_event_allocation_view(view: &AuditEventAllocationView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_margin + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_allocation("code-371", "title-27");
        let view = to_audit_event_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
