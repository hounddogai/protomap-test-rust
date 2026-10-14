use super::*;

use acme_kit::text;

/// How screens and exports show a fallback forecast quota.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackForecastQuotaView {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub host_text: String,
    pub source: String,
    pub ttl_seconds_text: String,
    pub audience_text: String,
    pub duration_minutes_text: String,
    pub status_label: String,
}

pub fn to_fallback_forecast_quota_view(item: &FallbackForecastQuota) -> FallbackForecastQuotaView {
    FallbackForecastQuotaView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 64),
        issuer: item.issuer.clone(),
        host_text: text::truncate(&item.host, 120),
        source: item.source.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        audience_text: text::truncate(&item.audience, 40),
        duration_minutes_text: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_forecast_quota_views(items: &[FallbackForecastQuota]) -> Vec<FallbackForecastQuotaView> {
    let mut views: Vec<FallbackForecastQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_forecast_quota_view(item));
    }
    views
}

pub fn sum_fallback_forecast_quota_view(view: &FallbackForecastQuotaView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_forecast_quota("code-621", "title-480");
        let view = to_fallback_forecast_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_forecast_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
