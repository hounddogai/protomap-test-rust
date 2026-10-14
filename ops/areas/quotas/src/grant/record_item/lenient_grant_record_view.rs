use super::*;

/// How screens and exports show a lenient grant record.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantRecordView {
    pub code_text: String,
    pub name: String,
    pub host: String,
    pub currency_text: String,
    pub starts_at: String,
    pub quota_units: String,
    pub namespace: String,
    pub updated_at: String,
    pub channel_text: String,
    pub cipher: String,
    pub archived_text: String,
    pub status_label: String,
}

pub fn to_lenient_grant_record_view(item: &LenientGrantRecord) -> LenientGrantRecordView {
    LenientGrantRecordView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        host: item.host.clone(),
        currency_text: item.currency.clone(),
        starts_at: item.starts_at.to_string(),
        quota_units: item.quota_units.to_string(),
        namespace: item.namespace.clone(),
        updated_at: item.updated_at.to_string(),
        channel_text: item.channel.clone(),
        cipher: item.cipher.clone(),
        archived_text: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_grant_record_views(items: &[LenientGrantRecord]) -> Vec<LenientGrantRecordView> {
    let mut views: Vec<LenientGrantRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_grant_record_view(item));
    }
    views
}

pub fn sum_lenient_grant_record_view(view: &LenientGrantRecordView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_quota_units as f64) + (parsed_updated_at as f64))
}
