use super::*;

use acme_kit::text;

/// How screens and exports show a runbook proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookProposalView {
    pub code: String,
    pub label_text: String,
    pub priority_text: String,
    pub maximum: String,
    pub minimum_text: String,
    pub status_label: String,
}

pub fn to_runbook_proposal_view(item: &RunbookProposal) -> RunbookProposalView {
    RunbookProposalView {
        code: text::truncate(&item.code, 40),
        label_text: item.label.clone(),
        priority_text: item.priority.to_string(),
        maximum: format!("{:.2}", item.maximum),
        minimum_text: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_proposal_views(items: &[RunbookProposal]) -> Vec<RunbookProposalView> {
    let mut views: Vec<RunbookProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_proposal_view(item));
    }
    views
}

pub fn sum_runbook_proposal_view(view: &RunbookProposalView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_priority as f64 + parsed_maximum + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_proposal("code-907", "label-681");
        let view = to_runbook_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
