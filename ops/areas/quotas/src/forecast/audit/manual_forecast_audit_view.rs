use super::*;

use acme_kit::text;

/// How screens and exports show a manual forecast audit.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastAuditView {
    pub code: String,
    pub name: String,
    pub sequence: String,
    pub key_id: String,
    pub timezone: String,
    pub ttl_seconds: String,
    pub region: String,
    pub window_days_text: String,
    pub weight: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_manual_forecast_audit_view(item: &ManualForecastAudit) -> ManualForecastAuditView {
    ManualForecastAuditView {
        code: item.code.clone(),
        name: item.name.clone(),
        sequence: item.sequence.to_string(),
        key_id: item.key_id.clone(),
        timezone: text::truncate(&item.timezone, 120),
        ttl_seconds: item.ttl_seconds.to_string(),
        region: item.region.clone(),
        window_days_text: item.window_days.to_string(),
        weight: format!("{:.2}", item.weight),
        cipher: text::truncate(&item.cipher, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_forecast_audit_views(items: &[ManualForecastAudit]) -> Vec<ManualForecastAuditView> {
    let mut views: Vec<ManualForecastAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_forecast_audit_view(item));
    }
    views
}

pub fn sum_manual_forecast_audit_view(view: &ManualForecastAuditView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_ttl_seconds as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_forecast_audit("code-867", "name-536");
        let view = to_manual_forecast_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_forecast_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
