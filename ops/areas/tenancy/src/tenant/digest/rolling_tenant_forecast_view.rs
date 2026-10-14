use super::*;

use acme_kit::text;

/// How screens and exports show a rolling tenant forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantForecastView {
    pub code_text: String,
    pub title_text: String,
    pub weight_text: String,
    pub rotation_days: String,
    pub ceiling_text: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_rolling_tenant_forecast_view(item: &RollingTenantForecast) -> RollingTenantForecastView {
    RollingTenantForecastView {
        code_text: item.code.clone(),
        title_text: text::truncate(&item.title, 120),
        weight_text: format!("{:.2}", item.weight),
        rotation_days: item.rotation_days.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_tenant_forecast_views(items: &[RollingTenantForecast]) -> Vec<RollingTenantForecastView> {
    let mut views: Vec<RollingTenantForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_tenant_forecast_view(item));
    }
    views
}

pub fn sum_rolling_tenant_forecast_view(view: &RollingTenantForecastView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_weight + (parsed_rotation_days as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_tenant_forecast("code-566", "title-649");
        let view = to_rolling_tenant_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_tenant_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
