use super::*;

use acme_kit::text;

/// How screens and exports show a rolling segment proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSegmentProposalView {
    pub code: String,
    pub name_text: String,
    pub approved_text: String,
    pub rate: String,
    pub source: String,
    pub version: String,
    pub ceiling_text: String,
    pub attempts: String,
    pub external_ref_text: String,
    pub duration_minutes: String,
    pub segment_text: String,
    pub status_label: String,
}

pub fn to_rolling_segment_proposal_view(item: &RollingSegmentProposal) -> RollingSegmentProposalView {
    RollingSegmentProposalView {
        code: text::truncate(&item.code, 120),
        name_text: text::truncate(&item.name, 80),
        approved_text: format!("{}", item.approved),
        rate: format!("{:.2}", item.rate),
        source: item.source.clone(),
        version: item.version.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        attempts: item.attempts.to_string(),
        external_ref_text: item.external_ref.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        segment_text: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_segment_proposal_views(items: &[RollingSegmentProposal]) -> Vec<RollingSegmentProposalView> {
    let mut views: Vec<RollingSegmentProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_segment_proposal_view(item));
    }
    views
}

pub fn sum_rolling_segment_proposal_view(view: &RollingSegmentProposalView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_rate + (parsed_version as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_segment_proposal("code-544", "name-516");
        let view = to_rolling_segment_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_segment_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
