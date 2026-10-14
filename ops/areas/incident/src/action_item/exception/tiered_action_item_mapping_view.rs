use super::*;

use acme_kit::text;

/// How screens and exports show a tiered action item mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredActionItemMappingView {
    pub code: String,
    pub title_text: String,
    pub limit: String,
    pub pinned_text: String,
    pub window_days: String,
    pub ends_at_text: String,
    pub revision: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_tiered_action_item_mapping_view(item: &TieredActionItemMapping) -> TieredActionItemMappingView {
    TieredActionItemMappingView {
        code: text::truncate(&item.code, 40),
        title_text: item.title.clone(),
        limit: item.limit.to_string(),
        pinned_text: format!("{}", item.pinned),
        window_days: item.window_days.to_string(),
        ends_at_text: item.ends_at.to_string(),
        revision: item.revision.to_string(),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_action_item_mapping_views(items: &[TieredActionItemMapping]) -> Vec<TieredActionItemMappingView> {
    let mut views: Vec<TieredActionItemMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_action_item_mapping_view(item));
    }
    views
}

pub fn sum_tiered_action_item_mapping_view(view: &TieredActionItemMappingView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_window_days as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_action_item_mapping("code-103", "title-832");
        let view = to_tiered_action_item_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_action_item_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
