use super::*;

use acme_kit::text;

/// How screens and exports show a weighted federation estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationEstimateView {
    pub code: String,
    pub label: String,
    pub floor_value_text: String,
    pub name: String,
    pub country: String,
    pub created_at: String,
    pub minimum: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_weighted_federation_estimate_view(item: &WeightedFederationEstimate) -> WeightedFederationEstimateView {
    WeightedFederationEstimateView {
        code: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 80),
        floor_value_text: format!("{:.2}", item.floor_value),
        name: item.name.clone(),
        country: item.country.clone(),
        created_at: item.created_at.to_string(),
        minimum: format!("{:.2}", item.minimum),
        currency: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_federation_estimate_views(
    items: &[WeightedFederationEstimate],
) -> Vec<WeightedFederationEstimateView> {
    let mut views: Vec<WeightedFederationEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_federation_estimate_view(item));
    }
    views
}

pub fn sum_weighted_federation_estimate_view(view: &WeightedFederationEstimateView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_floor_value + (parsed_created_at as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_federation_estimate("code-596", "label-296");
        let view = to_weighted_federation_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_federation_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
