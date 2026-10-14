use super::*;

/// How screens and exports show a data export preference.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportPreferenceView {
    pub code_text: String,
    pub name: String,
    pub scope_text: String,
    pub retries: String,
    pub category_text: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_data_export_preference_view(item: &DataExportPreference) -> DataExportPreferenceView {
    DataExportPreferenceView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        scope_text: item.scope.clone(),
        retries: item.retries.to_string(),
        category_text: item.category.clone(),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_preference_views(items: &[DataExportPreference]) -> Vec<DataExportPreferenceView> {
    let mut views: Vec<DataExportPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_preference_view(item));
    }
    views
}

pub fn sum_data_export_preference_view(view: &DataExportPreferenceView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_preference("code-410", "name-9");
        let view = to_data_export_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
