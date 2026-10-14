use super::*;

use acme_kit::text;

/// How screens and exports show a batched federation summary.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationSummaryView {
    pub code: String,
    pub name: String,
    pub minimum: String,
    pub budget: String,
    pub region: String,
    pub floor_value: String,
    pub rate: String,
    pub archived: String,
    pub threshold_text: String,
    pub status_label: String,
}

pub fn to_batched_federation_summary_view(item: &BatchedFederationSummary) -> BatchedFederationSummaryView {
    BatchedFederationSummaryView {
        code: text::truncate(&item.code, 64),
        name: text::truncate(&item.name, 64),
        minimum: format!("{:.2}", item.minimum),
        budget: format!("{:.2}", item.budget),
        region: item.region.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        rate: format!("{:.2}", item.rate),
        archived: format!("{}", item.archived),
        threshold_text: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_federation_summary_views(items: &[BatchedFederationSummary]) -> Vec<BatchedFederationSummaryView> {
    let mut views: Vec<BatchedFederationSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_federation_summary_view(item));
    }
    views
}

pub fn sum_batched_federation_summary_view(view: &BatchedFederationSummaryView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_minimum + parsed_budget + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_federation_summary("code-345", "name-769");
        let view = to_batched_federation_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_federation_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
