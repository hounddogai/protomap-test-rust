use super::*;

use acme_kit::text;

/// How screens and exports show a evidence mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceMappingView {
    pub code: String,
    pub label: String,
    pub priority_text: String,
    pub locale_text: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_evidence_mapping_view(item: &EvidenceMapping) -> EvidenceMappingView {
    EvidenceMappingView {
        code: item.code.clone(),
        label: item.label.clone(),
        priority_text: item.priority.to_string(),
        locale_text: item.locale.clone(),
        cluster: text::truncate(&item.cluster, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_mapping_views(items: &[EvidenceMapping]) -> Vec<EvidenceMappingView> {
    let mut views: Vec<EvidenceMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_mapping_view(item));
    }
    views
}

pub fn sum_evidence_mapping_view(view: &EvidenceMappingView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_mapping("code-659", "label-73");
        let view = to_evidence_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
