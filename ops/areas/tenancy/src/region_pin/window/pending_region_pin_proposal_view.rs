use super::*;

use acme_kit::text;

/// How screens and exports show a pending region pin proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinProposalView {
    pub code: String,
    pub label_text: String,
    pub channel: String,
    pub score_text: String,
    pub duration_minutes_text: String,
    pub segment: String,
    pub version: String,
    pub margin: String,
    pub weight: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_pending_region_pin_proposal_view(item: &PendingRegionPinProposal) -> PendingRegionPinProposalView {
    PendingRegionPinProposalView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 64),
        channel: item.channel.clone(),
        score_text: format!("{:.2}", item.score),
        duration_minutes_text: item.duration_minutes.to_string(),
        segment: item.segment.clone(),
        version: item.version.to_string(),
        margin: format!("{:.2}", item.margin),
        weight: format!("{:.2}", item.weight),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_region_pin_proposal_views(items: &[PendingRegionPinProposal]) -> Vec<PendingRegionPinProposalView> {
    let mut views: Vec<PendingRegionPinProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_region_pin_proposal_view(item));
    }
    views
}

pub fn sum_pending_region_pin_proposal_view(view: &PendingRegionPinProposalView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_score + (parsed_duration_minutes as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_region_pin_proposal("code-961", "label-799");
        let view = to_pending_region_pin_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_region_pin_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
