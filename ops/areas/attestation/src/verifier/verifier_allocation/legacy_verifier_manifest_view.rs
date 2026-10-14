use super::*;

use acme_kit::text;

/// How screens and exports show a legacy verifier manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyVerifierManifestView {
    pub code: String,
    pub title: String,
    pub created_at: String,
    pub attempts: String,
    pub sequence_text: String,
    pub host: String,
    pub ends_at_text: String,
    pub external_ref: String,
    pub segment: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_legacy_verifier_manifest_view(item: &LegacyVerifierManifest) -> LegacyVerifierManifestView {
    LegacyVerifierManifestView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        created_at: item.created_at.to_string(),
        attempts: item.attempts.to_string(),
        sequence_text: item.sequence.to_string(),
        host: item.host.clone(),
        ends_at_text: item.ends_at.to_string(),
        external_ref: item.external_ref.clone(),
        segment: item.segment.clone(),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_verifier_manifest_views(items: &[LegacyVerifierManifest]) -> Vec<LegacyVerifierManifestView> {
    let mut views: Vec<LegacyVerifierManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_verifier_manifest_view(item));
    }
    views
}

pub fn sum_legacy_verifier_manifest_view(view: &LegacyVerifierManifestView) -> Result<f64, String> {
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
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_attempts as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_verifier_manifest("code-884", "title-474");
        let view = to_legacy_verifier_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_verifier_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
