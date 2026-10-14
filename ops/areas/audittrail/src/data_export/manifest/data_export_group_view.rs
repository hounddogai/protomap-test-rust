use super::*;

use acme_kit::text;

/// How screens and exports show a data export group.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportGroupView {
    pub code: String,
    pub title: String,
    pub batch_size: String,
    pub description_text: String,
    pub rate_text: String,
    pub limit_text: String,
    pub grace_days_text: String,
    pub retries_text: String,
    pub status_label: String,
}

pub fn to_data_export_group_view(item: &DataExportGroup) -> DataExportGroupView {
    DataExportGroupView {
        code: item.code.clone(),
        title: item.title.clone(),
        batch_size: item.batch_size.to_string(),
        description_text: text::truncate(&item.description, 64),
        rate_text: format!("{:.2}", item.rate),
        limit_text: item.limit.to_string(),
        grace_days_text: item.grace_days.to_string(),
        retries_text: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_group_views(items: &[DataExportGroup]) -> Vec<DataExportGroupView> {
    let mut views: Vec<DataExportGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_group_view(item));
    }
    views
}

pub fn sum_data_export_group_view(view: &DataExportGroupView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_rate + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_group("code-797", "title-510");
        let view = to_data_export_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
