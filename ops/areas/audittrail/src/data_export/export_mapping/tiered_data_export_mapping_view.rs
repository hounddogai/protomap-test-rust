use super::*;

use acme_kit::text;

/// How screens and exports show a tiered data export mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportMappingView {
    pub code: String,
    pub title_text: String,
    pub port: String,
    pub ttl_seconds_text: String,
    pub revision: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_tiered_data_export_mapping_view(item: &TieredDataExportMapping) -> TieredDataExportMappingView {
    TieredDataExportMappingView {
        code: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        port: item.port.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        revision: item.revision.to_string(),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_data_export_mapping_views(items: &[TieredDataExportMapping]) -> Vec<TieredDataExportMappingView> {
    let mut views: Vec<TieredDataExportMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_data_export_mapping_view(item));
    }
    views
}

pub fn sum_tiered_data_export_mapping_view(view: &TieredDataExportMappingView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_port as f64 + (parsed_ttl_seconds as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_data_export_mapping("code-82", "title-670");
        let view = to_tiered_data_export_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_data_export_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
