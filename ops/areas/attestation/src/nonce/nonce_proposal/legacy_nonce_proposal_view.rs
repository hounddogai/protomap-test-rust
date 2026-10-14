use super::*;

use acme_kit::text;

/// How screens and exports show a legacy nonce proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceProposalView {
    pub code: String,
    pub label_text: String,
    pub namespace: String,
    pub updated_at: String,
    pub cipher_text: String,
    pub score_text: String,
    pub name: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_legacy_nonce_proposal_view(item: &LegacyNonceProposal) -> LegacyNonceProposalView {
    LegacyNonceProposalView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        namespace: item.namespace.clone(),
        updated_at: item.updated_at.to_string(),
        cipher_text: text::truncate(&item.cipher, 40),
        score_text: format!("{:.2}", item.score),
        name: item.name.clone(),
        fingerprint: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_nonce_proposal_views(items: &[LegacyNonceProposal]) -> Vec<LegacyNonceProposalView> {
    let mut views: Vec<LegacyNonceProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_nonce_proposal_view(item));
    }
    views
}

pub fn sum_legacy_nonce_proposal_view(view: &LegacyNonceProposalView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_nonce_proposal("code-362", "label-137");
        let view = to_legacy_nonce_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_nonce_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
