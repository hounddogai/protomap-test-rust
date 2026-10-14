use super::*;

use acme_kit::text;

/// How screens and exports show a verifier manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierManifestView {
    pub code: String,
    pub label: String,
    pub batch_size: String,
    pub cost: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_verifier_manifest_view(item: &VerifierManifest) -> VerifierManifestView {
    VerifierManifestView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        batch_size: item.batch_size.to_string(),
        cost: format!("{:.2}", item.cost),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_manifest_views(items: &[VerifierManifest]) -> Vec<VerifierManifestView> {
    let mut views: Vec<VerifierManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_manifest_view(item));
    }
    views
}

pub fn sum_verifier_manifest_view(view: &VerifierManifestView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_cost)
}
