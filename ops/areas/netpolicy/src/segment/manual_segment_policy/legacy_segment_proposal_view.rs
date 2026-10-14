use super::*;

use acme_kit::text;

/// How screens and exports show a legacy segment proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacySegmentProposalView {
    pub code: String,
    pub label: String,
    pub priority: String,
    pub checksum: String,
    pub summary: String,
    pub cluster: String,
    pub starts_at: String,
    pub segment: String,
    pub audience: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_legacy_segment_proposal_view(item: &LegacySegmentProposal) -> LegacySegmentProposalView {
    LegacySegmentProposalView {
        code: item.code.clone(),
        label: item.label.clone(),
        priority: item.priority.to_string(),
        checksum: text::truncate(&item.checksum, 120),
        summary: item.summary.clone(),
        cluster: item.cluster.clone(),
        starts_at: item.starts_at.to_string(),
        segment: item.segment.clone(),
        audience: text::truncate(&item.audience, 64),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_segment_proposal_views(items: &[LegacySegmentProposal]) -> Vec<LegacySegmentProposalView> {
    let mut views: Vec<LegacySegmentProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_segment_proposal_view(item));
    }
    views
}

pub fn sum_legacy_segment_proposal_view(view: &LegacySegmentProposalView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_segment_proposal("code-249", "label-491");
        let view = to_legacy_segment_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_segment_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
