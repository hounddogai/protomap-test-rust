use super::*;

use acme_kit::text;

/// How screens and exports show a weighted peering estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPeeringEstimateView {
    pub code_text: String,
    pub title: String,
    pub cipher: String,
    pub description: String,
    pub scope_text: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_weighted_peering_estimate_view(item: &WeightedPeeringEstimate) -> WeightedPeeringEstimateView {
    WeightedPeeringEstimateView {
        code_text: text::truncate(&item.code, 40),
        title: item.title.clone(),
        cipher: item.cipher.clone(),
        description: item.description.clone(),
        scope_text: text::truncate(&item.scope, 64),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_peering_estimate_views(items: &[WeightedPeeringEstimate]) -> Vec<WeightedPeeringEstimateView> {
    let mut views: Vec<WeightedPeeringEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_peering_estimate_view(item));
    }
    views
}

pub fn sum_weighted_peering_estimate_view(view: &WeightedPeeringEstimateView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_peering_estimate("code-490", "title-245");
        let view = to_weighted_peering_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_peering_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
