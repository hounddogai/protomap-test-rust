use super::*;

use acme_kit::text;

/// How screens and exports show a peering proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringProposalView {
    pub code: String,
    pub title: String,
    pub port: String,
    pub maximum: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_peering_proposal_view(item: &PeeringProposal) -> PeeringProposalView {
    PeeringProposalView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        port: item.port.to_string(),
        maximum: format!("{:.2}", item.maximum),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_proposal_views(items: &[PeeringProposal]) -> Vec<PeeringProposalView> {
    let mut views: Vec<PeeringProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_proposal_view(item));
    }
    views
}

pub fn sum_peering_proposal_view(view: &PeeringProposalView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_port as f64 + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_proposal("code-80", "title-492");
        let view = to_peering_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
