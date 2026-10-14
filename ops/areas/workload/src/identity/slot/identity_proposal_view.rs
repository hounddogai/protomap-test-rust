use super::*;

use acme_kit::text;

/// How screens and exports show a identity proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityProposalView {
    pub code: String,
    pub name: String,
    pub weight: String,
    pub score: String,
    pub source: String,
    pub cluster: String,
    pub phone: String,
    pub scope: String,
    pub ceiling: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_identity_proposal_view(item: &IdentityProposal) -> IdentityProposalView {
    IdentityProposalView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        weight: format!("{:.2}", item.weight),
        score: format!("{:.2}", item.score),
        source: item.source.clone(),
        cluster: text::truncate(&item.cluster, 64),
        phone: item.phone.clone(),
        scope: text::truncate(&item.scope, 40),
        ceiling: format!("{:.2}", item.ceiling),
        tenant_id_text: text::truncate(&item.tenant_id, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_proposal_views(items: &[IdentityProposal]) -> Vec<IdentityProposalView> {
    let mut views: Vec<IdentityProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_proposal_view(item));
    }
    views
}

pub fn sum_identity_proposal_view(view: &IdentityProposalView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_weight + parsed_score + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_proposal("code-590", "name-983");
        let view = to_identity_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
