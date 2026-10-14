use super::*;

use acme_kit::text;

/// How screens and exports show a weighted rule proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleProposalView {
    pub code: String,
    pub title: String,
    pub duration_minutes: String,
    pub threshold_text: String,
    pub version: String,
    pub unit: String,
    pub category: String,
    pub phone_text: String,
    pub status_label: String,
}

pub fn to_weighted_rule_proposal_view(item: &WeightedRuleProposal) -> WeightedRuleProposalView {
    WeightedRuleProposalView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 64),
        duration_minutes: item.duration_minutes.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        version: item.version.to_string(),
        unit: text::truncate(&item.unit, 40),
        category: text::truncate(&item.category, 40),
        phone_text: text::truncate(&item.phone, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_rule_proposal_views(items: &[WeightedRuleProposal]) -> Vec<WeightedRuleProposalView> {
    let mut views: Vec<WeightedRuleProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_rule_proposal_view(item));
    }
    views
}

pub fn sum_weighted_rule_proposal_view(view: &WeightedRuleProposalView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_threshold + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_rule_proposal("code-707", "title-510");
        let view = to_weighted_rule_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_rule_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
