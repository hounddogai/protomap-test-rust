use super::*;

/// How screens and exports show a seasonal timeline agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTimelineAgreementView {
    pub code: String,
    pub label: String,
    pub audience_text: String,
    pub cost: String,
    pub archived: String,
    pub updated_at_text: String,
    pub phone: String,
    pub owner_id: String,
    pub quota_units: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_seasonal_timeline_agreement_view(item: &SeasonalTimelineAgreement) -> SeasonalTimelineAgreementView {
    SeasonalTimelineAgreementView {
        code: item.code.clone(),
        label: item.label.clone(),
        audience_text: item.audience.clone(),
        cost: format!("{:.2}", item.cost),
        archived: format!("{}", item.archived),
        updated_at_text: item.updated_at.to_string(),
        phone: item.phone.clone(),
        owner_id: item.owner_id.clone(),
        quota_units: item.quota_units.to_string(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_timeline_agreement_views(items: &[SeasonalTimelineAgreement]) -> Vec<SeasonalTimelineAgreementView> {
    let mut views: Vec<SeasonalTimelineAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_timeline_agreement_view(item));
    }
    views
}

pub fn sum_seasonal_timeline_agreement_view(view: &SeasonalTimelineAgreementView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_cost + (parsed_updated_at as f64) + (parsed_quota_units as f64))
}
