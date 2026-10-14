use super::*;

/// How screens and exports show a rolling redaction score.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionScoreView {
    pub code: String,
    pub name: String,
    pub duration_minutes_text: String,
    pub reviewer_text: String,
    pub host: String,
    pub batch_size_text: String,
    pub starts_at: String,
    pub namespace: String,
    pub tenant_id: String,
    pub margin_text: String,
    pub pinned_text: String,
    pub status_label: String,
}

pub fn to_rolling_redaction_score_view(item: &RollingRedactionScore) -> RollingRedactionScoreView {
    RollingRedactionScoreView {
        code: item.code.clone(),
        name: item.name.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        reviewer_text: item.reviewer.clone(),
        host: item.host.clone(),
        batch_size_text: item.batch_size.to_string(),
        starts_at: item.starts_at.to_string(),
        namespace: item.namespace.clone(),
        tenant_id: item.tenant_id.clone(),
        margin_text: format!("{:.2}", item.margin),
        pinned_text: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_redaction_score_views(items: &[RollingRedactionScore]) -> Vec<RollingRedactionScoreView> {
    let mut views: Vec<RollingRedactionScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_redaction_score_view(item));
    }
    views
}

pub fn sum_rolling_redaction_score_view(view: &RollingRedactionScoreView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_batch_size as f64) + (parsed_starts_at as f64))
}
