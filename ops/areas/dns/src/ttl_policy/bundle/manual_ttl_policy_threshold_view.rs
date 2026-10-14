use super::*;

use acme_kit::text;

/// How screens and exports show a manual ttl policy threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyThresholdView {
    pub code: String,
    pub title: String,
    pub revision: String,
    pub name: String,
    pub quantity: String,
    pub retries_text: String,
    pub locale_text: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_manual_ttl_policy_threshold_view(item: &ManualTtlPolicyThreshold) -> ManualTtlPolicyThresholdView {
    ManualTtlPolicyThresholdView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        revision: item.revision.to_string(),
        name: item.name.clone(),
        quantity: item.quantity.to_string(),
        retries_text: item.retries.to_string(),
        locale_text: item.locale.clone(),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_ttl_policy_threshold_views(items: &[ManualTtlPolicyThreshold]) -> Vec<ManualTtlPolicyThresholdView> {
    let mut views: Vec<ManualTtlPolicyThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_ttl_policy_threshold_view(item));
    }
    views
}

pub fn sum_manual_ttl_policy_threshold_view(view: &ManualTtlPolicyThresholdView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_quantity as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_ttl_policy_threshold("code-942", "title-24");
        let view = to_manual_ttl_policy_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_ttl_policy_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
