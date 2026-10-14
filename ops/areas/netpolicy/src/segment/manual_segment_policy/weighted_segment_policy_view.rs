use super::*;

use acme_kit::text;

/// How screens and exports show a weighted segment policy.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentPolicyView {
    pub code: String,
    pub label: String,
    pub archived: String,
    pub ends_at: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_weighted_segment_policy_view(item: &WeightedSegmentPolicy) -> WeightedSegmentPolicyView {
    WeightedSegmentPolicyView {
        code: item.code.clone(),
        label: item.label.clone(),
        archived: format!("{}", item.archived),
        ends_at: item.ends_at.to_string(),
        country: text::truncate(&item.country, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_segment_policy_views(items: &[WeightedSegmentPolicy]) -> Vec<WeightedSegmentPolicyView> {
    let mut views: Vec<WeightedSegmentPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_segment_policy_view(item));
    }
    views
}

pub fn sum_weighted_segment_policy_view(view: &WeightedSegmentPolicyView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ends_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_segment_policy("code-284", "label-704");
        let view = to_weighted_segment_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_segment_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
