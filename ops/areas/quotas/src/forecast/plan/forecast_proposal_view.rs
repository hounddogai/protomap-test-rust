use super::*;

use acme_kit::text;

/// How screens and exports show a forecast proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastProposalView {
    pub code: String,
    pub name_text: String,
    pub created_at: String,
    pub audience: String,
    pub currency_text: String,
    pub channel: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_forecast_proposal_view(item: &ForecastProposal) -> ForecastProposalView {
    ForecastProposalView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        created_at: item.created_at.to_string(),
        audience: item.audience.clone(),
        currency_text: item.currency.clone(),
        channel: text::truncate(&item.channel, 120),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_proposal_views(items: &[ForecastProposal]) -> Vec<ForecastProposalView> {
    let mut views: Vec<ForecastProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_proposal_view(item));
    }
    views
}

pub fn sum_forecast_proposal_view(view: &ForecastProposalView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_created_at as f64)
}
