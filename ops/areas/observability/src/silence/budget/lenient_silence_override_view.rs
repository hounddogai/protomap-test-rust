use super::*;

/// How screens and exports show a lenient silence override.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSilenceOverrideView {
    pub code: String,
    pub label: String,
    pub priority: String,
    pub limit: String,
    pub retries: String,
    pub approved: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_lenient_silence_override_view(item: &LenientSilenceOverride) -> LenientSilenceOverrideView {
    LenientSilenceOverrideView {
        code: item.code.clone(),
        label: item.label.clone(),
        priority: item.priority.to_string(),
        limit: item.limit.to_string(),
        retries: item.retries.to_string(),
        approved: format!("{}", item.approved),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_silence_override_views(items: &[LenientSilenceOverride]) -> Vec<LenientSilenceOverrideView> {
    let mut views: Vec<LenientSilenceOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_silence_override_view(item));
    }
    views
}

pub fn sum_lenient_silence_override_view(view: &LenientSilenceOverrideView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_limit as f64) + (parsed_retries as f64))
}
