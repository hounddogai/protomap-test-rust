use super::*;

/// How screens and exports show a legacy failover proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFailoverProposalView {
    pub code: String,
    pub title: String,
    pub external_ref_text: String,
    pub created_at: String,
    pub capacity: String,
    pub scope: String,
    pub retries: String,
    pub port_text: String,
    pub minimum: String,
    pub threshold: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_legacy_failover_proposal_view(item: &LegacyFailoverProposal) -> LegacyFailoverProposalView {
    LegacyFailoverProposalView {
        code: item.code.clone(),
        title: item.title.clone(),
        external_ref_text: item.external_ref.clone(),
        created_at: item.created_at.to_string(),
        capacity: item.capacity.to_string(),
        scope: item.scope.clone(),
        retries: item.retries.to_string(),
        port_text: item.port.to_string(),
        minimum: format!("{:.2}", item.minimum),
        threshold: format!("{:.2}", item.threshold),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_failover_proposal_views(items: &[LegacyFailoverProposal]) -> Vec<LegacyFailoverProposalView> {
    let mut views: Vec<LegacyFailoverProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_failover_proposal_view(item));
    }
    views
}

pub fn sum_legacy_failover_proposal_view(view: &LegacyFailoverProposalView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_capacity as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_failover_proposal("code-297", "title-589");
        let view = to_legacy_failover_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_failover_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
