use super::*;

use acme_kit::text;

/// How screens and exports show a fallback runbook proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRunbookProposalView {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub cipher: String,
    pub description: String,
    pub reviewer: String,
    pub country: String,
    pub batch_size: String,
    pub issuer: String,
    pub locale: String,
    pub rotation_days_text: String,
    pub status_label: String,
}

pub fn to_fallback_runbook_proposal_view(item: &FallbackRunbookProposal) -> FallbackRunbookProposalView {
    FallbackRunbookProposalView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        checksum: item.checksum.clone(),
        cipher: item.cipher.clone(),
        description: item.description.clone(),
        reviewer: item.reviewer.clone(),
        country: item.country.clone(),
        batch_size: item.batch_size.to_string(),
        issuer: item.issuer.clone(),
        locale: item.locale.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_runbook_proposal_views(items: &[FallbackRunbookProposal]) -> Vec<FallbackRunbookProposalView> {
    let mut views: Vec<FallbackRunbookProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_runbook_proposal_view(item));
    }
    views
}

pub fn sum_fallback_runbook_proposal_view(view: &FallbackRunbookProposalView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_runbook_proposal("code-492", "label-522");
        let view = to_fallback_runbook_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_runbook_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
