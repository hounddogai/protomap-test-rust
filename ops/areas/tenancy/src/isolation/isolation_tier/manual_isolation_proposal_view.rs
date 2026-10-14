use super::*;

use acme_kit::text;

/// How screens and exports show a manual isolation proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualIsolationProposalView {
    pub code: String,
    pub name_text: String,
    pub expires_at: String,
    pub active: String,
    pub external_ref_text: String,
    pub cost: String,
    pub notes: String,
    pub key_id: String,
    pub budget: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_manual_isolation_proposal_view(item: &ManualIsolationProposal) -> ManualIsolationProposalView {
    ManualIsolationProposalView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 64),
        expires_at: item.expires_at.to_string(),
        active: format!("{}", item.active),
        external_ref_text: item.external_ref.clone(),
        cost: format!("{:.2}", item.cost),
        notes: item.notes.clone(),
        key_id: item.key_id.clone(),
        budget: format!("{:.2}", item.budget),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_isolation_proposal_views(items: &[ManualIsolationProposal]) -> Vec<ManualIsolationProposalView> {
    let mut views: Vec<ManualIsolationProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_isolation_proposal_view(item));
    }
    views
}

pub fn sum_manual_isolation_proposal_view(view: &ManualIsolationProposalView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_cost + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_isolation_proposal("code-367", "name-93");
        let view = to_manual_isolation_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_isolation_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
