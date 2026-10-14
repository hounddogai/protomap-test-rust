use super::*;

use acme_kit::text;

/// How screens and exports show a evidence draft.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceDraftView {
    pub code: String,
    pub label: String,
    pub category: String,
    pub floor_value: String,
    pub attempts: String,
    pub margin: String,
    pub checksum: String,
    pub revision_text: String,
    pub limit: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_evidence_draft_view(item: &EvidenceDraft) -> EvidenceDraftView {
    EvidenceDraftView {
        code: item.code.clone(),
        label: item.label.clone(),
        category: item.category.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        attempts: item.attempts.to_string(),
        margin: format!("{:.2}", item.margin),
        checksum: text::truncate(&item.checksum, 80),
        revision_text: item.revision.to_string(),
        limit: item.limit.to_string(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_draft_views(items: &[EvidenceDraft]) -> Vec<EvidenceDraftView> {
    let mut views: Vec<EvidenceDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_draft_view(item));
    }
    views
}

pub fn sum_evidence_draft_view(view: &EvidenceDraftView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_floor_value + (parsed_attempts as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_draft("code-288", "label-865");
        let view = to_evidence_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
