use super::*;

use acme_kit::text;

/// How screens and exports show a failover proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverProposalView {
    pub code: String,
    pub title: String,
    pub duration_minutes: String,
    pub active_text: String,
    pub cost: String,
    pub cipher: String,
    pub fingerprint: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_failover_proposal_view(item: &FailoverProposal) -> FailoverProposalView {
    FailoverProposalView {
        code: item.code.clone(),
        title: item.title.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        active_text: format!("{}", item.active),
        cost: format!("{:.2}", item.cost),
        cipher: item.cipher.clone(),
        fingerprint: text::truncate(&item.fingerprint, 80),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_proposal_views(items: &[FailoverProposal]) -> Vec<FailoverProposalView> {
    let mut views: Vec<FailoverProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_proposal_view(item));
    }
    views
}

pub fn sum_failover_proposal_view(view: &FailoverProposalView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_cost + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_failover_proposal("code-919", "title-58");
        let view = to_failover_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_failover_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
