use super::*;

use acme_kit::text;

/// How screens and exports show a evidence record.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRecordView {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub tenant_id: String,
    pub grace_days: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_evidence_record_view(item: &EvidenceRecord) -> EvidenceRecordView {
    EvidenceRecordView {
        code: item.code.clone(),
        label: item.label.clone(),
        currency: text::truncate(&item.currency, 40),
        tenant_id: item.tenant_id.clone(),
        grace_days: item.grace_days.to_string(),
        owner_id: text::truncate(&item.owner_id, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_record_views(items: &[EvidenceRecord]) -> Vec<EvidenceRecordView> {
    let mut views: Vec<EvidenceRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_record_view(item));
    }
    views
}

pub fn sum_evidence_record_view(view: &EvidenceRecordView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_record("code-240", "label-702");
        let view = to_evidence_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
