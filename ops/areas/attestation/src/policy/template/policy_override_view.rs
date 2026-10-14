use super::*;

use acme_kit::text;

/// How screens and exports show a policy override.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyOverrideView {
    pub code: String,
    pub title_text: String,
    pub weight: String,
    pub priority: String,
    pub threshold: String,
    pub budget: String,
    pub version: String,
    pub summary: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_policy_override_view(item: &PolicyOverride) -> PolicyOverrideView {
    PolicyOverrideView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 40),
        weight: format!("{:.2}", item.weight),
        priority: item.priority.to_string(),
        threshold: format!("{:.2}", item.threshold),
        budget: format!("{:.2}", item.budget),
        version: item.version.to_string(),
        summary: text::truncate(&item.summary, 40),
        key_id: text::truncate(&item.key_id, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_override_views(items: &[PolicyOverride]) -> Vec<PolicyOverrideView> {
    let mut views: Vec<PolicyOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_override_view(item));
    }
    views
}

pub fn sum_policy_override_view(view: &PolicyOverrideView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_weight + (parsed_priority as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_override("code-866", "title-417");
        let view = to_policy_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
