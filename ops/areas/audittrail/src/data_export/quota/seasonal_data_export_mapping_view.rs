use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal data export mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportMappingView {
    pub code: String,
    pub name: String,
    pub minimum_text: String,
    pub cost: String,
    pub label: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_seasonal_data_export_mapping_view(item: &SeasonalDataExportMapping) -> SeasonalDataExportMappingView {
    SeasonalDataExportMappingView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        minimum_text: format!("{:.2}", item.minimum),
        cost: format!("{:.2}", item.cost),
        label: item.label.clone(),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_data_export_mapping_views(
    items: &[SeasonalDataExportMapping],
) -> Vec<SeasonalDataExportMappingView> {
    let mut views: Vec<SeasonalDataExportMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_data_export_mapping_view(item));
    }
    views
}

pub fn sum_seasonal_data_export_mapping_view(view: &SeasonalDataExportMappingView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_minimum + parsed_cost + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_data_export_mapping("code-186", "name-76");
        let view = to_seasonal_data_export_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_data_export_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
