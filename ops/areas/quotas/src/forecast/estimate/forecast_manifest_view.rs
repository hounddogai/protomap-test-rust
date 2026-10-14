use super::*;

use acme_kit::text;

/// How screens and exports show a forecast manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastManifestView {
    pub code: String,
    pub name: String,
    pub capacity: String,
    pub locale_text: String,
    pub maximum_text: String,
    pub reviewer_text: String,
    pub status_label: String,
}

pub fn to_forecast_manifest_view(item: &ForecastManifest) -> ForecastManifestView {
    ForecastManifestView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        capacity: item.capacity.to_string(),
        locale_text: item.locale.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        reviewer_text: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_manifest_views(items: &[ForecastManifest]) -> Vec<ForecastManifestView> {
    let mut views: Vec<ForecastManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_manifest_view(item));
    }
    views
}

pub fn sum_forecast_manifest_view(view: &ForecastManifestView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_manifest("code-638", "name-302");
        let view = to_forecast_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
