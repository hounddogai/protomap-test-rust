use super::*;

use acme_kit::text;

/// How screens and exports show a data export profile.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportProfileView {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub retries_text: String,
    pub currency: String,
    pub cost_text: String,
    pub status_label: String,
}

pub fn to_data_export_profile_view(item: &DataExportProfile) -> DataExportProfileView {
    DataExportProfileView {
        code: item.code.clone(),
        name: item.name.clone(),
        namespace: text::truncate(&item.namespace, 40),
        retries_text: item.retries.to_string(),
        currency: item.currency.clone(),
        cost_text: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_profile_views(items: &[DataExportProfile]) -> Vec<DataExportProfileView> {
    let mut views: Vec<DataExportProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_profile_view(item));
    }
    views
}

pub fn sum_data_export_profile_view(view: &DataExportProfileView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_retries as f64 + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_profile("code-425", "name-783");
        let view = to_data_export_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
