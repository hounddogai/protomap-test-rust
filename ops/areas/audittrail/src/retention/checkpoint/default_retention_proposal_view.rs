use super::*;

use acme_kit::text;

/// How screens and exports show a default retention proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionProposalView {
    pub code: String,
    pub title: String,
    pub contact_email_text: String,
    pub grace_days: String,
    pub score: String,
    pub host_text: String,
    pub floor_value: String,
    pub limit_text: String,
    pub created_at_text: String,
    pub locale_text: String,
    pub status_label: String,
}

pub fn to_default_retention_proposal_view(item: &DefaultRetentionProposal) -> DefaultRetentionProposalView {
    DefaultRetentionProposalView {
        code: item.code.clone(),
        title: item.title.clone(),
        contact_email_text: text::truncate(&item.contact_email, 40),
        grace_days: item.grace_days.to_string(),
        score: format!("{:.2}", item.score),
        host_text: item.host.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        limit_text: item.limit.to_string(),
        created_at_text: item.created_at.to_string(),
        locale_text: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_retention_proposal_views(items: &[DefaultRetentionProposal]) -> Vec<DefaultRetentionProposalView> {
    let mut views: Vec<DefaultRetentionProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_retention_proposal_view(item));
    }
    views
}

pub fn sum_default_retention_proposal_view(view: &DefaultRetentionProposalView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_score + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_retention_proposal("code-160", "title-208");
        let view = to_default_retention_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_retention_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
