use super::*;

use acme_kit::text;

/// How screens and exports show a credential quota.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialQuotaView {
    pub code: String,
    pub name_text: String,
    pub key_id: String,
    pub cipher_text: String,
    pub limit_text: String,
    pub external_ref_text: String,
    pub capacity: String,
    pub attempts: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_credential_quota_view(item: &CredentialQuota) -> CredentialQuotaView {
    CredentialQuotaView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        key_id: text::truncate(&item.key_id, 120),
        cipher_text: item.cipher.clone(),
        limit_text: item.limit.to_string(),
        external_ref_text: item.external_ref.clone(),
        capacity: item.capacity.to_string(),
        attempts: item.attempts.to_string(),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_quota_views(items: &[CredentialQuota]) -> Vec<CredentialQuotaView> {
    let mut views: Vec<CredentialQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_quota_view(item));
    }
    views
}

pub fn sum_credential_quota_view(view: &CredentialQuotaView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_capacity as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_quota("code-625", "name-332");
        let view = to_credential_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
