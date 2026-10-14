use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive usage segment.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageSegmentView {
    pub code_text: String,
    pub name: String,
    pub ratio: String,
    pub ends_at: String,
    pub scope_text: String,
    pub status_label: String,
}

pub fn to_adaptive_usage_segment_view(item: &AdaptiveUsageSegment) -> AdaptiveUsageSegmentView {
    AdaptiveUsageSegmentView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 80),
        ratio: format!("{:.2}", item.ratio),
        ends_at: item.ends_at.to_string(),
        scope_text: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_usage_segment_views(items: &[AdaptiveUsageSegment]) -> Vec<AdaptiveUsageSegmentView> {
    let mut views: Vec<AdaptiveUsageSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_usage_segment_view(item));
    }
    views
}

pub fn sum_adaptive_usage_segment_view(view: &AdaptiveUsageSegmentView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ratio + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_usage_segment("code-496", "name-342");
        let view = to_adaptive_usage_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_usage_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
