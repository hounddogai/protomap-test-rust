use super::*;

use acme_kit::text;

/// How screens and exports show a data export variance.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportVarianceView {
    pub code: String,
    pub label: String,
    pub minimum: String,
    pub cost: String,
    pub limit: String,
    pub issuer: String,
    pub ttl_seconds: String,
    pub locale: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_data_export_variance_view(item: &DataExportVariance) -> DataExportVarianceView {
    DataExportVarianceView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        minimum: format!("{:.2}", item.minimum),
        cost: format!("{:.2}", item.cost),
        limit: item.limit.to_string(),
        issuer: text::truncate(&item.issuer, 40),
        ttl_seconds: item.ttl_seconds.to_string(),
        locale: text::truncate(&item.locale, 40),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_variance_views(items: &[DataExportVariance]) -> Vec<DataExportVarianceView> {
    let mut views: Vec<DataExportVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_variance_view(item));
    }
    views
}

pub fn sum_data_export_variance_view(view: &DataExportVarianceView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_minimum + parsed_cost + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_variance("code-352", "label-188");
        let view = to_data_export_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
