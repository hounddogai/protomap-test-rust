use super::*;

/// How screens and exports show a seasonal health check baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckBaselineView {
    pub code_text: String,
    pub title: String,
    pub segment_text: String,
    pub sequence: String,
    pub retries: String,
    pub locale_text: String,
    pub cost: String,
    pub burst: String,
    pub checksum_text: String,
    pub label_text: String,
    pub owner_id_text: String,
    pub status_label: String,
}

pub fn to_seasonal_health_check_baseline_view(item: &SeasonalHealthCheckBaseline) -> SeasonalHealthCheckBaselineView {
    SeasonalHealthCheckBaselineView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        segment_text: item.segment.clone(),
        sequence: item.sequence.to_string(),
        retries: item.retries.to_string(),
        locale_text: item.locale.clone(),
        cost: format!("{:.2}", item.cost),
        burst: item.burst.to_string(),
        checksum_text: item.checksum.clone(),
        label_text: item.label.clone(),
        owner_id_text: item.owner_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_health_check_baseline_views(
    items: &[SeasonalHealthCheckBaseline],
) -> Vec<SeasonalHealthCheckBaselineView> {
    let mut views: Vec<SeasonalHealthCheckBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_health_check_baseline_view(item));
    }
    views
}

pub fn sum_seasonal_health_check_baseline_view(view: &SeasonalHealthCheckBaselineView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_retries as f64) + parsed_cost)
}
