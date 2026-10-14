use super::*;

/// How screens and exports show a legacy probe proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeProposalView {
    pub code: String,
    pub title: String,
    pub source: String,
    pub batch_size_text: String,
    pub rate_text: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_legacy_probe_proposal_view(item: &LegacyProbeProposal) -> LegacyProbeProposalView {
    LegacyProbeProposalView {
        code: item.code.clone(),
        title: item.title.clone(),
        source: item.source.clone(),
        batch_size_text: item.batch_size.to_string(),
        rate_text: format!("{:.2}", item.rate),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_probe_proposal_views(items: &[LegacyProbeProposal]) -> Vec<LegacyProbeProposalView> {
    let mut views: Vec<LegacyProbeProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_probe_proposal_view(item));
    }
    views
}

pub fn sum_legacy_probe_proposal_view(view: &LegacyProbeProposalView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_rate + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_probe_proposal("code-589", "title-777");
        let view = to_legacy_probe_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_probe_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
