use super::*;

use acme_kit::text;

/// How screens and exports show a primary action item baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemBaselineView {
    pub code: String,
    pub title_text: String,
    pub quota_units_text: String,
    pub priority: String,
    pub grace_days: String,
    pub country: String,
    pub updated_at_text: String,
    pub ceiling: String,
    pub pinned_text: String,
    pub status_label: String,
}

pub fn to_primary_action_item_baseline_view(item: &PrimaryActionItemBaseline) -> PrimaryActionItemBaselineView {
    PrimaryActionItemBaselineView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 80),
        quota_units_text: item.quota_units.to_string(),
        priority: item.priority.to_string(),
        grace_days: item.grace_days.to_string(),
        country: item.country.clone(),
        updated_at_text: item.updated_at.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        pinned_text: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_action_item_baseline_views(
    items: &[PrimaryActionItemBaseline],
) -> Vec<PrimaryActionItemBaselineView> {
    let mut views: Vec<PrimaryActionItemBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_action_item_baseline_view(item));
    }
    views
}

pub fn sum_primary_action_item_baseline_view(view: &PrimaryActionItemBaselineView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_priority as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_action_item_baseline("code-933", "title-185");
        let view = to_primary_action_item_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_action_item_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
