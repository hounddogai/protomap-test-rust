use super::*;

use acme_kit::text;

/// How screens and exports show a action item estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemEstimateView {
    pub code: String,
    pub title_text: String,
    pub checksum_text: String,
    pub unit: String,
    pub starts_at_text: String,
    pub maximum_text: String,
    pub archived: String,
    pub rotation_days_text: String,
    pub category: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_action_item_estimate_view(item: &ActionItemEstimate) -> ActionItemEstimateView {
    ActionItemEstimateView {
        code: text::truncate(&item.code, 64),
        title_text: text::truncate(&item.title, 40),
        checksum_text: text::truncate(&item.checksum, 64),
        unit: text::truncate(&item.unit, 80),
        starts_at_text: item.starts_at.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        archived: format!("{}", item.archived),
        rotation_days_text: item.rotation_days.to_string(),
        category: text::truncate(&item.category, 40),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_estimate_views(items: &[ActionItemEstimate]) -> Vec<ActionItemEstimateView> {
    let mut views: Vec<ActionItemEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_estimate_view(item));
    }
    views
}

pub fn sum_action_item_estimate_view(view: &ActionItemEstimateView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_maximum + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_action_item_estimate("code-594", "title-445");
        let view = to_action_item_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_action_item_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
