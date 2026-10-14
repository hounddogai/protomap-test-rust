use super::*;

use acme_kit::text;

/// How screens and exports show a data export estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportEstimateView {
    pub code: String,
    pub title_text: String,
    pub segment: String,
    pub budget: String,
    pub notes: String,
    pub contact_email: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_data_export_estimate_view(item: &DataExportEstimate) -> DataExportEstimateView {
    DataExportEstimateView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        segment: item.segment.clone(),
        budget: format!("{:.2}", item.budget),
        notes: text::truncate(&item.notes, 80),
        contact_email: item.contact_email.clone(),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_estimate_views(items: &[DataExportEstimate]) -> Vec<DataExportEstimateView> {
    let mut views: Vec<DataExportEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_estimate_view(item));
    }
    views
}

pub fn sum_data_export_estimate_view(view: &DataExportEstimateView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_budget + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_estimate("code-198", "title-691");
        let view = to_data_export_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
