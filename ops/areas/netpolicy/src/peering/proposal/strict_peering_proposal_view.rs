use super::*;

use acme_kit::text;

/// How screens and exports show a strict peering proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPeeringProposalView {
    pub code: String,
    pub name_text: String,
    pub created_at: String,
    pub summary: String,
    pub fingerprint: String,
    pub maximum: String,
    pub principal: String,
    pub updated_at: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_strict_peering_proposal_view(item: &StrictPeeringProposal) -> StrictPeeringProposalView {
    StrictPeeringProposalView {
        code: text::truncate(&item.code, 64),
        name_text: item.name.clone(),
        created_at: item.created_at.to_string(),
        summary: item.summary.clone(),
        fingerprint: item.fingerprint.clone(),
        maximum: format!("{:.2}", item.maximum),
        principal: text::truncate(&item.principal, 40),
        updated_at: item.updated_at.to_string(),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_peering_proposal_views(items: &[StrictPeeringProposal]) -> Vec<StrictPeeringProposalView> {
    let mut views: Vec<StrictPeeringProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_peering_proposal_view(item));
    }
    views
}

pub fn sum_strict_peering_proposal_view(view: &StrictPeeringProposalView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_maximum + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_peering_proposal("code-638", "name-293");
        let view = to_strict_peering_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_peering_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
