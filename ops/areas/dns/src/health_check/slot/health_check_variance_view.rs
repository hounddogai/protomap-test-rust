use super::*;

use acme_kit::text;

/// How screens and exports show a health check variance.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckVarianceView {
    pub code_text: String,
    pub title: String,
    pub created_at: String,
    pub approved: String,
    pub cost_text: String,
    pub segment: String,
    pub floor_value_text: String,
    pub audience: String,
    pub discount: String,
    pub unit: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_health_check_variance_view(item: &HealthCheckVariance) -> HealthCheckVarianceView {
    HealthCheckVarianceView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        created_at: item.created_at.to_string(),
        approved: format!("{}", item.approved),
        cost_text: format!("{:.2}", item.cost),
        segment: item.segment.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        audience: text::truncate(&item.audience, 40),
        discount: format!("{:.2}", item.discount),
        unit: text::truncate(&item.unit, 64),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_variance_views(items: &[HealthCheckVariance]) -> Vec<HealthCheckVarianceView> {
    let mut views: Vec<HealthCheckVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_variance_view(item));
    }
    views
}

pub fn sum_health_check_variance_view(view: &HealthCheckVarianceView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_cost + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_variance("code-177", "title-165");
        let view = to_health_check_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
