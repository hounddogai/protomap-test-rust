use super::*;

use acme_kit::text;

/// How screens and exports show a weighted policy estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyEstimateView {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub ttl_seconds: String,
    pub external_ref: String,
    pub sequence_text: String,
    pub limit: String,
    pub rate: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_weighted_policy_estimate_view(item: &WeightedPolicyEstimate) -> WeightedPolicyEstimateView {
    WeightedPolicyEstimateView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        key_id: item.key_id.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        external_ref: item.external_ref.clone(),
        sequence_text: item.sequence.to_string(),
        limit: item.limit.to_string(),
        rate: format!("{:.2}", item.rate),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_policy_estimate_views(items: &[WeightedPolicyEstimate]) -> Vec<WeightedPolicyEstimateView> {
    let mut views: Vec<WeightedPolicyEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_policy_estimate_view(item));
    }
    views
}

pub fn sum_weighted_policy_estimate_view(view: &WeightedPolicyEstimateView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_sequence as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_policy_estimate("code-442", "label-498");
        let view = to_weighted_policy_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_policy_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
