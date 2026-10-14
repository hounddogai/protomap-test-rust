use super::*;

use acme_kit::text;

/// How screens and exports show a grant segment.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantSegmentView {
    pub code: String,
    pub label_text: String,
    pub ratio_text: String,
    pub key_id: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_grant_segment_view(item: &GrantSegment) -> GrantSegmentView {
    GrantSegmentView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        ratio_text: format!("{:.2}", item.ratio),
        key_id: item.key_id.clone(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_segment_views(items: &[GrantSegment]) -> Vec<GrantSegmentView> {
    let mut views: Vec<GrantSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_segment_view(item));
    }
    views
}

pub fn sum_grant_segment_view(view: &GrantSegmentView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_ratio + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_segment("code-105", "label-584");
        let view = to_grant_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
