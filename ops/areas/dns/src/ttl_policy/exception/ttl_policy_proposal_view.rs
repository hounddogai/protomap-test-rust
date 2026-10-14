use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyProposalView {
    pub code: String,
    pub title: String,
    pub region: String,
    pub cluster_text: String,
    pub visible_text: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_ttl_policy_proposal_view(item: &TtlPolicyProposal) -> TtlPolicyProposalView {
    TtlPolicyProposalView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        region: text::truncate(&item.region, 80),
        cluster_text: item.cluster.clone(),
        visible_text: format!("{}", item.visible),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_proposal_views(items: &[TtlPolicyProposal]) -> Vec<TtlPolicyProposalView> {
    let mut views: Vec<TtlPolicyProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_proposal_view(item));
    }
    views
}

pub fn sum_ttl_policy_proposal_view(view: &TtlPolicyProposalView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_ttl_policy_proposal("code-131", "title-553");
        let view = to_ttl_policy_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
