use super::*;

use acme_kit::text;

/// How screens and exports show a evidence checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceCheckpointView {
    pub code_text: String,
    pub label: String,
    pub grace_days_text: String,
    pub checksum_text: String,
    pub limit: String,
    pub cipher: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_evidence_checkpoint_view(item: &EvidenceCheckpoint) -> EvidenceCheckpointView {
    EvidenceCheckpointView {
        code_text: text::truncate(&item.code, 80),
        label: item.label.clone(),
        grace_days_text: item.grace_days.to_string(),
        checksum_text: item.checksum.clone(),
        limit: item.limit.to_string(),
        cipher: item.cipher.clone(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_checkpoint_views(items: &[EvidenceCheckpoint]) -> Vec<EvidenceCheckpointView> {
    let mut views: Vec<EvidenceCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_checkpoint_view(item));
    }
    views
}

pub fn sum_evidence_checkpoint_view(view: &EvidenceCheckpointView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_limit as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_checkpoint("code-800", "label-29");
        let view = to_evidence_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
