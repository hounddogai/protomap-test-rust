use super::*;

use acme_kit::text;

/// How screens and exports show a weighted rollout profile.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRolloutProfileView {
    pub code_text: String,
    pub name: String,
    pub discount_text: String,
    pub reviewer_text: String,
    pub source_text: String,
    pub ceiling_text: String,
    pub approved: String,
    pub weight: String,
    pub burst_text: String,
    pub status_label: String,
}

pub fn to_weighted_rollout_profile_view(item: &WeightedRolloutProfile) -> WeightedRolloutProfileView {
    WeightedRolloutProfileView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 80),
        discount_text: format!("{:.2}", item.discount),
        reviewer_text: item.reviewer.clone(),
        source_text: item.source.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        approved: format!("{}", item.approved),
        weight: format!("{:.2}", item.weight),
        burst_text: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_rollout_profile_views(items: &[WeightedRolloutProfile]) -> Vec<WeightedRolloutProfileView> {
    let mut views: Vec<WeightedRolloutProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_rollout_profile_view(item));
    }
    views
}

pub fn sum_weighted_rollout_profile_view(view: &WeightedRolloutProfileView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_discount + parsed_ceiling + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_rollout_profile("code-769", "name-852");
        let view = to_weighted_rollout_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_rollout_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
