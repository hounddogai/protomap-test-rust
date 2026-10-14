use super::*;

use acme_kit::text;

/// How screens and exports show a capped data export allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportAllocationView {
    pub code: String,
    pub name: String,
    pub priority_text: String,
    pub title: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_capped_data_export_allocation_view(item: &CappedDataExportAllocation) -> CappedDataExportAllocationView {
    CappedDataExportAllocationView {
        code: item.code.clone(),
        name: item.name.clone(),
        priority_text: item.priority.to_string(),
        title: text::truncate(&item.title, 80),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_data_export_allocation_views(
    items: &[CappedDataExportAllocation],
) -> Vec<CappedDataExportAllocationView> {
    let mut views: Vec<CappedDataExportAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_data_export_allocation_view(item));
    }
    views
}

pub fn sum_capped_data_export_allocation_view(view: &CappedDataExportAllocationView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_priority as f64 + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_data_export_allocation("code-949", "name-604");
        let view = to_capped_data_export_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_data_export_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
