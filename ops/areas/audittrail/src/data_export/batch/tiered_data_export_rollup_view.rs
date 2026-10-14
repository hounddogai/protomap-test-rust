use super::*;

use acme_kit::text;

/// How screens and exports show a tiered data export rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportRollupView {
    pub code: String,
    pub title: String,
    pub limit: String,
    pub capacity: String,
    pub notes: String,
    pub ratio: String,
    pub duration_minutes: String,
    pub active: String,
    pub summary: String,
    pub margin: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_tiered_data_export_rollup_view(item: &TieredDataExportRollup) -> TieredDataExportRollupView {
    TieredDataExportRollupView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        limit: item.limit.to_string(),
        capacity: item.capacity.to_string(),
        notes: text::truncate(&item.notes, 40),
        ratio: format!("{:.2}", item.ratio),
        duration_minutes: item.duration_minutes.to_string(),
        active: format!("{}", item.active),
        summary: text::truncate(&item.summary, 40),
        margin: format!("{:.2}", item.margin),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_data_export_rollup_views(items: &[TieredDataExportRollup]) -> Vec<TieredDataExportRollupView> {
    let mut views: Vec<TieredDataExportRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_data_export_rollup_view(item));
    }
    views
}

pub fn sum_tiered_data_export_rollup_view(view: &TieredDataExportRollupView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_capacity as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_data_export_rollup("code-959", "title-899");
        let view = to_tiered_data_export_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_data_export_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
