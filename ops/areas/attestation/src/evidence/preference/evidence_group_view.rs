use super::*;

use acme_kit::text;

/// How screens and exports show a evidence group.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceGroupView {
    pub code_text: String,
    pub label: String,
    pub batch_size_text: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_evidence_group_view(item: &EvidenceGroup) -> EvidenceGroupView {
    EvidenceGroupView {
        code_text: text::truncate(&item.code, 120),
        label: text::truncate(&item.label, 120),
        batch_size_text: item.batch_size.to_string(),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_group_views(items: &[EvidenceGroup]) -> Vec<EvidenceGroupView> {
    let mut views: Vec<EvidenceGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_group_view(item));
    }
    views
}

pub fn sum_evidence_group_view(view: &EvidenceGroupView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_group("code-818", "label-189");
        let view = to_evidence_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
