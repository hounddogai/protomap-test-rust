use super::*;

use acme_kit::text;

/// How screens and exports show a health check template.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTemplateView {
    pub code: String,
    pub label: String,
    pub starts_at: String,
    pub channel_text: String,
    pub score: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_health_check_template_view(item: &HealthCheckTemplate) -> HealthCheckTemplateView {
    HealthCheckTemplateView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        starts_at: item.starts_at.to_string(),
        channel_text: item.channel.clone(),
        score: format!("{:.2}", item.score),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_template_views(items: &[HealthCheckTemplate]) -> Vec<HealthCheckTemplateView> {
    let mut views: Vec<HealthCheckTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_template_view(item));
    }
    views
}

pub fn sum_health_check_template_view(view: &HealthCheckTemplateView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_score + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_template("code-282", "label-227");
        let view = to_health_check_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
