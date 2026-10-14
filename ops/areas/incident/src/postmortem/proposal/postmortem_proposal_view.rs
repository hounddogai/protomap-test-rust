use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemProposalView {
    pub code: String,
    pub label_text: String,
    pub duration_minutes_text: String,
    pub contact_email: String,
    pub cost: String,
    pub pinned_text: String,
    pub country: String,
    pub score: String,
    pub burst: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_postmortem_proposal_view(item: &PostmortemProposal) -> PostmortemProposalView {
    PostmortemProposalView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        duration_minutes_text: item.duration_minutes.to_string(),
        contact_email: text::truncate(&item.contact_email, 64),
        cost: format!("{:.2}", item.cost),
        pinned_text: format!("{}", item.pinned),
        country: text::truncate(&item.country, 64),
        score: format!("{:.2}", item.score),
        burst: item.burst.to_string(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_proposal_views(items: &[PostmortemProposal]) -> Vec<PostmortemProposalView> {
    let mut views: Vec<PostmortemProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_proposal_view(item));
    }
    views
}

pub fn sum_postmortem_proposal_view(view: &PostmortemProposalView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_cost + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_postmortem_proposal("code-943", "label-817");
        let view = to_postmortem_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_postmortem_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
