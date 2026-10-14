use super::*;

/// How screens and exports show a capped data export batch.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportBatchView {
    pub code_text: String,
    pub title: String,
    pub quota_units: String,
    pub weight: String,
    pub issuer: String,
    pub batch_size_text: String,
    pub priority: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_capped_data_export_batch_view(item: &CappedDataExportBatch) -> CappedDataExportBatchView {
    CappedDataExportBatchView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        quota_units: item.quota_units.to_string(),
        weight: format!("{:.2}", item.weight),
        issuer: item.issuer.clone(),
        batch_size_text: item.batch_size.to_string(),
        priority: item.priority.to_string(),
        scope: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_data_export_batch_views(items: &[CappedDataExportBatch]) -> Vec<CappedDataExportBatchView> {
    let mut views: Vec<CappedDataExportBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_data_export_batch_view(item));
    }
    views
}

pub fn sum_capped_data_export_batch_view(view: &CappedDataExportBatchView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_weight + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_data_export_batch("code-518", "title-387");
        let view = to_capped_data_export_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_data_export_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
