use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal region pin proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRegionPinProposalView {
    pub code: String,
    pub label_text: String,
    pub timezone_text: String,
    pub rate: String,
    pub external_ref: String,
    pub source: String,
    pub region: String,
    pub fingerprint: String,
    pub sequence: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_seasonal_region_pin_proposal_view(item: &SeasonalRegionPinProposal) -> SeasonalRegionPinProposalView {
    SeasonalRegionPinProposalView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        timezone_text: text::truncate(&item.timezone, 120),
        rate: format!("{:.2}", item.rate),
        external_ref: item.external_ref.clone(),
        source: item.source.clone(),
        region: item.region.clone(),
        fingerprint: text::truncate(&item.fingerprint, 80),
        sequence: item.sequence.to_string(),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_region_pin_proposal_views(
    items: &[SeasonalRegionPinProposal],
) -> Vec<SeasonalRegionPinProposalView> {
    let mut views: Vec<SeasonalRegionPinProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_region_pin_proposal_view(item));
    }
    views
}

pub fn sum_seasonal_region_pin_proposal_view(view: &SeasonalRegionPinProposalView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_rate + (parsed_sequence as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_region_pin_proposal("code-161", "label-702");
        let view = to_seasonal_region_pin_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_region_pin_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
