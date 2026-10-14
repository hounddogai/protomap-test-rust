use super::*;

use acme_kit::text;

/// How screens and exports show a evidence bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBundleView {
    pub code: String,
    pub label_text: String,
    pub timezone_text: String,
    pub unit: String,
    pub title: String,
    pub ttl_seconds_text: String,
    pub namespace: String,
    pub revision: String,
    pub batch_size: String,
    pub contact_email_text: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_evidence_bundle_view(item: &EvidenceBundle) -> EvidenceBundleView {
    EvidenceBundleView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 64),
        timezone_text: text::truncate(&item.timezone, 120),
        unit: item.unit.clone(),
        title: item.title.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        namespace: item.namespace.clone(),
        revision: item.revision.to_string(),
        batch_size: item.batch_size.to_string(),
        contact_email_text: text::truncate(&item.contact_email, 40),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_bundle_views(items: &[EvidenceBundle]) -> Vec<EvidenceBundleView> {
    let mut views: Vec<EvidenceBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_bundle_view(item));
    }
    views
}

pub fn sum_evidence_bundle_view(view: &EvidenceBundleView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_revision as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_bundle("code-399", "label-82");
        let view = to_evidence_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
