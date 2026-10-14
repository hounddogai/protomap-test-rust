use super::*;

/// How screens and exports show a segment mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMappingView {
    pub code: String,
    pub title: String,
    pub score: String,
    pub label: String,
    pub priority: String,
    pub approved_text: String,
    pub margin_text: String,
    pub cluster: String,
    pub sequence_text: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_segment_mapping_view(item: &SegmentMapping) -> SegmentMappingView {
    SegmentMappingView {
        code: item.code.clone(),
        title: item.title.clone(),
        score: format!("{:.2}", item.score),
        label: item.label.clone(),
        priority: item.priority.to_string(),
        approved_text: format!("{}", item.approved),
        margin_text: format!("{:.2}", item.margin),
        cluster: item.cluster.clone(),
        sequence_text: item.sequence.to_string(),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_mapping_views(items: &[SegmentMapping]) -> Vec<SegmentMappingView> {
    let mut views: Vec<SegmentMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_mapping_view(item));
    }
    views
}

pub fn sum_segment_mapping_view(view: &SegmentMappingView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_score + (parsed_priority as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_mapping("code-93", "title-156");
        let view = to_segment_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
