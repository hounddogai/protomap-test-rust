use super::*;

use acme_kit::text;

/// How screens and exports show a health check preference.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckPreferenceView {
    pub code: String,
    pub label: String,
    pub host: String,
    pub archived: String,
    pub margin: String,
    pub grace_days: String,
    pub locale: String,
    pub batch_size: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_health_check_preference_view(item: &HealthCheckPreference) -> HealthCheckPreferenceView {
    HealthCheckPreferenceView {
        code: item.code.clone(),
        label: item.label.clone(),
        host: item.host.clone(),
        archived: format!("{}", item.archived),
        margin: format!("{:.2}", item.margin),
        grace_days: item.grace_days.to_string(),
        locale: text::truncate(&item.locale, 40),
        batch_size: item.batch_size.to_string(),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_preference_views(items: &[HealthCheckPreference]) -> Vec<HealthCheckPreferenceView> {
    let mut views: Vec<HealthCheckPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_preference_view(item));
    }
    views
}

pub fn sum_health_check_preference_view(view: &HealthCheckPreferenceView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_margin + (parsed_grace_days as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_preference("code-950", "label-455");
        let view = to_health_check_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
