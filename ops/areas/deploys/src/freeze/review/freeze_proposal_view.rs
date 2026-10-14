use super::*;

use acme_kit::text;

/// How screens and exports show a freeze proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeProposalView {
    pub code_text: String,
    pub label: String,
    pub summary: String,
    pub cost: String,
    pub duration_minutes: String,
    pub source_text: String,
    pub ratio_text: String,
    pub retries: String,
    pub category: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_freeze_proposal_view(item: &FreezeProposal) -> FreezeProposalView {
    FreezeProposalView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 120),
        summary: item.summary.clone(),
        cost: format!("{:.2}", item.cost),
        duration_minutes: item.duration_minutes.to_string(),
        source_text: item.source.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        retries: item.retries.to_string(),
        category: text::truncate(&item.category, 64),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_proposal_views(items: &[FreezeProposal]) -> Vec<FreezeProposalView> {
    let mut views: Vec<FreezeProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_proposal_view(item));
    }
    views
}

pub fn sum_freeze_proposal_view(view: &FreezeProposalView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_cost + (parsed_duration_minutes as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_proposal("code-81", "label-297");
        let view = to_freeze_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
