use super::*;

use acme_kit::text;

/// How screens and exports show a action item slot.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSlotView {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub ttl_seconds: String,
    pub limit: String,
    pub category: String,
    pub ratio: String,
    pub floor_value: String,
    pub summary: String,
    pub maximum_text: String,
    pub status_label: String,
}

pub fn to_action_item_slot_view(item: &ActionItemSlot) -> ActionItemSlotView {
    ActionItemSlotView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 80),
        external_ref: text::truncate(&item.external_ref, 64),
        ttl_seconds: item.ttl_seconds.to_string(),
        limit: item.limit.to_string(),
        category: item.category.clone(),
        ratio: format!("{:.2}", item.ratio),
        floor_value: format!("{:.2}", item.floor_value),
        summary: item.summary.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_slot_views(items: &[ActionItemSlot]) -> Vec<ActionItemSlotView> {
    let mut views: Vec<ActionItemSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_slot_view(item));
    }
    views
}

pub fn sum_action_item_slot_view(view: &ActionItemSlotView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_limit as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_action_item_slot("code-718", "title-612");
        let view = to_action_item_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_action_item_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
