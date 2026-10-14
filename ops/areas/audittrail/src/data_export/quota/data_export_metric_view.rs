use super::*;

/// How screens and exports show a data export metric.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportMetricView {
    pub code: String,
    pub label: String,
    pub margin: String,
    pub cost_text: String,
    pub source: String,
    pub weight: String,
    pub contact_email_text: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_data_export_metric_view(item: &DataExportMetric) -> DataExportMetricView {
    DataExportMetricView {
        code: item.code.clone(),
        label: item.label.clone(),
        margin: format!("{:.2}", item.margin),
        cost_text: format!("{:.2}", item.cost),
        source: item.source.clone(),
        weight: format!("{:.2}", item.weight),
        contact_email_text: item.contact_email.clone(),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_metric_views(items: &[DataExportMetric]) -> Vec<DataExportMetricView> {
    let mut views: Vec<DataExportMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_metric_view(item));
    }
    views
}

pub fn sum_data_export_metric_view(view: &DataExportMetricView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_margin + parsed_cost + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_metric("code-245", "label-622");
        let view = to_data_export_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
