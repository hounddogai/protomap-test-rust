use super::*;

/// How screens and exports show a weighted data export segment.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportSegmentView {
    pub code: String,
    pub label: String,
    pub score: String,
    pub ends_at: String,
    pub cluster_text: String,
    pub margin: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_weighted_data_export_segment_view(item: &WeightedDataExportSegment) -> WeightedDataExportSegmentView {
    WeightedDataExportSegmentView {
        code: item.code.clone(),
        label: item.label.clone(),
        score: format!("{:.2}", item.score),
        ends_at: item.ends_at.to_string(),
        cluster_text: item.cluster.clone(),
        margin: format!("{:.2}", item.margin),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_data_export_segment_views(
    items: &[WeightedDataExportSegment],
) -> Vec<WeightedDataExportSegmentView> {
    let mut views: Vec<WeightedDataExportSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_data_export_segment_view(item));
    }
    views
}

pub fn sum_weighted_data_export_segment_view(view: &WeightedDataExportSegmentView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_score + (parsed_ends_at as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_data_export_segment("code-597", "label-891");
        let view = to_weighted_data_export_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_data_export_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
