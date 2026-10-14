use super::*;

use acme_kit::text;

/// How screens and exports show a strict allowlist proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistProposalView {
    pub code: String,
    pub label: String,
    pub maximum_text: String,
    pub checksum: String,
    pub external_ref: String,
    pub notes: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_strict_allowlist_proposal_view(item: &StrictAllowlistProposal) -> StrictAllowlistProposalView {
    StrictAllowlistProposalView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        checksum: text::truncate(&item.checksum, 64),
        external_ref: text::truncate(&item.external_ref, 80),
        notes: item.notes.clone(),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_allowlist_proposal_views(items: &[StrictAllowlistProposal]) -> Vec<StrictAllowlistProposalView> {
    let mut views: Vec<StrictAllowlistProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_allowlist_proposal_view(item));
    }
    views
}

pub fn sum_strict_allowlist_proposal_view(view: &StrictAllowlistProposalView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_allowlist_proposal("code-837", "label-74");
        let view = to_strict_allowlist_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_allowlist_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
