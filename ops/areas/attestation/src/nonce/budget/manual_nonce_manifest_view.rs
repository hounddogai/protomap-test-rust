use super::*;

use acme_kit::text;

/// How screens and exports show a manual nonce manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceManifestView {
    pub code: String,
    pub title_text: String,
    pub scope: String,
    pub tenant_id: String,
    pub cost_text: String,
    pub notes_text: String,
    pub segment: String,
    pub phone_text: String,
    pub starts_at_text: String,
    pub status_label: String,
}

pub fn to_manual_nonce_manifest_view(item: &ManualNonceManifest) -> ManualNonceManifestView {
    ManualNonceManifestView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        scope: text::truncate(&item.scope, 40),
        tenant_id: text::truncate(&item.tenant_id, 40),
        cost_text: format!("{:.2}", item.cost),
        notes_text: text::truncate(&item.notes, 40),
        segment: text::truncate(&item.segment, 40),
        phone_text: item.phone.clone(),
        starts_at_text: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_nonce_manifest_views(items: &[ManualNonceManifest]) -> Vec<ManualNonceManifestView> {
    let mut views: Vec<ManualNonceManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_nonce_manifest_view(item));
    }
    views
}

pub fn sum_manual_nonce_manifest_view(view: &ManualNonceManifestView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_cost + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_nonce_manifest("code-9", "title-43");
        let view = to_manual_nonce_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_nonce_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
