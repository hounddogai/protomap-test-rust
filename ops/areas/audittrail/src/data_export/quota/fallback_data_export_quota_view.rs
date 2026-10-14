use super::*;

use acme_kit::text;

/// How screens and exports show a fallback data export quota.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportQuotaView {
    pub code_text: String,
    pub label_text: String,
    pub rate: String,
    pub cost: String,
    pub quota_units_text: String,
    pub region_text: String,
    pub title_text: String,
    pub status_label: String,
}

pub fn to_fallback_data_export_quota_view(item: &FallbackDataExportQuota) -> FallbackDataExportQuotaView {
    FallbackDataExportQuotaView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        rate: format!("{:.2}", item.rate),
        cost: format!("{:.2}", item.cost),
        quota_units_text: item.quota_units.to_string(),
        region_text: text::truncate(&item.region, 40),
        title_text: text::truncate(&item.title, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_data_export_quota_views(items: &[FallbackDataExportQuota]) -> Vec<FallbackDataExportQuotaView> {
    let mut views: Vec<FallbackDataExportQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_data_export_quota_view(item));
    }
    views
}

pub fn sum_fallback_data_export_quota_view(view: &FallbackDataExportQuotaView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_rate + parsed_cost + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_data_export_quota("code-993", "label-513");
        let view = to_fallback_data_export_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_data_export_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
