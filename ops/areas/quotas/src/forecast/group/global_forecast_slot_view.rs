use super::*;

use acme_kit::text;

/// How screens and exports show a global forecast slot.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastSlotView {
    pub code: String,
    pub label: String,
    pub quota_units: String,
    pub host: String,
    pub updated_at: String,
    pub margin: String,
    pub batch_size: String,
    pub port_text: String,
    pub checksum: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_global_forecast_slot_view(item: &GlobalForecastSlot) -> GlobalForecastSlotView {
    GlobalForecastSlotView {
        code: item.code.clone(),
        label: item.label.clone(),
        quota_units: item.quota_units.to_string(),
        host: text::truncate(&item.host, 64),
        updated_at: item.updated_at.to_string(),
        margin: format!("{:.2}", item.margin),
        batch_size: item.batch_size.to_string(),
        port_text: item.port.to_string(),
        checksum: item.checksum.clone(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_forecast_slot_views(items: &[GlobalForecastSlot]) -> Vec<GlobalForecastSlotView> {
    let mut views: Vec<GlobalForecastSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_forecast_slot_view(item));
    }
    views
}

pub fn sum_global_forecast_slot_view(view: &GlobalForecastSlotView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_updated_at as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_forecast_slot("code-270", "label-473");
        let view = to_global_forecast_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_forecast_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
