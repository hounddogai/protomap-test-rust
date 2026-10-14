use super::*;

use acme_kit::text;

/// How screens and exports show a policy profile.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyProfileView {
    pub code: String,
    pub label: String,
    pub expires_at: String,
    pub phone: String,
    pub batch_size: String,
    pub external_ref: String,
    pub issuer: String,
    pub country: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_policy_profile_view(item: &PolicyProfile) -> PolicyProfileView {
    PolicyProfileView {
        code: text::truncate(&item.code, 120),
        label: text::truncate(&item.label, 40),
        expires_at: item.expires_at.to_string(),
        phone: item.phone.clone(),
        batch_size: item.batch_size.to_string(),
        external_ref: item.external_ref.clone(),
        issuer: item.issuer.clone(),
        country: text::truncate(&item.country, 120),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_profile_views(items: &[PolicyProfile]) -> Vec<PolicyProfileView> {
    let mut views: Vec<PolicyProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_profile_view(item));
    }
    views
}

pub fn sum_policy_profile_view(view: &PolicyProfileView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_batch_size as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_profile("code-749", "label-496");
        let view = to_policy_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
