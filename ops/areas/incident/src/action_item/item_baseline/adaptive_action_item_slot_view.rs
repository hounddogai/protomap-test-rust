use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive action item slot.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveActionItemSlotView {
    pub code: String,
    pub title: String,
    pub capacity: String,
    pub cost: String,
    pub quantity_text: String,
    pub region: String,
    pub scope: String,
    pub label_text: String,
    pub burst: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_adaptive_action_item_slot_view(item: &AdaptiveActionItemSlot) -> AdaptiveActionItemSlotView {
    AdaptiveActionItemSlotView {
        code: item.code.clone(),
        title: item.title.clone(),
        capacity: item.capacity.to_string(),
        cost: format!("{:.2}", item.cost),
        quantity_text: item.quantity.to_string(),
        region: item.region.clone(),
        scope: item.scope.clone(),
        label_text: text::truncate(&item.label, 80),
        burst: item.burst.to_string(),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_action_item_slot_views(items: &[AdaptiveActionItemSlot]) -> Vec<AdaptiveActionItemSlotView> {
    let mut views: Vec<AdaptiveActionItemSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_action_item_slot_view(item));
    }
    views
}

pub fn sum_adaptive_action_item_slot_view(view: &AdaptiveActionItemSlotView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_cost + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_action_item_slot("code-490", "title-436");
        let view = to_adaptive_action_item_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_action_item_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
