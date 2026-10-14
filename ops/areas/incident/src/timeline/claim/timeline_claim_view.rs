use super::*;

use acme_kit::text;

/// How screens and exports show a timeline claim.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineClaimView {
    pub code: String,
    pub name: String,
    pub source: String,
    pub minimum: String,
    pub version: String,
    pub quantity: String,
    pub phone_text: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_timeline_claim_view(item: &TimelineClaim) -> TimelineClaimView {
    TimelineClaimView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        source: item.source.clone(),
        minimum: format!("{:.2}", item.minimum),
        version: item.version.to_string(),
        quantity: item.quantity.to_string(),
        phone_text: text::truncate(&item.phone, 120),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_claim_views(items: &[TimelineClaim]) -> Vec<TimelineClaimView> {
    let mut views: Vec<TimelineClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_claim_view(item));
    }
    views
}

pub fn sum_timeline_claim_view(view: &TimelineClaimView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_minimum + (parsed_version as f64) + (parsed_quantity as f64))
}
