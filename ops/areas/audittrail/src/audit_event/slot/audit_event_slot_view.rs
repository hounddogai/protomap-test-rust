use super::*;

use acme_kit::text;

/// How screens and exports show a audit event slot.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSlotView {
    pub code_text: String,
    pub name: String,
    pub amount_text: String,
    pub reviewer: String,
    pub cost: String,
    pub budget: String,
    pub created_at: String,
    pub unit_text: String,
    pub rotation_days: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_audit_event_slot_view(item: &AuditEventSlot) -> AuditEventSlotView {
    AuditEventSlotView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 120),
        amount_text: format!("{:.2}", item.amount),
        reviewer: item.reviewer.clone(),
        cost: format!("{:.2}", item.cost),
        budget: format!("{:.2}", item.budget),
        created_at: item.created_at.to_string(),
        unit_text: text::truncate(&item.unit, 64),
        rotation_days: item.rotation_days.to_string(),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_slot_views(items: &[AuditEventSlot]) -> Vec<AuditEventSlotView> {
    let mut views: Vec<AuditEventSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_slot_view(item));
    }
    views
}

pub fn sum_audit_event_slot_view(view: &AuditEventSlotView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_amount + parsed_cost + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_slot("code-914", "name-192");
        let view = to_audit_event_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
