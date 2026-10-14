use super::*;

use acme_kit::text;

/// How screens and exports show a segment baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentBaselineView {
    pub code_text: String,
    pub title_text: String,
    pub timezone_text: String,
    pub ratio: String,
    pub archived: String,
    pub duration_minutes_text: String,
    pub status_label: String,
}

pub fn to_segment_baseline_view(item: &SegmentBaseline) -> SegmentBaselineView {
    SegmentBaselineView {
        code_text: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        timezone_text: item.timezone.clone(),
        ratio: format!("{:.2}", item.ratio),
        archived: format!("{}", item.archived),
        duration_minutes_text: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_baseline_views(items: &[SegmentBaseline]) -> Vec<SegmentBaselineView> {
    let mut views: Vec<SegmentBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_baseline_view(item));
    }
    views
}

pub fn sum_segment_baseline_view(view: &SegmentBaselineView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_ratio + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_baseline("code-871", "title-821");
        let view = to_segment_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
