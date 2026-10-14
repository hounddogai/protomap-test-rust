use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal token rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenRollupView {
    pub code: String,
    pub name: String,
    pub rate: String,
    pub floor_value_text: String,
    pub checksum: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_seasonal_token_rollup_view(item: &SeasonalTokenRollup) -> SeasonalTokenRollupView {
    SeasonalTokenRollupView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        rate: format!("{:.2}", item.rate),
        floor_value_text: format!("{:.2}", item.floor_value),
        checksum: item.checksum.clone(),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_token_rollup_views(items: &[SeasonalTokenRollup]) -> Vec<SeasonalTokenRollupView> {
    let mut views: Vec<SeasonalTokenRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_token_rollup_view(item));
    }
    views
}

pub fn sum_seasonal_token_rollup_view(view: &SeasonalTokenRollupView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_rate + parsed_floor_value + (parsed_sequence as f64))
}
