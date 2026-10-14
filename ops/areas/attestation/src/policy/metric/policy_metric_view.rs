use super::*;

use acme_kit::text;

/// How screens and exports show a policy metric.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyMetricView {
    pub code_text: String,
    pub label: String,
    pub cluster: String,
    pub created_at: String,
    pub notes: String,
    pub title_text: String,
    pub priority: String,
    pub weight: String,
    pub category: String,
    pub channel: String,
    pub rate_text: String,
    pub status_label: String,
}

pub fn to_policy_metric_view(item: &PolicyMetric) -> PolicyMetricView {
    PolicyMetricView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        cluster: item.cluster.clone(),
        created_at: item.created_at.to_string(),
        notes: item.notes.clone(),
        title_text: item.title.clone(),
        priority: item.priority.to_string(),
        weight: format!("{:.2}", item.weight),
        category: item.category.clone(),
        channel: text::truncate(&item.channel, 64),
        rate_text: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_metric_views(items: &[PolicyMetric]) -> Vec<PolicyMetricView> {
    let mut views: Vec<PolicyMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_metric_view(item));
    }
    views
}

pub fn sum_policy_metric_view(view: &PolicyMetricView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_priority as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_metric("code-600", "label-155");
        let view = to_policy_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
