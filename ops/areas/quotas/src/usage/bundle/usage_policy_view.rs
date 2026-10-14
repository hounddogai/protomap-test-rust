use super::*;

use acme_kit::text;

/// How screens and exports show a usage policy.
#[derive(Debug, Clone, PartialEq)]
pub struct UsagePolicyView {
    pub code: String,
    pub label_text: String,
    pub retries: String,
    pub tenant_id: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_usage_policy_view(item: &UsagePolicy) -> UsagePolicyView {
    UsagePolicyView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        retries: item.retries.to_string(),
        tenant_id: item.tenant_id.clone(),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_policy_views(items: &[UsagePolicy]) -> Vec<UsagePolicyView> {
    let mut views: Vec<UsagePolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_policy_view(item));
    }
    views
}

pub fn sum_usage_policy_view(view: &UsagePolicyView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_policy("code-474", "label-747");
        let view = to_usage_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
