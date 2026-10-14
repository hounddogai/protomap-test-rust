use super::*;

use acme_kit::text;

/// How screens and exports show a identity manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityManifestView {
    pub code_text: String,
    pub title: String,
    pub tenant_id_text: String,
    pub checksum: String,
    pub revision_text: String,
    pub cluster: String,
    pub attempts: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_identity_manifest_view(item: &IdentityManifest) -> IdentityManifestView {
    IdentityManifestView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        tenant_id_text: item.tenant_id.clone(),
        checksum: item.checksum.clone(),
        revision_text: item.revision.to_string(),
        cluster: text::truncate(&item.cluster, 40),
        attempts: item.attempts.to_string(),
        host: text::truncate(&item.host, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_manifest_views(items: &[IdentityManifest]) -> Vec<IdentityManifestView> {
    let mut views: Vec<IdentityManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_manifest_view(item));
    }
    views
}

pub fn sum_identity_manifest_view(view: &IdentityManifestView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_manifest("code-863", "title-323");
        let view = to_identity_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
