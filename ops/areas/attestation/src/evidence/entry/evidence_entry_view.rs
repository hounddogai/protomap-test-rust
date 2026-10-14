use super::*;

/// How screens and exports show a evidence entry.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceEntryView {
    pub code: String,
    pub title_text: String,
    pub cipher: String,
    pub amount: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_evidence_entry_view(item: &EvidenceEntry) -> EvidenceEntryView {
    EvidenceEntryView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        cipher: item.cipher.clone(),
        amount: format!("{:.2}", item.amount),
        owner_id: item.owner_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_entry_views(items: &[EvidenceEntry]) -> Vec<EvidenceEntryView> {
    let mut views: Vec<EvidenceEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_entry_view(item));
    }
    views
}

pub fn sum_evidence_entry_view(view: &EvidenceEntryView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}
