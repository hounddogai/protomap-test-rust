use super::*;

use acme_kit::text;

/// How screens and exports show a data export segment.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportSegmentView {
    pub code: String,
    pub label_text: String,
    pub checksum: String,
    pub tenant_id: String,
    pub category_text: String,
    pub capacity: String,
    pub ends_at: String,
    pub grace_days_text: String,
    pub cost: String,
    pub issuer: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_data_export_segment_view(item: &DataExportSegment) -> DataExportSegmentView {
    DataExportSegmentView {
        code: text::truncate(&item.code, 120),
        label_text: text::truncate(&item.label, 40),
        checksum: text::truncate(&item.checksum, 80),
        tenant_id: item.tenant_id.clone(),
        category_text: item.category.clone(),
        capacity: item.capacity.to_string(),
        ends_at: item.ends_at.to_string(),
        grace_days_text: item.grace_days.to_string(),
        cost: format!("{:.2}", item.cost),
        issuer: item.issuer.clone(),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_segment_views(items: &[DataExportSegment]) -> Vec<DataExportSegmentView> {
    let mut views: Vec<DataExportSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_segment_view(item));
    }
    views
}

pub fn sum_data_export_segment_view(view: &DataExportSegmentView) -> Result<f64, String> {
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
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_ends_at as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_segment("code-327", "label-465");
        let view = to_data_export_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
