use super::*;

use acme_kit::text;

/// How screens and exports show a retention policy.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionPolicyView {
    pub code: String,
    pub label_text: String,
    pub quota_units: String,
    pub weight: String,
    pub created_at_text: String,
    pub attempts: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_retention_policy_view(item: &RetentionPolicy) -> RetentionPolicyView {
    RetentionPolicyView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 64),
        quota_units: item.quota_units.to_string(),
        weight: format!("{:.2}", item.weight),
        created_at_text: item.created_at.to_string(),
        attempts: item.attempts.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_policy_views(items: &[RetentionPolicy]) -> Vec<RetentionPolicyView> {
    let mut views: Vec<RetentionPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_policy_view(item));
    }
    views
}

pub fn sum_retention_policy_view(view: &RetentionPolicyView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_weight + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_policy("code-458", "label-716");
        let view = to_retention_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
