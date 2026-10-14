use super::*;

use acme_kit::text;

/// How screens and exports show a data export rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRollupView {
    pub code: String,
    pub title: String,
    pub grace_days: String,
    pub region_text: String,
    pub checksum: String,
    pub issuer: String,
    pub summary: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_data_export_rollup_view(item: &DataExportRollup) -> DataExportRollupView {
    DataExportRollupView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        grace_days: item.grace_days.to_string(),
        region_text: item.region.clone(),
        checksum: item.checksum.clone(),
        issuer: text::truncate(&item.issuer, 64),
        summary: item.summary.clone(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_rollup_views(items: &[DataExportRollup]) -> Vec<DataExportRollupView> {
    let mut views: Vec<DataExportRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_rollup_view(item));
    }
    views
}

pub fn sum_data_export_rollup_view(view: &DataExportRollupView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_rollup("code-225", "title-823");
        let view = to_data_export_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
