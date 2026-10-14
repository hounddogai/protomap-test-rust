use super::*;

use acme_kit::text;

/// How screens and exports show a identity estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityEstimateView {
    pub code: String,
    pub label: String,
    pub floor_value: String,
    pub updated_at: String,
    pub owner_id: String,
    pub score: String,
    pub unit: String,
    pub name: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_identity_estimate_view(item: &IdentityEstimate) -> IdentityEstimateView {
    IdentityEstimateView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        updated_at: item.updated_at.to_string(),
        owner_id: item.owner_id.clone(),
        score: format!("{:.2}", item.score),
        unit: item.unit.clone(),
        name: item.name.clone(),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_estimate_views(items: &[IdentityEstimate]) -> Vec<IdentityEstimateView> {
    let mut views: Vec<IdentityEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_estimate_view(item));
    }
    views
}

pub fn sum_identity_estimate_view(view: &IdentityEstimateView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_floor_value + (parsed_updated_at as f64) + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_estimate("code-123", "label-677");
        let view = to_identity_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
