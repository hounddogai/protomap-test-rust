use super::*;

use acme_kit::text;

/// How screens and exports show a data export cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportCycleView {
    pub code: String,
    pub name: String,
    pub ceiling: String,
    pub tenant_id: String,
    pub scope_text: String,
    pub status_label: String,
}

pub fn to_data_export_cycle_view(item: &DataExportCycle) -> DataExportCycleView {
    DataExportCycleView {
        code: item.code.clone(),
        name: item.name.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        tenant_id: item.tenant_id.clone(),
        scope_text: text::truncate(&item.scope, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_cycle_views(items: &[DataExportCycle]) -> Vec<DataExportCycleView> {
    let mut views: Vec<DataExportCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_cycle_view(item));
    }
    views
}

pub fn sum_data_export_cycle_view(view: &DataExportCycleView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_ceiling)
}
