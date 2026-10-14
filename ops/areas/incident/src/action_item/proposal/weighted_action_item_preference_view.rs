use super::*;

use acme_kit::text;

/// How screens and exports show a weighted action item preference.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedActionItemPreferenceView {
    pub code: String,
    pub name: String,
    pub capacity: String,
    pub ceiling_text: String,
    pub minimum: String,
    pub visible: String,
    pub segment_text: String,
    pub priority_text: String,
    pub title: String,
    pub principal_text: String,
    pub status_label: String,
}

pub fn to_weighted_action_item_preference_view(
    item: &WeightedActionItemPreference,
) -> WeightedActionItemPreferenceView {
    WeightedActionItemPreferenceView {
        code: item.code.clone(),
        name: item.name.clone(),
        capacity: item.capacity.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        minimum: format!("{:.2}", item.minimum),
        visible: format!("{}", item.visible),
        segment_text: text::truncate(&item.segment, 80),
        priority_text: item.priority.to_string(),
        title: item.title.clone(),
        principal_text: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_action_item_preference_views(
    items: &[WeightedActionItemPreference],
) -> Vec<WeightedActionItemPreferenceView> {
    let mut views: Vec<WeightedActionItemPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_action_item_preference_view(item));
    }
    views
}

pub fn sum_weighted_action_item_preference_view(view: &WeightedActionItemPreferenceView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_ceiling + parsed_minimum)
}
