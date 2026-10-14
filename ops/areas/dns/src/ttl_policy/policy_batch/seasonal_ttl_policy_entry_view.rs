use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal ttl policy entry.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyEntryView {
    pub code: String,
    pub title_text: String,
    pub channel: String,
    pub retries: String,
    pub batch_size: String,
    pub score_text: String,
    pub cost: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_seasonal_ttl_policy_entry_view(item: &SeasonalTtlPolicyEntry) -> SeasonalTtlPolicyEntryView {
    SeasonalTtlPolicyEntryView {
        code: text::truncate(&item.code, 120),
        title_text: text::truncate(&item.title, 64),
        channel: text::truncate(&item.channel, 80),
        retries: item.retries.to_string(),
        batch_size: item.batch_size.to_string(),
        score_text: format!("{:.2}", item.score),
        cost: format!("{:.2}", item.cost),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_ttl_policy_entry_views(items: &[SeasonalTtlPolicyEntry]) -> Vec<SeasonalTtlPolicyEntryView> {
    let mut views: Vec<SeasonalTtlPolicyEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_ttl_policy_entry_view(item));
    }
    views
}

pub fn sum_seasonal_ttl_policy_entry_view(view: &SeasonalTtlPolicyEntryView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_batch_size as f64) + parsed_score)
}
