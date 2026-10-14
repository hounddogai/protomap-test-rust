use super::*;

use acme_kit::text;

/// How screens and exports show a rollback slot.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSlotView {
    pub code_text: String,
    pub label: String,
    pub active_text: String,
    pub description: String,
    pub unit_text: String,
    pub timezone: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_rollback_slot_view(item: &RollbackSlot) -> RollbackSlotView {
    RollbackSlotView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 80),
        active_text: format!("{}", item.active),
        description: item.description.clone(),
        unit_text: text::truncate(&item.unit, 64),
        timezone: text::truncate(&item.timezone, 64),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_slot_views(items: &[RollbackSlot]) -> Vec<RollbackSlotView> {
    let mut views: Vec<RollbackSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_slot_view(item));
    }
    views
}

pub fn sum_rollback_slot_view(view: &RollbackSlotView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_slot("code-513", "label-100");
        let view = to_rollback_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
