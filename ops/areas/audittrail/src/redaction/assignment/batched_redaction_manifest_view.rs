use super::*;

use acme_kit::text;

/// How screens and exports show a batched redaction manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionManifestView {
    pub code_text: String,
    pub name: String,
    pub checksum: String,
    pub audience_text: String,
    pub priority: String,
    pub fingerprint: String,
    pub rate_text: String,
    pub score: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_batched_redaction_manifest_view(item: &BatchedRedactionManifest) -> BatchedRedactionManifestView {
    BatchedRedactionManifestView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 64),
        checksum: text::truncate(&item.checksum, 80),
        audience_text: text::truncate(&item.audience, 64),
        priority: item.priority.to_string(),
        fingerprint: item.fingerprint.clone(),
        rate_text: format!("{:.2}", item.rate),
        score: format!("{:.2}", item.score),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_redaction_manifest_views(items: &[BatchedRedactionManifest]) -> Vec<BatchedRedactionManifestView> {
    let mut views: Vec<BatchedRedactionManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_redaction_manifest_view(item));
    }
    views
}

pub fn sum_batched_redaction_manifest_view(view: &BatchedRedactionManifestView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_priority as f64 + parsed_rate + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_redaction_manifest("code-69", "name-165");
        let view = to_batched_redaction_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_redaction_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
