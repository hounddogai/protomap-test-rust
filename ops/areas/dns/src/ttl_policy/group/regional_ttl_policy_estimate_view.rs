use super::*;

use acme_kit::text;

/// How screens and exports show a regional ttl policy estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTtlPolicyEstimateView {
    pub code_text: String,
    pub label: String,
    pub grace_days: String,
    pub audience_text: String,
    pub rotation_days: String,
    pub updated_at: String,
    pub version: String,
    pub locale_text: String,
    pub visible: String,
    pub fingerprint_text: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_regional_ttl_policy_estimate_view(item: &RegionalTtlPolicyEstimate) -> RegionalTtlPolicyEstimateView {
    RegionalTtlPolicyEstimateView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        grace_days: item.grace_days.to_string(),
        audience_text: text::truncate(&item.audience, 40),
        rotation_days: item.rotation_days.to_string(),
        updated_at: item.updated_at.to_string(),
        version: item.version.to_string(),
        locale_text: item.locale.clone(),
        visible: format!("{}", item.visible),
        fingerprint_text: item.fingerprint.clone(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_ttl_policy_estimate_views(
    items: &[RegionalTtlPolicyEstimate],
) -> Vec<RegionalTtlPolicyEstimateView> {
    let mut views: Vec<RegionalTtlPolicyEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_ttl_policy_estimate_view(item));
    }
    views
}

pub fn sum_regional_ttl_policy_estimate_view(view: &RegionalTtlPolicyEstimateView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_rotation_days as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_ttl_policy_estimate("code-776", "label-33");
        let view = to_regional_ttl_policy_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_ttl_policy_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
