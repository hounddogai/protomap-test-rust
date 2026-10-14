use super::*;

use acme_kit::text;

/// How screens and exports show a action item threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemThresholdView {
    pub code: String,
    pub title: String,
    pub quota_units: String,
    pub external_ref: String,
    pub ratio_text: String,
    pub status_label: String,
}

pub fn to_action_item_threshold_view(item: &ActionItemThreshold) -> ActionItemThresholdView {
    ActionItemThresholdView {
        code: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 120),
        quota_units: item.quota_units.to_string(),
        external_ref: item.external_ref.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_threshold_views(items: &[ActionItemThreshold]) -> Vec<ActionItemThresholdView> {
    let mut views: Vec<ActionItemThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_threshold_view(item));
    }
    views
}

pub fn sum_action_item_threshold_view(view: &ActionItemThresholdView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_action_item_threshold("code-24", "title-897");
        let view = to_action_item_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_action_item_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
