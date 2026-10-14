use super::*;

use acme_kit::text;

/// How screens and exports show a data export template.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportTemplateView {
    pub code_text: String,
    pub name: String,
    pub currency: String,
    pub external_ref_text: String,
    pub approved_text: String,
    pub capacity: String,
    pub ends_at: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_data_export_template_view(item: &DataExportTemplate) -> DataExportTemplateView {
    DataExportTemplateView {
        code_text: text::truncate(&item.code, 120),
        name: item.name.clone(),
        currency: text::truncate(&item.currency, 40),
        external_ref_text: item.external_ref.clone(),
        approved_text: format!("{}", item.approved),
        capacity: item.capacity.to_string(),
        ends_at: item.ends_at.to_string(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_template_views(items: &[DataExportTemplate]) -> Vec<DataExportTemplateView> {
    let mut views: Vec<DataExportTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_template_view(item));
    }
    views
}

pub fn sum_data_export_template_view(view: &DataExportTemplateView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_ends_at as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_template("code-684", "name-117");
        let view = to_data_export_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
