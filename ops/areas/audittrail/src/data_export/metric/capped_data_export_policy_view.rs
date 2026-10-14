use super::*;

use acme_kit::text;

/// How screens and exports show a capped data export policy.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportPolicyView {
    pub code: String,
    pub title_text: String,
    pub currency: String,
    pub category_text: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_capped_data_export_policy_view(item: &CappedDataExportPolicy) -> CappedDataExportPolicyView {
    CappedDataExportPolicyView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 80),
        currency: item.currency.clone(),
        category_text: item.category.clone(),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_data_export_policy_views(items: &[CappedDataExportPolicy]) -> Vec<CappedDataExportPolicyView> {
    let mut views: Vec<CappedDataExportPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_data_export_policy_view(item));
    }
    views
}

pub fn sum_capped_data_export_policy_view(view: &CappedDataExportPolicyView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_data_export_policy("code-231", "title-235");
        let view = to_capped_data_export_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_data_export_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
