use super::*;

use acme_kit::text;

/// How screens and exports show a tiered forecast mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastMappingView {
    pub code: String,
    pub label: String,
    pub owner_id_text: String,
    pub minimum: String,
    pub pinned: String,
    pub region: String,
    pub rotation_days_text: String,
    pub scope: String,
    pub version: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_tiered_forecast_mapping_view(item: &TieredForecastMapping) -> TieredForecastMappingView {
    TieredForecastMappingView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        owner_id_text: item.owner_id.clone(),
        minimum: format!("{:.2}", item.minimum),
        pinned: format!("{}", item.pinned),
        region: item.region.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        scope: item.scope.clone(),
        version: item.version.to_string(),
        phone: text::truncate(&item.phone, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_forecast_mapping_views(items: &[TieredForecastMapping]) -> Vec<TieredForecastMappingView> {
    let mut views: Vec<TieredForecastMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_forecast_mapping_view(item));
    }
    views
}

pub fn sum_tiered_forecast_mapping_view(view: &TieredForecastMappingView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_minimum + (parsed_rotation_days as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_forecast_mapping("code-793", "label-57");
        let view = to_tiered_forecast_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_forecast_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
