use super::*;

use acme_kit::text;

/// How screens and exports show a timeline proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineProposalView {
    pub code: String,
    pub label: String,
    pub timezone_text: String,
    pub namespace: String,
    pub issuer_text: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_timeline_proposal_view(item: &TimelineProposal) -> TimelineProposalView {
    TimelineProposalView {
        code: item.code.clone(),
        label: item.label.clone(),
        timezone_text: text::truncate(&item.timezone, 120),
        namespace: item.namespace.clone(),
        issuer_text: text::truncate(&item.issuer, 80),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_proposal_views(items: &[TimelineProposal]) -> Vec<TimelineProposalView> {
    let mut views: Vec<TimelineProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_proposal_view(item));
    }
    views
}

pub fn sum_timeline_proposal_view(view: &TimelineProposalView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_timeline_proposal("code-569", "label-609");
        let view = to_timeline_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_timeline_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
