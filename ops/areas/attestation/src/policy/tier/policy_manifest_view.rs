use super::*;

use acme_kit::text;

/// How screens and exports show a policy manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyManifestView {
    pub code_text: String,
    pub label_text: String,
    pub attempts: String,
    pub expires_at: String,
    pub external_ref: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_policy_manifest_view(item: &PolicyManifest) -> PolicyManifestView {
    PolicyManifestView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        attempts: item.attempts.to_string(),
        expires_at: item.expires_at.to_string(),
        external_ref: item.external_ref.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_manifest_views(items: &[PolicyManifest]) -> Vec<PolicyManifestView> {
    let mut views: Vec<PolicyManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_manifest_view(item));
    }
    views
}

pub fn sum_policy_manifest_view(view: &PolicyManifestView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_expires_at as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_manifest("code-900", "label-528");
        let view = to_policy_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
