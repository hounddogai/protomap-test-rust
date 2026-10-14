use super::*;

use acme_kit::text;

/// How screens and exports show a batched postmortem profile.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemProfileView {
    pub code: String,
    pub title: String,
    pub host: String,
    pub discount: String,
    pub notes: String,
    pub audience: String,
    pub channel_text: String,
    pub status_label: String,
}

pub fn to_batched_postmortem_profile_view(item: &BatchedPostmortemProfile) -> BatchedPostmortemProfileView {
    BatchedPostmortemProfileView {
        code: item.code.clone(),
        title: item.title.clone(),
        host: item.host.clone(),
        discount: format!("{:.2}", item.discount),
        notes: text::truncate(&item.notes, 80),
        audience: item.audience.clone(),
        channel_text: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_postmortem_profile_views(items: &[BatchedPostmortemProfile]) -> Vec<BatchedPostmortemProfileView> {
    let mut views: Vec<BatchedPostmortemProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_postmortem_profile_view(item));
    }
    views
}

pub fn sum_batched_postmortem_profile_view(view: &BatchedPostmortemProfileView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_discount)
}
