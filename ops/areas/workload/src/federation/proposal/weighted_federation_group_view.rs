use super::*;

use acme_kit::text;

/// How screens and exports show a weighted federation group.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationGroupView {
    pub code: String,
    pub name: String,
    pub window_days: String,
    pub pinned_text: String,
    pub region: String,
    pub weight_text: String,
    pub status_label: String,
}

pub fn to_weighted_federation_group_view(item: &WeightedFederationGroup) -> WeightedFederationGroupView {
    WeightedFederationGroupView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        window_days: item.window_days.to_string(),
        pinned_text: format!("{}", item.pinned),
        region: item.region.clone(),
        weight_text: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_federation_group_views(items: &[WeightedFederationGroup]) -> Vec<WeightedFederationGroupView> {
    let mut views: Vec<WeightedFederationGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_federation_group_view(item));
    }
    views
}

pub fn sum_weighted_federation_group_view(view: &WeightedFederationGroupView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_federation_group("code-167", "name-947");
        let view = to_weighted_federation_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_federation_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
