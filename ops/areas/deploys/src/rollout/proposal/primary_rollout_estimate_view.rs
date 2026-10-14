use super::*;

use acme_kit::text;

/// How screens and exports show a primary rollout estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutEstimateView {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub approved_text: String,
    pub source: String,
    pub burst: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_primary_rollout_estimate_view(item: &PrimaryRolloutEstimate) -> PrimaryRolloutEstimateView {
    PrimaryRolloutEstimateView {
        code: item.code.clone(),
        name: item.name.clone(),
        cipher: item.cipher.clone(),
        approved_text: format!("{}", item.approved),
        source: text::truncate(&item.source, 40),
        burst: item.burst.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_rollout_estimate_views(items: &[PrimaryRolloutEstimate]) -> Vec<PrimaryRolloutEstimateView> {
    let mut views: Vec<PrimaryRolloutEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_rollout_estimate_view(item));
    }
    views
}

pub fn sum_primary_rollout_estimate_view(view: &PrimaryRolloutEstimateView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_burst as f64 + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_rollout_estimate("code-708", "name-775");
        let view = to_primary_rollout_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_rollout_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
