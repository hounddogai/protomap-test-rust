use super::*;

use acme_kit::text;

/// How screens and exports show a tiered health check record.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckRecordView {
    pub code: String,
    pub title_text: String,
    pub timezone: String,
    pub maximum: String,
    pub summary: String,
    pub ttl_seconds_text: String,
    pub phone: String,
    pub budget: String,
    pub country: String,
    pub ends_at_text: String,
    pub status_label: String,
}

pub fn to_tiered_health_check_record_view(item: &TieredHealthCheckRecord) -> TieredHealthCheckRecordView {
    TieredHealthCheckRecordView {
        code: text::truncate(&item.code, 64),
        title_text: text::truncate(&item.title, 120),
        timezone: item.timezone.clone(),
        maximum: format!("{:.2}", item.maximum),
        summary: item.summary.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        phone: text::truncate(&item.phone, 120),
        budget: format!("{:.2}", item.budget),
        country: item.country.clone(),
        ends_at_text: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_health_check_record_views(items: &[TieredHealthCheckRecord]) -> Vec<TieredHealthCheckRecordView> {
    let mut views: Vec<TieredHealthCheckRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_health_check_record_view(item));
    }
    views
}

pub fn sum_tiered_health_check_record_view(view: &TieredHealthCheckRecordView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_maximum + (parsed_ttl_seconds as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_health_check_record("code-400", "title-460");
        let view = to_tiered_health_check_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_health_check_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
