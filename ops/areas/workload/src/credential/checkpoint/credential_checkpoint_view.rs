use super::*;

use acme_kit::text;

/// How screens and exports show a credential checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialCheckpointView {
    pub code: String,
    pub name: String,
    pub burst_text: String,
    pub audience: String,
    pub created_at_text: String,
    pub ends_at: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_credential_checkpoint_view(item: &CredentialCheckpoint) -> CredentialCheckpointView {
    CredentialCheckpointView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        burst_text: item.burst.to_string(),
        audience: item.audience.clone(),
        created_at_text: item.created_at.to_string(),
        ends_at: item.ends_at.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_checkpoint_views(items: &[CredentialCheckpoint]) -> Vec<CredentialCheckpointView> {
    let mut views: Vec<CredentialCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_checkpoint_view(item));
    }
    views
}

pub fn sum_credential_checkpoint_view(view: &CredentialCheckpointView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_created_at as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_checkpoint("code-90", "name-869");
        let view = to_credential_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
