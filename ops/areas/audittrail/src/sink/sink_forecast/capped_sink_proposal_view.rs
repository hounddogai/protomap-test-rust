use super::*;

use acme_kit::text;

/// How screens and exports show a capped sink proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSinkProposalView {
    pub code: String,
    pub label_text: String,
    pub currency: String,
    pub ends_at: String,
    pub namespace: String,
    pub pinned: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_capped_sink_proposal_view(item: &CappedSinkProposal) -> CappedSinkProposalView {
    CappedSinkProposalView {
        code: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        currency: item.currency.clone(),
        ends_at: item.ends_at.to_string(),
        namespace: item.namespace.clone(),
        pinned: format!("{}", item.pinned),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_sink_proposal_views(items: &[CappedSinkProposal]) -> Vec<CappedSinkProposalView> {
    let mut views: Vec<CappedSinkProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_sink_proposal_view(item));
    }
    views
}

pub fn sum_capped_sink_proposal_view(view: &CappedSinkProposalView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_sink_proposal("code-853", "label-42");
        let view = to_capped_sink_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_sink_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
