use super::*;

use acme_kit::text;

/// How screens and exports show a regional action item preference.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemPreferenceView {
    pub code: String,
    pub name_text: String,
    pub region: String,
    pub issuer_text: String,
    pub threshold: String,
    pub weight: String,
    pub title: String,
    pub label_text: String,
    pub visible: String,
    pub quota_units: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_regional_action_item_preference_view(
    item: &RegionalActionItemPreference,
) -> RegionalActionItemPreferenceView {
    RegionalActionItemPreferenceView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        region: text::truncate(&item.region, 64),
        issuer_text: item.issuer.clone(),
        threshold: format!("{:.2}", item.threshold),
        weight: format!("{:.2}", item.weight),
        title: text::truncate(&item.title, 64),
        label_text: item.label.clone(),
        visible: format!("{}", item.visible),
        quota_units: item.quota_units.to_string(),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_action_item_preference_views(
    items: &[RegionalActionItemPreference],
) -> Vec<RegionalActionItemPreferenceView> {
    let mut views: Vec<RegionalActionItemPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_action_item_preference_view(item));
    }
    views
}

pub fn sum_regional_action_item_preference_view(view: &RegionalActionItemPreferenceView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_threshold + parsed_weight + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_action_item_preference("code-99", "name-628");
        let view = to_regional_action_item_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_action_item_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
