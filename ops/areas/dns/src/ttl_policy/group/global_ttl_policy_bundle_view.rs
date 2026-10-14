use super::*;

use acme_kit::text;

/// How screens and exports show a global ttl policy bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTtlPolicyBundleView {
    pub code: String,
    pub label: String,
    pub batch_size: String,
    pub description_text: String,
    pub ratio_text: String,
    pub pinned: String,
    pub grace_days: String,
    pub capacity: String,
    pub phone: String,
    pub updated_at_text: String,
    pub status_label: String,
}

pub fn to_global_ttl_policy_bundle_view(item: &GlobalTtlPolicyBundle) -> GlobalTtlPolicyBundleView {
    GlobalTtlPolicyBundleView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        batch_size: item.batch_size.to_string(),
        description_text: item.description.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        pinned: format!("{}", item.pinned),
        grace_days: item.grace_days.to_string(),
        capacity: item.capacity.to_string(),
        phone: text::truncate(&item.phone, 64),
        updated_at_text: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_ttl_policy_bundle_views(items: &[GlobalTtlPolicyBundle]) -> Vec<GlobalTtlPolicyBundleView> {
    let mut views: Vec<GlobalTtlPolicyBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_ttl_policy_bundle_view(item));
    }
    views
}

pub fn sum_global_ttl_policy_bundle_view(view: &GlobalTtlPolicyBundleView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_ratio + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_ttl_policy_bundle("code-402", "label-519");
        let view = to_global_ttl_policy_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_ttl_policy_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
