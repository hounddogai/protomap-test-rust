use super::*;

use acme_kit::text;

/// How screens and exports show a data export assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAssignmentView {
    pub code: String,
    pub label: String,
    pub weight: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_data_export_assignment_view(item: &DataExportAssignment) -> DataExportAssignmentView {
    DataExportAssignmentView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        weight: format!("{:.2}", item.weight),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_assignment_views(items: &[DataExportAssignment]) -> Vec<DataExportAssignmentView> {
    let mut views: Vec<DataExportAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_assignment_view(item));
    }
    views
}

pub fn sum_data_export_assignment_view(view: &DataExportAssignmentView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_weight + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_assignment("code-82", "label-514");
        let view = to_data_export_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
