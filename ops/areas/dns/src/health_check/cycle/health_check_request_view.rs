use super::*;

use acme_kit::text;

/// How screens and exports show a health check request.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRequestView {
    pub code: String,
    pub title_text: String,
    pub timezone: String,
    pub key_id_text: String,
    pub currency_text: String,
    pub margin: String,
    pub starts_at: String,
    pub ceiling_text: String,
    pub fingerprint: String,
    pub audience: String,
    pub floor_value_text: String,
    pub status_label: String,
}

pub fn to_health_check_request_view(item: &HealthCheckRequest) -> HealthCheckRequestView {
    HealthCheckRequestView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        timezone: text::truncate(&item.timezone, 40),
        key_id_text: item.key_id.clone(),
        currency_text: text::truncate(&item.currency, 80),
        margin: format!("{:.2}", item.margin),
        starts_at: item.starts_at.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        fingerprint: item.fingerprint.clone(),
        audience: item.audience.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_request_views(items: &[HealthCheckRequest]) -> Vec<HealthCheckRequestView> {
    let mut views: Vec<HealthCheckRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_request_view(item));
    }
    views
}

pub fn sum_health_check_request_view(view: &HealthCheckRequestView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_margin + (parsed_starts_at as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_request("code-704", "title-529");
        let view = to_health_check_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
