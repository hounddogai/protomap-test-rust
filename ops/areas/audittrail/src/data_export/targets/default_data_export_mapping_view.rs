use super::*;

use acme_kit::text;

/// How screens and exports show a default data export mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportMappingView {
    pub code: String,
    pub label: String,
    pub expires_at_text: String,
    pub name_text: String,
    pub namespace_text: String,
    pub category: String,
    pub ceiling: String,
    pub contact_email_text: String,
    pub scope: String,
    pub updated_at: String,
    pub rate_text: String,
    pub status_label: String,
}

pub fn to_default_data_export_mapping_view(item: &DefaultDataExportMapping) -> DefaultDataExportMappingView {
    DefaultDataExportMappingView {
        code: item.code.clone(),
        label: item.label.clone(),
        expires_at_text: item.expires_at.to_string(),
        name_text: item.name.clone(),
        namespace_text: item.namespace.clone(),
        category: item.category.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        contact_email_text: item.contact_email.clone(),
        scope: text::truncate(&item.scope, 80),
        updated_at: item.updated_at.to_string(),
        rate_text: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_data_export_mapping_views(items: &[DefaultDataExportMapping]) -> Vec<DefaultDataExportMappingView> {
    let mut views: Vec<DefaultDataExportMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_data_export_mapping_view(item));
    }
    views
}

pub fn sum_default_data_export_mapping_view(view: &DefaultDataExportMappingView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_ceiling + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_data_export_mapping("code-780", "label-49");
        let view = to_default_data_export_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_data_export_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
