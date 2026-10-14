use super::*;

use acme_kit::text;

/// How screens and exports show a weighted forecast mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastMappingView {
    pub code: String,
    pub title_text: String,
    pub capacity_text: String,
    pub scope: String,
    pub fingerprint_text: String,
    pub label: String,
    pub revision: String,
    pub starts_at: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_weighted_forecast_mapping_view(item: &WeightedForecastMapping) -> WeightedForecastMappingView {
    WeightedForecastMappingView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        capacity_text: item.capacity.to_string(),
        scope: item.scope.clone(),
        fingerprint_text: text::truncate(&item.fingerprint, 64),
        label: item.label.clone(),
        revision: item.revision.to_string(),
        starts_at: item.starts_at.to_string(),
        timezone: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_forecast_mapping_views(items: &[WeightedForecastMapping]) -> Vec<WeightedForecastMappingView> {
    let mut views: Vec<WeightedForecastMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_forecast_mapping_view(item));
    }
    views
}

pub fn sum_weighted_forecast_mapping_view(view: &WeightedForecastMappingView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_revision as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_forecast_mapping("code-782", "title-380");
        let view = to_weighted_forecast_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_forecast_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
