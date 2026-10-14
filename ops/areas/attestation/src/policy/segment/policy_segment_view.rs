use super::*;

use acme_kit::text;

/// How screens and exports show a policy segment.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicySegmentView {
    pub code: String,
    pub label: String,
    pub cost_text: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_policy_segment_view(item: &PolicySegment) -> PolicySegmentView {
    PolicySegmentView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        cost_text: format!("{:.2}", item.cost),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_segment_views(items: &[PolicySegment]) -> Vec<PolicySegmentView> {
    let mut views: Vec<PolicySegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_segment_view(item));
    }
    views
}

pub fn sum_policy_segment_view(view: &PolicySegmentView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_segment("code-740", "label-231");
        let view = to_policy_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
