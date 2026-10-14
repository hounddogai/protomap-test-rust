use super::*;

use acme_kit::text;

/// How screens and exports show a health check exception.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckExceptionView {
    pub code: String,
    pub title: String,
    pub quota_units: String,
    pub segment: String,
    pub capacity: String,
    pub label: String,
    pub cipher_text: String,
    pub attempts_text: String,
    pub channel: String,
    pub currency: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_health_check_exception_view(item: &HealthCheckException) -> HealthCheckExceptionView {
    HealthCheckExceptionView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        quota_units: item.quota_units.to_string(),
        segment: item.segment.clone(),
        capacity: item.capacity.to_string(),
        label: text::truncate(&item.label, 80),
        cipher_text: text::truncate(&item.cipher, 80),
        attempts_text: item.attempts.to_string(),
        channel: item.channel.clone(),
        currency: item.currency.clone(),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_exception_views(items: &[HealthCheckException]) -> Vec<HealthCheckExceptionView> {
    let mut views: Vec<HealthCheckExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_exception_view(item));
    }
    views
}

pub fn sum_health_check_exception_view(view: &HealthCheckExceptionView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_capacity as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_exception("code-738", "title-901");
        let view = to_health_check_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
