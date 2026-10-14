use super::*;

use acme_kit::text;

/// How screens and exports show a regional federation proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationProposalView {
    pub code: String,
    pub title: String,
    pub rate: String,
    pub pinned: String,
    pub window_days: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_regional_federation_proposal_view(item: &RegionalFederationProposal) -> RegionalFederationProposalView {
    RegionalFederationProposalView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        rate: format!("{:.2}", item.rate),
        pinned: format!("{}", item.pinned),
        window_days: item.window_days.to_string(),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_federation_proposal_views(
    items: &[RegionalFederationProposal],
) -> Vec<RegionalFederationProposalView> {
    let mut views: Vec<RegionalFederationProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_federation_proposal_view(item));
    }
    views
}

pub fn sum_regional_federation_proposal_view(view: &RegionalFederationProposalView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_rate + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_federation_proposal("code-37", "title-432");
        let view = to_regional_federation_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_federation_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
