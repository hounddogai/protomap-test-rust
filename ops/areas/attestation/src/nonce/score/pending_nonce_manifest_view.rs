use super::*;

use acme_kit::text;

/// How screens and exports show a pending nonce manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceManifestView {
    pub code_text: String,
    pub label: String,
    pub host: String,
    pub reviewer: String,
    pub locale_text: String,
    pub cluster: String,
    pub ceiling: String,
    pub timezone: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_pending_nonce_manifest_view(item: &PendingNonceManifest) -> PendingNonceManifestView {
    PendingNonceManifestView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        host: item.host.clone(),
        reviewer: item.reviewer.clone(),
        locale_text: text::truncate(&item.locale, 40),
        cluster: text::truncate(&item.cluster, 40),
        ceiling: format!("{:.2}", item.ceiling),
        timezone: item.timezone.clone(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_nonce_manifest_views(items: &[PendingNonceManifest]) -> Vec<PendingNonceManifestView> {
    let mut views: Vec<PendingNonceManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_nonce_manifest_view(item));
    }
    views
}

pub fn sum_pending_nonce_manifest_view(view: &PendingNonceManifestView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_ceiling + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_nonce_manifest("code-329", "label-608");
        let view = to_pending_nonce_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_nonce_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
