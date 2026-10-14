use super::*;

use acme_kit::text;

/// How screens and exports show a legacy data export batch.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportBatchView {
    pub code_text: String,
    pub name: String,
    pub margin_text: String,
    pub scope: String,
    pub checksum: String,
    pub floor_value: String,
    pub rate_text: String,
    pub expires_at: String,
    pub rotation_days: String,
    pub pinned: String,
    pub channel_text: String,
    pub status_label: String,
}

pub fn to_legacy_data_export_batch_view(item: &LegacyDataExportBatch) -> LegacyDataExportBatchView {
    LegacyDataExportBatchView {
        code_text: text::truncate(&item.code, 40),
        name: item.name.clone(),
        margin_text: format!("{:.2}", item.margin),
        scope: item.scope.clone(),
        checksum: item.checksum.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        rate_text: format!("{:.2}", item.rate),
        expires_at: item.expires_at.to_string(),
        rotation_days: item.rotation_days.to_string(),
        pinned: format!("{}", item.pinned),
        channel_text: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_data_export_batch_views(items: &[LegacyDataExportBatch]) -> Vec<LegacyDataExportBatchView> {
    let mut views: Vec<LegacyDataExportBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_data_export_batch_view(item));
    }
    views
}

pub fn sum_legacy_data_export_batch_view(view: &LegacyDataExportBatchView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_margin + parsed_floor_value + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_data_export_batch("code-372", "name-231");
        let view = to_legacy_data_export_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_data_export_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
