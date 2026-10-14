use super::*;

/// How screens and exports show a redaction profile.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionProfileView {
    pub code: String,
    pub label: String,
    pub phone_text: String,
    pub rate: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_redaction_profile_view(item: &RedactionProfile) -> RedactionProfileView {
    RedactionProfileView {
        code: item.code.clone(),
        label: item.label.clone(),
        phone_text: item.phone.clone(),
        rate: format!("{:.2}", item.rate),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_profile_views(items: &[RedactionProfile]) -> Vec<RedactionProfileView> {
    let mut views: Vec<RedactionProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_profile_view(item));
    }
    views
}

pub fn sum_redaction_profile_view(view: &RedactionProfileView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_rate + (parsed_duration_minutes as f64))
}
