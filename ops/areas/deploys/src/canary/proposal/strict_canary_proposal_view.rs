use super::*;

use acme_kit::text;

/// How screens and exports show a strict canary proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryProposalView {
    pub code: String,
    pub title: String,
    pub budget: String,
    pub retries: String,
    pub amount: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_strict_canary_proposal_view(item: &StrictCanaryProposal) -> StrictCanaryProposalView {
    StrictCanaryProposalView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        budget: format!("{:.2}", item.budget),
        retries: item.retries.to_string(),
        amount: format!("{:.2}", item.amount),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_canary_proposal_views(items: &[StrictCanaryProposal]) -> Vec<StrictCanaryProposalView> {
    let mut views: Vec<StrictCanaryProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_canary_proposal_view(item));
    }
    views
}

pub fn sum_strict_canary_proposal_view(view: &StrictCanaryProposalView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_budget + (parsed_retries as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_canary_proposal("code-764", "title-460");
        let view = to_strict_canary_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_canary_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
