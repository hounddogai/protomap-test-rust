use super::*;

use acme_kit::text;

/// How screens and exports show a credential target.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialTargetView {
    pub code_text: String,
    pub name_text: String,
    pub fingerprint: String,
    pub created_at: String,
    pub cluster: String,
    pub attempts: String,
    pub version_text: String,
    pub scope_text: String,
    pub status_label: String,
}

pub fn to_credential_target_view(item: &CredentialTarget) -> CredentialTargetView {
    CredentialTargetView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        fingerprint: item.fingerprint.clone(),
        created_at: item.created_at.to_string(),
        cluster: text::truncate(&item.cluster, 40),
        attempts: item.attempts.to_string(),
        version_text: item.version.to_string(),
        scope_text: text::truncate(&item.scope, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_target_views(items: &[CredentialTarget]) -> Vec<CredentialTargetView> {
    let mut views: Vec<CredentialTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_target_view(item));
    }
    views
}

pub fn sum_credential_target_view(view: &CredentialTargetView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_attempts as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_target("code-664", "name-278");
        let view = to_credential_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
