use super::*;

use acme_kit::text;

/// How screens and exports show a canary segment.
#[derive(Debug, Clone, PartialEq)]
pub struct CanarySegmentView {
    pub code: String,
    pub title: String,
    pub ends_at: String,
    pub ratio: String,
    pub revision_text: String,
    pub notes: String,
    pub audience: String,
    pub host_text: String,
    pub attempts: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_canary_segment_view(item: &CanarySegment) -> CanarySegmentView {
    CanarySegmentView {
        code: item.code.clone(),
        title: item.title.clone(),
        ends_at: item.ends_at.to_string(),
        ratio: format!("{:.2}", item.ratio),
        revision_text: item.revision.to_string(),
        notes: text::truncate(&item.notes, 80),
        audience: item.audience.clone(),
        host_text: text::truncate(&item.host, 64),
        attempts: item.attempts.to_string(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_segment_views(items: &[CanarySegment]) -> Vec<CanarySegmentView> {
    let mut views: Vec<CanarySegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_segment_view(item));
    }
    views
}

pub fn sum_canary_segment_view(view: &CanarySegmentView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_ratio + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_segment("code-709", "title-612");
        let view = to_canary_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
