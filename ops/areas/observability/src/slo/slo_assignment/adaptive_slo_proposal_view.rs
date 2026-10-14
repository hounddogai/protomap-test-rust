use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive slo proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloProposalView {
    pub code: String,
    pub title: String,
    pub timezone_text: String,
    pub reviewer_text: String,
    pub ratio: String,
    pub category: String,
    pub label: String,
    pub port: String,
    pub owner_id: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_adaptive_slo_proposal_view(item: &AdaptiveSloProposal) -> AdaptiveSloProposalView {
    AdaptiveSloProposalView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        timezone_text: text::truncate(&item.timezone, 64),
        reviewer_text: item.reviewer.clone(),
        ratio: format!("{:.2}", item.ratio),
        category: item.category.clone(),
        label: item.label.clone(),
        port: item.port.to_string(),
        owner_id: text::truncate(&item.owner_id, 120),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_slo_proposal_views(items: &[AdaptiveSloProposal]) -> Vec<AdaptiveSloProposalView> {
    let mut views: Vec<AdaptiveSloProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_slo_proposal_view(item));
    }
    views
}

pub fn sum_adaptive_slo_proposal_view(view: &AdaptiveSloProposalView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ratio + (parsed_port as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_slo_proposal("code-191", "title-819");
        let view = to_adaptive_slo_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_slo_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
